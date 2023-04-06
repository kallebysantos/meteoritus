use std::{
    error::Error,
    fs::{self, File},
    io::{BufReader, ErrorKind, Seek, SeekFrom, Write},
    path::Path,
};

use super::{
    file_info::{Built, Completed, Created, FileInfo},
    metadata::Metadata,
};

pub enum PatchOption {
    Patched(u64),
    Completed(FileInfo<Completed>),
}

#[derive(Debug)]
pub enum VaultError {
    CreationError(Box<dyn Error>),
    ReadError(Box<dyn Error>),
    Error,
}

pub trait Vault: Send + Sync {
    fn build_file(
        &self,
        length: u64,
        metadata: Option<&str>,
    ) -> Result<FileInfo<Built>, VaultError>;

    fn create_file(
        &self,
        file: FileInfo<Built>,
    ) -> Result<FileInfo<Created>, VaultError>;

    fn exists(&self, file_id: &str) -> bool;

    fn get_file(&self, file_id: &str) -> Result<FileInfo<Created>, VaultError>;

    fn patch_file(
        &self,
        file_id: &str,
        buf: &mut [u8],
        offset: u64,
    ) -> Result<PatchOption, VaultError>;
}

pub struct LocalVault {
    save_path: &'static str,
}

impl LocalVault {
    pub fn new(save_path: &'static str) -> Self {
        Self { save_path }
    }
}

impl Vault for LocalVault {
    fn build_file(
        &self,
        length: u64,
        metadata: Option<&str>,
    ) -> Result<FileInfo<Built>, VaultError> {
        let metadata = match metadata {
            Some(metadata) => match Metadata::try_from(metadata) {
                Ok(m) => m,
                Err(e) => return Err(VaultError::CreationError(Box::new(e))),
            },

            None => Metadata::default(),
        };

        let file_info = FileInfo::new(length)
            .with_uuid()
            .with_metadata(metadata)
            .build();

        Ok(file_info)
    }

    fn create_file(
        &self,
        file_info: FileInfo<Built>,
    ) -> Result<FileInfo<Created>, VaultError> {
        let file_dir = Path::new(self.save_path).join(&file_info.id());

        if !file_dir.exists() {
            if let Err(e) = fs::create_dir_all(&file_dir).map_err(|e| e.into())
            {
                return Err(VaultError::CreationError(e));
            };
        }

        /* Creating file for upload */
        if let Err(e) = match File::create_new(file_dir.join("file")) {
            Ok(file) => file.set_len(*file_info.length()).map_err(|e| e.into()),
            Err(e) => Err(e.into()),
        } {
            return Err(VaultError::CreationError(e));
        };

        /* Storing file info */
        if let Err(e) =
            match File::create_new(file_dir.join("info").with_extension("json"))
            {
                Ok(info) => serde_json::to_writer(info, &file_info)
                    .map_err(|e| e.into()),
                Err(e) => Err(e.into()),
            }
        {
            return Err(VaultError::CreationError(e));
        };

        /* Retrieving disk file_path as &str */
        let Some(file_name) = file_dir.as_path().to_str() else {
            return Err(VaultError::CreationError(Box::new(
                std::io::Error::from(ErrorKind::InvalidFilename),
            )))
        };

        Ok(file_info.mark_as_created(file_name))
    }

    fn exists(&self, file_id: &str) -> bool {
        let file_dir = Path::new(self.save_path).join(file_id);
        let file_path = file_dir.join("file");
        let file_info_path = file_dir.join("info").with_extension("json");

        file_dir.exists() && file_path.exists() && file_info_path.exists()
    }

    fn get_file(&self, file_id: &str) -> Result<FileInfo<Created>, VaultError> {
        let file_dir = Path::new(self.save_path).join(file_id);

        let info_path = file_dir.join("info").with_extension("json");

        let file = match File::open(info_path) {
            Ok(file) => file,
            Err(e) => return Err(VaultError::CreationError(e.into())),
        };

        let reader = BufReader::new(file);

        serde_json::from_reader(reader)
            .map_err(|e| VaultError::ReadError(e.into()))
    }

    fn patch_file(
        &self,
        file_id: &str,
        buf: &mut [u8],
        offset: u64,
    ) -> Result<PatchOption, VaultError> {
        let mut file = self.get_file(file_id)?;

        if *file.offset() != offset {
            return Err(VaultError::Error);
        }

        let file_dir = Path::new(self.save_path).join(file_id);

        let file_path = file_dir.join("file");

        let mut file_content =
            File::options().write(true).open(file_path).unwrap();

        file_content.seek(SeekFrom::Start(offset)).unwrap();

        let written_bytes = file_content.write(buf).unwrap();

        if written_bytes >= u64::MAX as usize {
            return Err(VaultError::Error);
        }

        let offset = offset + written_bytes as u64;
        file.set_offset(offset).unwrap();

        let file_info_path = file_dir.join("info").with_extension("json");

        let mut file_info =
            File::options().write(true).open(file_info_path).unwrap();

        serde_json::to_writer(&mut file_info, &file).unwrap();

        match file.check_completion() {
            Some(file) => Ok(PatchOption::Completed(file)),
            None => Ok(PatchOption::Patched(offset)),
        }
    }
}

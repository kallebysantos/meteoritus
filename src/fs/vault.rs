use std::{
    error::Error,
    fs::{self, File},
    io::ErrorKind,
    path::Path,
};

use super::{
    file_info::{Built, Created, FileInfo},
    metadata::Metadata,
};

#[derive(Debug)]
pub enum VaultError {
    CreationError(Box<dyn Error>),
}

pub trait Vault: Send + Sync {
    fn build_file(
        &self,
        length: u64,
        metadata: Option<&str>,
    ) -> Result<FileInfo<Built>, VaultError>;

    fn create_file(&self, file: FileInfo<Built>) -> Result<FileInfo<Created>, VaultError>;
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

    fn create_file(&self, file_info: FileInfo<Built>) -> Result<FileInfo<Created>, VaultError> {
        let file_dir = Path::new(self.save_path).join(&file_info.id());

        if !file_dir.exists() {
            if let Err(e) = fs::create_dir_all(&file_dir) {
                return Err(VaultError::CreationError(Box::new(e)));
            };
        }

        /* Creating file for upload */
        if let Err(e) = match File::create_new(file_dir.join("file")) {
            Ok(file) => file.set_len(*file_info.length()),
            Err(e) => Err(e),
        } {
            return Err(VaultError::CreationError(Box::new(e)));
        };

        /* Storing file info */
        if let Err(Some(e)) = match File::create_new(file_dir.join("info").with_extension("json")) {
            Ok(info) => serde_json::to_writer(info, &file_info).or_else(|e| Err(e.source())),
            Err(e) => Err(e.source()),
        } {
            return Err(VaultError::CreationError(Box::new(e)));
        };

        /* Retrieving disk file_path as &str */
        let Some(file_name) = file_dir.as_path().to_str() else {
            return Err(VaultError::CreationError(Box::new(
                std::io::Error::from(ErrorKind::InvalidFilename),
            )))
        };

        Ok(file_info.mark_as_created(file_name))
    }
}

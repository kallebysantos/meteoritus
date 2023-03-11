use std::io::{prelude::*, SeekFrom};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Error, ErrorKind, Result},
    path::Path,
};

use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CometFile {
    id: String,
    length: u64,
    offset: u64,
    metadata: Option<HashMap<String, String>>,
}

impl CometFile {
    pub fn new(length: u64) -> Self {
        Self {
            length,
            ..CometFile::default()
        }
    }

    pub fn with_uuid(&mut self) -> Self {
        self.with_raw_id(Uuid::new_v4().simple().to_string())
    }

    pub fn with_raw_id(&mut self, id: String) -> Self {
        self.id = id;
        self.to_owned()
    }

    pub fn with_metadata(&mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self.to_owned()
    }

    pub fn set_offset(&mut self, offset: u64) -> Result<()> {
        if offset > self.length {
            return Err(Error::from(ErrorKind::OutOfMemory));
        }

        self.offset = offset;

        Ok(())
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn length(&self) -> &u64 {
        &self.length
    }

    pub fn metadata(&self) -> &Option<HashMap<String, String>> {
        &self.metadata
    }

    pub fn offset(&self) -> &u64 {
        &self.offset
    }
}

pub trait CometVault: Send + Sync {
    fn add(&self, file: &CometFile) -> Result<()>;

    fn take(&self, id: String) -> Result<CometFile>;

    fn update(&self, file: &mut CometFile, buf: &mut [u8]) -> Result<()>;

    fn remove(&self, file: &CometFile) -> Result<()>;
}

pub struct MeteorVault {
    save_path: &'static str,
}

impl MeteorVault {
    pub fn new(save_path: &'static str) -> Self {
        Self { save_path }
    }
}

impl CometVault for MeteorVault {
    fn add(&self, file: &CometFile) -> Result<()> {
        let file_dir = Path::new(self.save_path).join(&file.id);

        if !file_dir.exists() {
            fs::create_dir_all(&file_dir)?;
        }

        let file_path = file_dir.join(&file.id);
        File::create_new(file_path)?.set_len(file.length)?;

        let info_path = file_dir.join(&file.id).with_extension("json");

        let mut info_file = File::create_new(&info_path)?;
        serde_json::to_writer(&mut info_file, &file)?;

        Ok(())
    }

    fn take(&self, id: String) -> Result<CometFile> {
        let file_dir = Path::new(self.save_path).join(&id);

        let info_path = file_dir.join(&id).with_extension("json");

        let file = File::open(info_path)?;
        let reader = BufReader::new(file);

        let info: CometFile = serde_json::from_reader(reader)?;

        Ok(info)
    }

    fn update(&self, file: &mut CometFile, buf: &mut [u8]) -> Result<()> {
        let file_dir = Path::new(self.save_path).join(&file.id);

        let file_path = file_dir.join(&file.id);

        let mut file_content = File::options().write(true).open(file_path)?;

        file_content.seek(SeekFrom::Start(file.offset))?;

        let written_bytes = file_content.write(buf)?;

        if written_bytes >= u64::MAX as usize {
            return Err(Error::from(ErrorKind::OutOfMemory));
        }

        file.set_offset(file.offset + written_bytes as u64)?;

        let info_path = file_dir.join(&file.id).with_extension("json");

        let mut file_info = File::options().write(true).open(info_path)?;

        serde_json::to_writer(&mut file_info, &file)?;

        Ok(())
    }

    fn remove(&self, _file: &CometFile) -> Result<()> {
        todo!()
    }
}

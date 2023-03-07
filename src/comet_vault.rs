use std::{
    collections::HashMap,
    fs::{self, File},
    io::Error,
    path::Path,
};

use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CometFile {
    id: String,
    length: u64,
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

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn length(&self) -> &u64 {
        &self.length
    }
    pub fn metadata(&self) -> &Option<HashMap<String, String>> {
        &self.metadata
    }
}

pub trait CometVault: Send + Sync {
    fn add(&self, file: &CometFile) -> Result<(), Error>;

    fn take(&self, id: String) -> Result<CometFile, Error>;

    fn remove(&self, file: &CometFile) -> Result<(), Error>;
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
    fn add(&self, file: &CometFile) -> Result<(), Error> {
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

    fn take(&self, _id: String) -> Result<CometFile, Error> {
        todo!()
    }

    fn remove(&self, _file: &CometFile) -> Result<(), Error> {
        todo!()
    }
}

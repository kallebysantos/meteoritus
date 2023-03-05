use std::{collections::HashMap, io::Error};

use uuid::Uuid;

#[derive(Default, Clone)]
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

pub struct MeteorVault {}

impl MeteorVault {
    pub fn new() -> Self {
        Self {}
    }
}

impl CometVault for MeteorVault {
    fn add(&self, file: &CometFile) -> Result<(), Error> {
        Ok(())
    }

    fn take(&self, id: String) -> Result<CometFile, Error> {
        todo!()
    }

    fn remove(&self, file: &CometFile) -> Result<(), Error> {
        todo!()
    }
}

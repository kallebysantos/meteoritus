use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fs::metadata::Metadata;
use std::{
    io::{Error, ErrorKind, Result},
    marker::PhantomData,
};

#[derive(Default)]
pub struct Building;

#[derive(Default)]
pub struct Created;

#[derive(Default)]
pub struct Uploading;

#[derive(Default)]
pub struct Completed;

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FileInfo<State = Building> {
    id: String,
    filename: String,
    length: u64,
    offset: u64,
    metadata: Option<Metadata>,
    state: PhantomData<State>,
}

impl<State> FileInfo<State> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }

    pub fn length(&self) -> &u64 {
        &self.length
    }

    pub fn metadata(&self) -> &Option<Metadata> {
        &self.metadata
    }
}

impl FileInfo {
    pub fn new(length: u64) -> Self {
        Self {
            length,
            ..Default::default()
        }
    }

    pub fn with_uuid(self) -> Self {
        self.with_raw_id(Uuid::new_v4().simple().to_string())
    }

    pub fn with_raw_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn build(self) -> FileInfo<Created> {
        FileInfo::<Created> {
            state: std::marker::PhantomData,
            ..self
        }
    }
}

impl FileInfo<Uploading> {
    pub fn offset(&self) -> &u64 {
        &self.offset
    }

    pub fn set_offset(mut self, offset: u64) -> Result<()> {
        if offset > self.length {
            return Err(Error::from(ErrorKind::OutOfMemory));
        }

        self.offset = offset;

        Ok(())
    }

    pub fn check_completion(self) -> Option<FileInfo<Completed>> {
        if self.offset != self.length {
            return None;
        }

        Some(FileInfo::<Completed> {
            state: std::marker::PhantomData,
            ..self
        })
    }
}

use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fs::metadata::Metadata;
use std::{
    io::{Error, ErrorKind, Result},
    marker::PhantomData,
};

/// Indicates the [`FileInfo`] `Created` state.
#[derive(Default, Debug)]
pub struct Building;

/// Indicates the [`FileInfo`] `Built` state.
#[derive(Default, Debug)]
pub struct Built;

/// Indicates the [`FileInfo`] `Created` state.
#[derive(Default, Debug)]
pub struct Created;

/// Indicates the [`FileInfo`] `Completed` state.
#[derive(Default, Debug)]
pub struct Completed;

/// A struct representing a file and its metadata during various stages of processing.
///
/// The struct has four possible states: [`Built`], [`Created`], and [`Completed`].
/// - [`Built`] - The file instances has been built and is ready to create information on disk.
/// - [`Created`] - The file information has been saved on disk.
/// - [`Completed`] - The file has been fully processed and is ready to be used.
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FileInfo<State = Building> {
    id: String,
    file_name: String,
    length: u64,
    offset: u64,
    metadata: Option<Metadata>,

    #[serde(skip)]
    state: PhantomData<State>,
}

impl<State> FileInfo<State> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn length(&self) -> &u64 {
        &self.length
    }

    pub fn metadata(&self) -> &Option<Metadata> {
        &self.metadata
    }
}

impl FileInfo<Building> {
    pub(super) fn new(length: u64) -> Self {
        Self {
            length,
            ..Default::default()
        }
    }

    pub(super) fn with_uuid(self) -> Self {
        self.with_raw_id(Uuid::new_v4().simple().to_string())
    }

    pub(super) fn with_raw_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub(super) fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub(super) fn build(self) -> FileInfo<Built> {
        FileInfo::<Built> {
            state: std::marker::PhantomData,
            ..self
        }
    }
}

impl FileInfo<Built> {
    pub(super) fn mark_as_created(self, file_name: &str) -> FileInfo<Created> {
        FileInfo::<Created> {
            file_name: file_name.to_string(),
            state: std::marker::PhantomData,
            ..self
        }
    }
}

impl FileInfo<Created> {
    pub fn offset(&self) -> &u64 {
        &self.offset
    }

    pub(super) fn set_offset(&mut self, offset: u64) -> Result<()> {
        if offset > self.length {
            return Err(Error::from(ErrorKind::OutOfMemory));
        }

        self.offset = offset;

        Ok(())
    }

    pub(crate) fn check_completion(self) -> Option<FileInfo<Completed>> {
        if self.offset != self.length {
            return None;
        }

        Some(FileInfo::<Completed> {
            state: std::marker::PhantomData,
            ..self
        })
    }
}

impl FileInfo<Completed> {
    /// Returns where the file is located
    pub fn file_name(&self) -> &String {
        &self.file_name
    }
}

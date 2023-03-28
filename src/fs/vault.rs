use std::error::Error;

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
        todo!()
    }
}

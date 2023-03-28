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

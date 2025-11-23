mod debug;
mod filesystem;

use crate::error::{DeleteError, LoadError, SaveError};
use crate::settings::StorageCommands;
use crate::storage::{debug::DebugStorage, filesystem::FileSystemStorage};
use rocket::fs::{NamedFile, TempFile};

pub enum InputFile<'r> {
    TempFile(&'r mut TempFile<'r>),
    Bytes(&'r [u8]),
}
#[rocket::async_trait]
pub trait Storage: Send + Sync {
    async fn save<'r>(&self, file: InputFile<'r>, filename: &str) -> Result<(), SaveError>;
    async fn load(&self, filename: &str) -> Result<NamedFile, LoadError>;
    async fn delete(&self, filename: &str) -> Result<(), DeleteError>;
}

pub(super) fn init(storage_config: &StorageCommands) -> Box<dyn Storage> {
    match storage_config {
        StorageCommands::FileSystem { path } => Box::new(FileSystemStorage::new(path)),
        StorageCommands::Debug => Box::new(DebugStorage::new()),
    }
}

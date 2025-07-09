use super::{InputFile, Storage};
use crate::error::*;
use rocket::fs::NamedFile;
use std::path::Path;

pub struct FileSystemStorage {
    path: &'static Path,
}

#[rocket::async_trait]
impl Storage for FileSystemStorage {
    async fn save<'r>(&self, file: InputFile<'r>, filename: &str) -> Result<(), SaveError> {
        match file {
            InputFile::TempFile(file) => {
                file.move_copy_to(self.path.join(filename))
                    .await
                    .map_err(SaveError::new)?;
            }
            InputFile::Bytes(stream) => {
                tokio::fs::write(self.path.join(filename), stream)
                    .await
                    .map_err(SaveError::new)?;
            }
        }

        Ok(())
    }

    async fn load(&self, filename: &str) -> Result<NamedFile, LoadError> {
        let file_path = self.path.join(filename);
        if file_path.exists() {
            NamedFile::open(file_path)
                .await
                .map_err(|e| LoadError::PermissionDenied(e.to_string()))
        } else {
            Err(LoadError::FileNotExist(format!(
                "file {filename} does not exist"
            )))
        }
    }

    async fn delete(&self, filename: &str) -> Result<(), DeleteError> {
        let file_path = self.path.join(filename);
        tokio::fs::remove_file(file_path)
            .await
            .map_err(DeleteError::new)
    }
}

impl FileSystemStorage {
    pub(super) fn new(path: &'static Path) -> Self {
        FileSystemStorage { path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::{path::PathBuf, sync::LazyLock};
    use tokio::io::AsyncReadExt;

    const TEST_FILE_NAME: &str = "test.png";
    static FILE_PATH: LazyLock<PathBuf> =
        LazyLock::new(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/"));
    static TEST_FILE: LazyLock<PathBuf> = LazyLock::new(|| FILE_PATH.join(TEST_FILE_NAME));
    static FILE_HASH: LazyLock<[u8; 32]> = LazyLock::new(|| {
        let bytes = std::fs::read(&*TEST_FILE).unwrap();
        <[u8; 32]>::from(Sha256::digest(bytes))
    });

    #[tokio::test]
    async fn save_test_file() {
        let storage = FileSystemStorage::new(&FILE_PATH);
        let bytes = tokio::fs::read(&*TEST_FILE).await.unwrap();

        storage
            .save(InputFile::Bytes(&bytes), "testfile.png")
            .await
            .unwrap();
        let saved_bytes = tokio::fs::read(FILE_PATH.join("testfile.png"))
            .await
            .unwrap();
        let result = Sha256::digest(saved_bytes);

        assert_eq!(result[..], FILE_HASH[..]);
        tokio::fs::remove_file(FILE_PATH.join("testfile.png"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn load_test_file() {
        let storage = FileSystemStorage::new(&FILE_PATH);

        let loaded = storage.load(TEST_FILE_NAME).await.unwrap();
        let mut buffer = vec![0; loaded.metadata().await.unwrap().len() as usize];
        loaded.take_file().read_exact(&mut buffer).await.unwrap();
        let result = Sha256::digest(buffer);
        assert_eq!(result[..], FILE_HASH[..]);
    }

    #[tokio::test]
    async fn delete_test_file() {
        let storage = FileSystemStorage::new(&FILE_PATH);

        tokio::fs::write(FILE_PATH.join("delete_test.txt"), "delete test")
            .await
            .unwrap();
        storage.delete("delete_test.txt").await.unwrap();
        assert!(!FILE_PATH.join("delete_test.txt").exists());
    }
}

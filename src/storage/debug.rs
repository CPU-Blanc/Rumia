use crate::error::{DeleteError, LoadError, SaveError};
use crate::storage::{InputFile, Storage};
use rocket::fs::NamedFile;
use std::{collections::HashMap, sync::Arc};
use tokio::{io::AsyncReadExt, sync::Mutex};

pub(crate) struct DebugStorage {
    store: Mutex<HashMap<String, Arc<[u8]>>>,
}

#[allow(clippy::unwrap_used)]
#[rocket::async_trait]
impl Storage for DebugStorage {
    async fn save<'r>(&self, file: InputFile<'r>, filename: &str) -> Result<(), SaveError> {
        match file {
            InputFile::Bytes(bytes) => {
                self.store
                    .lock()
                    .await
                    .insert(String::from(filename), Arc::from(bytes));
            }
            InputFile::TempFile(file) => {
                let mut buffer = vec![0; usize::try_from(file.len()).unwrap()];

                file.open()
                    .await
                    .unwrap()
                    .read_exact(&mut buffer)
                    .await
                    .unwrap();

                self.store
                    .lock()
                    .await
                    .insert(String::from(filename), Arc::from(buffer));
            }
        }

        Ok(())
    }

    async fn load(&self, filename: &str) -> Result<NamedFile, LoadError> {
        let data;
        {
            data = Arc::clone(self.store.lock().await.get(filename).ok_or(
                LoadError::FileNotExist(format!("file {filename} does not exist")),
            )?);
        }

        tokio::fs::write(filename, &*data).await.unwrap();
        let file = NamedFile::open(filename).await;
        tokio::fs::remove_file(filename).await.unwrap();
        file.map_err(|_| LoadError::FileNotExist(format!("file {filename} does not exist")))
    }

    async fn delete(&self, filename: &str) -> Result<(), DeleteError> {
        self.store.lock().await.remove(filename);
        Ok(())
    }
}

impl DebugStorage {
    pub(super) fn new() -> Self {
        DebugStorage {
            store: Mutex::new(HashMap::new()),
        }
    }
}

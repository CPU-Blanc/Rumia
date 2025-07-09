#![allow(dead_code)]

use rocket::{http::ContentType, local::blocking::Client};
use rumia::{STORAGE, server, storage::InputFile};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};
use uuid::Uuid;

pub(crate) const TEST_FILE_NAME: &str = "test.png";
pub(crate) static FILE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/"));
pub(crate) static TEST_FILE: LazyLock<PathBuf> = LazyLock::new(|| FILE_PATH.join(TEST_FILE_NAME));
pub(crate) static FILE_HASH: LazyLock<[u8; 32]> = LazyLock::new(|| {
    let bytes = std::fs::read(&*TEST_FILE).unwrap();
    <[u8; 32]>::from(Sha256::digest(bytes))
});

pub(crate) fn setup_client() -> Client {
    Client::untracked(server()).unwrap()
}

pub(crate) fn create_new_test_file() -> String {
    let uuid = Uuid::new_v4().to_string();
    let bytes = std::fs::read(&*TEST_FILE).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        STORAGE
            .save(InputFile::Bytes(&bytes), &format!("{uuid}.png"))
            .await
            .unwrap()
    });
    uuid
}
const BOUNDARY: &str = "------------------------ea3bbcf87c101592";

pub(crate) fn get_image_data<T: AsRef<Path>>(filepath: T, fake: bool) -> (ContentType, Vec<u8>) {
    let ct = format!("multipart/form-data; boundary={BOUNDARY}")
        .parse::<ContentType>()
        .unwrap();

    let filename = filepath
        .as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let ext = filename.split_once(".").unwrap().1;

    let mut file_data = if fake {
        vec![]
    } else {
        std::fs::read(filepath).unwrap()
    };

    let file_type = ContentType::from_extension(ext).unwrap().to_string();
    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
    result
        .extend_from_slice("Content-Disposition: form-data; name=\"filename\"\r\n\r\n".as_bytes());
    result.extend_from_slice(format!("{filename}\r\n").as_bytes());
    result.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
    result.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
            .as_bytes(),
    );
    result.extend_from_slice(format!("Content-Type: {file_type}\r\n\r\n").as_bytes());
    result.append(&mut file_data);
    result.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n\r\n").as_bytes());
    (ct, result)
}

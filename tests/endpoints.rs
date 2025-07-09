use tokio::io::AsyncReadExt;
mod common;

use crate::common::{
    FILE_HASH, FILE_PATH, TEST_FILE, create_new_test_file, get_image_data, setup_client,
};
use rocket::http::{Header, Status};
use rumia::STORAGE;
use sha2::{Digest, Sha256};
use std::path::Path;

const PROTECTED: [(Method, &str); 3] = [
    (Method::POST, "/api/upload/file"),
    (Method::POST, "/api/upload/https%3A%2F%2Fgoogle.com"),
    (Method::DELETE, "/attachment/543543/test.png"),
];

#[allow(clippy::upper_case_acronyms)]
#[allow(dead_code)]
#[derive(Debug)]
enum Method {
    GET,
    POST,
    DELETE,
}

#[test]
fn healthcheck() {
    let client = setup_client();
    let resp = client.get("/health").dispatch();

    assert_eq!(resp.status(), Status::Ok);
    assert_eq!(resp.into_string().unwrap(), "ok");
}

#[test]
fn can_auth() {
    let client = setup_client();
    let resp = client
        .delete("/attachment/543543/543543.gif".to_string())
        .header(Header::new("x-api-key", "12345"))
        .dispatch();

    assert_eq!(resp.status(), Status::BadRequest);
}

#[test]
fn cannot_access_protected_endpoints() {
    let client = setup_client();

    let mut failed = false;

    for (method, uri) in PROTECTED {
        let resp = match method {
            Method::GET => client.get(uri).dispatch(),
            Method::POST => client.post(uri).dispatch(),
            Method::DELETE => client.delete(uri).dispatch(),
        };

        if resp.status() != Status::Unauthorized {
            println!(
                "endpoint {uri} failed to respond UNAUTHORIZED - got {}",
                resp.status()
            );
            failed = true;
        };
    }

    assert!(!failed);
}

#[test]
fn upload_file_from_binary() {
    let client = setup_client();
    let (ct, data) = get_image_data(&*TEST_FILE, false);

    let resp = client
        .post("/api/upload/file")
        .header(Header::new("x-api-key", "12345"))
        .header(ct)
        .body(data)
        .dispatch();

    assert_eq!(resp.status(), Status::Ok);

    let resp = resp.into_string().unwrap();
    let strings: Vec<&str> = resp.split("/").collect();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let loaded = STORAGE.load(&format!("{}.png", strings[4])).await.unwrap();
        let mut buffer = vec![0; loaded.metadata().await.unwrap().len() as usize];
        loaded.take_file().read_exact(&mut buffer).await.unwrap();
        Sha256::digest(buffer)
    });

    assert_eq!(result[..], FILE_HASH[..]);
}

#[test]
fn upload_file_from_url() {
    let client = setup_client();

    let resp = client.post("/api/upload/https%3A%2F%2Fraw.githubusercontent.com%2FCPU-Blanc%2FRumia%2Frefs%2Fheads%2Fmaster%2Fresources%2Ftest%2Ftest.png")
        .header(Header::new("x-api-key", "12345"))
        .dispatch();

    assert_eq!(resp.status(), Status::Ok);

    let resp = resp.into_string().unwrap();
    let strings: Vec<&str> = resp.split("/").collect();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let loaded = STORAGE.load(&format!("{}.png", strings[4])).await.unwrap();
        let mut buffer = vec![0; loaded.metadata().await.unwrap().len() as usize];
        loaded.take_file().read_exact(&mut buffer).await.unwrap();
        Sha256::digest(buffer)
    });

    assert_eq!(result[..], FILE_HASH[..]);
}

#[test]
fn cannot_upload_bad_binary() {
    let client = setup_client();
    let (ct, body) = get_image_data("test.exe", true);

    let resp = client
        .post("/api/upload/file")
        .header(Header::new("x-api-key", "12345"))
        .header(ct)
        .body(body)
        .dispatch();

    assert_eq!(resp.status(), Status::UnsupportedMediaType);
}

#[test]
fn cannot_upload_bad_url() {
    let client = setup_client();

    let resp = client
        .post("/api/upload/https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ")
        .header(Header::new("x-api-key", "12345"))
        .dispatch();

    assert_eq!(resp.status(), Status::BadRequest);
}

#[test]
fn can_get_test_file() {
    let uuid = create_new_test_file();

    let client = setup_client();
    let resp = client
        .get(format!("/attachment/{uuid}/test.png"))
        .dispatch();

    assert_eq!(resp.status(), Status::Ok);

    let data = resp.into_bytes().unwrap();

    assert!(!data.is_empty());

    let bytes = Sha256::digest(data);
    assert_eq!(FILE_HASH[..], bytes[..]);
}

#[test]
fn random_file_return_404() {
    let client = setup_client();
    let resp = client
        .get("/attachment/39514323-7c35-4e01-95b0-4a6b55c550c1/test.gif")
        .dispatch();

    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn delete_file() {
    let client = setup_client();
    let uuid = create_new_test_file();

    let resp = client
        .delete(format!("/attachment/{uuid}/test.png"))
        .header(Header::new("x-api-key", "12345"))
        .dispatch();

    assert_eq!(resp.status(), Status::Ok);
    assert!(!Path::new(&FILE_PATH.join(uuid + ".png")).exists());
}

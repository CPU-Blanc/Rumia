use crate::{SETTINGS, STORAGE, error::ApiKeyError, storage::InputFile};
use rocket::{
    Request,
    form::{Form, Strict},
    fs::{NamedFile, TempFile},
    http::Status,
    outcome::Outcome,
    request::FromRequest,
};
use std::{path::Path, str::FromStr};
use url::Url;
use uuid::Uuid;

#[derive(PartialEq)]
pub(crate) struct ApiKey<'r>(&'r str);

#[derive(FromForm)]
pub(crate) struct Upload<'r> {
    file: TempFile<'r>,
    filename: String,
}

const BLACKLISTED_EXT: [&str; 6] = ["exe", "dll", "html", "css", "php", "pub"];
const BLACKLISTED_NAME: [&str; 2] = ["_rsa", "_ed25519"];

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(
        request: &'r Request<'_>,
    ) -> Outcome<ApiKey<'r>, (Status, ApiKeyError), Status> {
        fn is_valid(key: &str) -> bool {
            !key.is_empty()
        }

        match request.headers().get_one("x-api-key") {
            None => Outcome::Error((Status::Unauthorized, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Error((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}

#[post("/api/upload/file", data = "<upload>")]
pub(crate) async fn upload_file(
    key: ApiKey<'_>,
    mut upload: Form<Strict<Upload<'_>>>,
) -> Result<String, Status> {
    validate_key(key)?;

    let (filename, extension) = validate_file(&upload.filename)?;
    let hash = Uuid::new_v4().to_string();
    let save_name = format!("{hash}.{extension}");

    STORAGE
        .save(InputFile::TempFile(&mut upload.file), &save_name)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(format!("{}/attachment/{hash}/{filename}", &SETTINGS.url))
}

#[post("/api/upload/<url>")]
pub(crate) async fn upload_file_url(key: ApiKey<'_>, url: &str) -> Result<String, Status> {
    validate_key(key)?;

    let url = Url::parse(url).map_err(|_| Status::BadRequest)?;

    let filename = Path::new(url.path())
        .file_name()
        .ok_or(Status::BadRequest)?
        .to_str()
        .unwrap();

    let (_, extension) = validate_file(filename)?;
    let hash = Uuid::new_v4().to_string();
    let save_name = format!("{hash}.{extension}");

    let resp = reqwest::get(url.clone())
        .await
        .map_err(|_| Status::BadGateway)?
        .error_for_status()
        .map_err(|error| {
            let status = error.status().unwrap();
            Status::from_code(status.as_u16()).unwrap()
        })?;

    let bytes = resp.bytes().await.map_err(|_| Status::FailedDependency)?;

    STORAGE
        .save(InputFile::Bytes(&bytes), &save_name)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(format!("{}/attachment/{hash}/{filename}", &SETTINGS.url))
}

#[get("/attachment/<hash>/<filename>")]
pub(crate) async fn get_file(hash: &str, filename: &str) -> Result<NamedFile, Status> {
    let (_, extension) = validate_file(filename)?;
    let hash = validate_hash(hash)?;
    let filename = format!("{hash}.{extension}");

    STORAGE.load(&filename).await.map_err(|_| Status::NotFound)
}

#[delete("/attachment/<hash>/<filename>")]
pub async fn delete_file(key: ApiKey<'_>, hash: &str, filename: &str) -> Result<(), Status> {
    validate_key(key)?;
    let (_, extension) = validate_file(filename)?;
    let hash = validate_hash(hash)?;
    let filename = format!("{hash}.{extension}");
    STORAGE
        .delete(&filename)
        .await
        .map_err(|_| Status::NotFound)
}

fn validate_file(filename: &str) -> Result<(String, String), Status> {
    let path = Path::new(filename);

    let filename = path
        .file_name()
        .ok_or(Status::BadRequest)?
        .to_str()
        .unwrap()
        .to_owned();

    let extension = path
        .extension()
        .ok_or(Status::BadRequest)?
        .to_str()
        .unwrap()
        .to_owned();

    if BLACKLISTED_EXT.iter().any(|ext| *ext == extension)
        || BLACKLISTED_NAME.iter().any(|name| filename.contains(name))
    {
        Err(Status::UnsupportedMediaType)
    } else {
        Ok((filename, extension))
    }
}

fn validate_hash<T: AsRef<str>>(hash: T) -> Result<String, Status> {
    Ok(Uuid::from_str(hash.as_ref())
        .map_err(|_| Status::BadRequest)?
        .to_string())
}

fn validate_key(provided: ApiKey<'_>) -> Result<(), Status> {
    if provided.0 != SETTINGS.api_key {
        Err(Status::Unauthorized)
    } else {
        Ok(())
    }
}

#[cfg(all(feature = "cli", feature = "docker"))]
compile_error!(
    "feature \"cli\" and feature \"docker\" cannot be enabled at the same time. Use \"--no-default-features --features docker\" if you wish to build for docker"
);

use crate::settings::StorageCommands;
use rocket::{
    Build, Rocket,
    config::LogLevel,
    data::{Limits, ToByteUnit},
};
use routes::*;
use settings::Settings;
use std::{net::Ipv4Addr, sync::LazyLock};
use storage::Storage;

mod error;
mod routes;
mod settings;
pub mod storage;

#[macro_use]
extern crate rocket;

pub(crate) static SETTINGS: LazyLock<Settings> = {
    if cfg!(debug_assertions) {
        LazyLock::new(|| Settings {
            api_key: "12345",
            port: 10032,
            url: "http://localhost",
            verbose: true,
            ip: Ipv4Addr::new(0, 0, 0, 0),
            storage_type: StorageCommands::Debug,
        })
    } else {
        LazyLock::new(Settings::new)
    }
};

pub static STORAGE: LazyLock<Box<dyn Storage>> =
    LazyLock::new(|| storage::init(&SETTINGS.storage_type));

pub fn server() -> Rocket<Build> {
    let config = rocket::Config {
        port: SETTINGS.port,
        address: SETTINGS.ip.into(),
        limits: Limits::default()
            .limit("data-form", 200.megabytes())
            .limit("file", 200.megabytes()),
        log_level: if SETTINGS.verbose {
            LogLevel::Normal
        } else {
            LogLevel::Critical
        },
        ..rocket::Config::default()
    };

    rocket::custom(config).mount(
        "/",
        routes![
            healthcheck,
            upload_file,
            upload_file_url,
            delete_file,
            get_file,
        ],
    )
}

#[get("/health")]
async fn healthcheck() -> &'static str {
    "ok"
}

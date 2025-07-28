#[cfg(feature = "cli")]
use clap::{Parser, Subcommand, ValueEnum};
use dotenv::dotenv;
#[cfg(feature = "docker")]
use std::env;
use std::{net::Ipv4Addr, path::Path, str::FromStr};

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum StorageType {
    #[default]
    File,
}

impl FromStr for StorageType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "file" => Ok(Self::File),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Subcommand))]
#[cfg_attr(feature = "cli", command(about, version))]
pub enum StorageCommands {
    /// Use the filesystem for file storage
    FileSystem {
        #[cfg_attr(feature = "cli", arg(long, env = "RUMIA_FILESYSTEM_PATH", value_parser = return_leaked_path))]
        path: &'static Path,
    },
    Debug,
}

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(feature = "cli", command(version, about))]
pub struct Settings {
    #[cfg_attr(feature = "cli", arg(short, long, env = "RUMIA_API_KEY", value_parser = return_leaked_str))]
    pub api_key: &'static str,

    #[cfg_attr(
        feature = "cli",
        arg(short, long, env = "RUMIA_PORT", default_value_t = 10032)
    )]
    pub port: u16,

    #[cfg_attr(feature = "cli", arg(short, long, env = "RUMIA_URL", default_value = "http://localhost", value_parser = return_leaked_str))]
    pub url: &'static str,

    #[cfg_attr(
        feature = "cli",
        arg(short, long, env = "RUMIA_VERBOSE", default_value_t = false)
    )]
    pub verbose: bool,

    #[cfg_attr(
        feature = "cli",
        arg(short, long, env = "RUMIA_IP", default_value = "0.0.0.0")
    )]
    pub ip: Ipv4Addr,

    #[cfg_attr(feature = "cli", command(subcommand))]
    pub storage_type: StorageCommands,
}

#[cfg(feature = "cli")]
fn return_leaked_str(s: &str) -> Result<&'static str, String> {
    Ok(s.to_owned().leak())
}

#[cfg(feature = "cli")]
fn return_leaked_path(s: &str) -> Result<&'static Path, String> {
    Ok(Box::leak(Box::from(Path::new(s))))
}

impl Settings {
    pub(crate) fn new() -> Self {
        dotenv().ok();

        #[cfg(feature = "docker")]
        {
            Settings {
                api_key: Box::leak(Box::from(
                    env::var("RUMIA_API_KEY").expect("API key not provided"),
                )),
                port: env::var("RUMIA_PORT")
                    .map(|s| s.parse().unwrap())
                    .unwrap_or(10032),
                url: env::var("RUMIA_URL")
                    .unwrap_or(String::from("http://localhost"))
                    .leak(),
                verbose: env::var("RUMIA_VERBOSE")
                    .unwrap_or(String::from("false").to_lowercase())
                    .parse()
                    .unwrap(),
                ip: env::var("RUMIA_IP")
                    .unwrap_or(String::from("0.0.0.0"))
                    .parse()
                    .unwrap(),
                storage_type: match env::var("RUMIA_STORAGE")
                    .unwrap_or(String::from("FILE"))
                    .parse::<StorageType>()
                    .unwrap()
                {
                    StorageType::File => StorageCommands::FileSystem {
                        path: Path::new(
                            env::var("RUMIA_FILESYSTEM_PATH")
                                .unwrap_or(String::from("/filestore"))
                                .leak(),
                        ),
                    },
                },
            }
        }

        #[cfg(feature = "cli")]
        {
            Settings::parse()
        }
    }
}

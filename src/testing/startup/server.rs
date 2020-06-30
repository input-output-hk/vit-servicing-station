use crate::server_settings::ServiceSettings;
use assert_cmd;
use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;
use thiserror::Error;

use crate::testing::server::Server;

const BIN_NAME: &str = "vit-servicing-station";

pub struct Starter {
    settings: ServiceSettings,
}

impl Starter {
    pub fn new() -> Self {
        Self {
            settings: Default::default(),
        }
    }

    pub fn with_localhost_address(&mut self, port: u32) -> &mut Self {
        self.settings.address = format!("127.0.0.1:{}", port).parse().unwrap();
        self
    }

    pub fn with_db_path<S: Into<String>>(&mut self, db_url: S) -> &mut Self {
        self.settings.db_url = db_url.into();
        self
    }

    pub fn with_block0_path<S: Into<String>>(&mut self, block0_path: S) -> &mut Self {
        self.settings.block0_path = block0_path.into();
        self
    }

    pub fn start(&self) -> Result<Server, StarterError> {
        let mut command = Command::cargo_bin(BIN_NAME)?;
        command
            .arg("--db-url")
            .arg(self.settings.db_url.to_string())
            .arg("--block0-path")
            .arg(self.settings.block0_path.to_string());

        let child = command.spawn()?;

        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(Server::new(child, self.settings.clone()))
    }
}

#[derive(Debug, Error)]
pub enum StarterError {
    #[error("cannot spawn process")]
    ProcessSpawnError(#[from] std::io::Error),
    #[error("cannot find binary (0)")]
    CargoError(#[from] assert_cmd::cargo::CargoError),
}

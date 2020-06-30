use crate::server_settings::ServiceSettings;
use crate::testing::rest_client::RestClient;
use std::process::Child;

pub struct Server {
    process: Child,
    settings: ServiceSettings,
}

impl Server {
    pub fn new(process: Child, settings: ServiceSettings) -> Self {
        Self { process, settings }
    }

    pub fn rest_client(&self) -> RestClient {
        RestClient::new(self.settings.address.to_string())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        self.process.wait().unwrap();
    }
}

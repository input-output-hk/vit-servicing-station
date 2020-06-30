use crate::db::models::funds::Fund;
use thiserror::Error;

pub struct RestClient {
    address: String,
    api_token: Option<String>,
}

impl RestClient {
    pub fn new(address: String) -> Self {
        Self {
            address,
            api_token: None,
        }
    }

    pub fn funds(&self) -> Result<Vec<Fund>, RestError> {
        let content = self.get("funds")?.text()?;
        println!("Response: {}", content);

        if content.is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let request = self.path(path);
        println!("Request: {}", request);
        let client = reqwest::blocking::Client::new();
        let mut res = client.get(&request);

        if let Some(api_token) = &self.api_token {
            res = res.header("API-Token", api_token.to_string());
        }
        res.send()
    }

    fn path(&self, path: &str) -> String {
        format!("http://{}/api/v0/{}", self.address, path)
    }

    pub fn set_api_token(&mut self, token: String) {
        self.api_token = Some(token);
    }
}

#[derive(Debug, Error)]
pub enum RestError {
    #[error("could not deserialize response")]
    CannotDeserialize(#[from] serde_json::Error),
    #[error("could not send reqeuest")]
    RequestError(#[from] reqwest::Error),
}

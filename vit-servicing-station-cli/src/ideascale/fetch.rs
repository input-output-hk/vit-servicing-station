use serde::Deserialize;

use std::convert::TryInto;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
pub struct Challenge {
    id: u64,
    #[serde(alias = "name")]
    title: String,
    description: String,
    #[serde(alias = "groupId")]
    fund_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct Fund {
    id: u64,
    name: String,
    #[serde(alias = "campaigns")]
    challenges: Vec<Challenge>,
}

lazy_static::lazy_static!(
    static ref BASE_IDEASCALE_URL: url::Url = "https://apitest.ideascale.com/a/rest/v1/".try_into().unwrap();
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
);

async fn get_funds_data(api_token: &str) -> Result<Vec<Fund>, Error> {
    CLIENT
        .get(BASE_IDEASCALE_URL.join("campaigns/groups").unwrap())
        .header("api_token", api_token)
        .send()
        .await?
        .json()
        .await
        .map_err(Error::RequestError)
}

// async fn get_challenges_data(api_token: &str) -> Result<Vec<>>

#[cfg(test)]
mod tests {
    use crate::ideascale::fetch::get_funds_data;

    #[tokio::test]
    async fn test_request_campaigns() {
        let results = get_funds_data("asdf")
            .await
            .expect("All current campaigns data");
        println!("{}", results.len());
        for campaign in results {
            println!("{:?}", campaign);
        }
    }
}

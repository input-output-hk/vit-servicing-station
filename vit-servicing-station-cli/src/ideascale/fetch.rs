use super::models::{Fund, Proposal};

use serde::Deserialize;

use crate::ideascale::models::{Challenge, Funnel};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::convert::TryInto;
use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error("Could not get value from json, missing attribute {attribute_name}")]
    MissingAttribute { attribute_name: &'static str },
}

#[derive(Debug, Deserialize)]
struct Score {
    #[serde(alias = "ideaId")]
    id: i32,
    #[serde(alias = "avgScoreOfIdea")]
    score: f32,
}

pub type Scores = HashMap<i32, f32>;

lazy_static::lazy_static!(
    static ref BASE_IDEASCALE_URL: url::Url = "https://apitest.ideascale.com/a/rest/v1/".try_into().unwrap();
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
);

async fn request_data<T: DeserializeOwned>(api_token: String, url: Url) -> Result<T, Error> {
    CLIENT
        .get(url)
        .header("api_token", api_token)
        .send()
        .await?
        .json()
        .await
        .map_err(Error::RequestError)
}

pub async fn get_funds_data(api_token: String) -> Result<Vec<Fund>, Error> {
    request_data(
        api_token,
        BASE_IDEASCALE_URL.join("campaigns/groups").unwrap(),
    )
    .await
}

const ASSESSMENT_ID_ATTR: &str = "assessmentId";

pub async fn get_assessment_id(stage_id: i32, api_token: String) -> Result<i64, Error> {
    let assessment: serde_json::Value = request_data(
        api_token,
        BASE_IDEASCALE_URL
            .join(&format!("stages/{}", stage_id))
            .unwrap(),
    )
    .await?;
    // should be safe to unwrap that the value is an i64
    Ok(assessment
        .get(ASSESSMENT_ID_ATTR)
        .ok_or(Error::MissingAttribute {
            attribute_name: ASSESSMENT_ID_ATTR,
        })?
        .as_i64()
        .unwrap())
}

pub async fn get_assessments_score(assessment_id: i64, api_token: String) -> Result<Scores, Error> {
    let scores: Vec<Score> = request_data(
        api_token,
        BASE_IDEASCALE_URL
            .join(&format!("assessment/{}/results", assessment_id))
            .unwrap(),
    )
    .await?;
    Ok(scores.into_iter().map(|s| (s.id, s.score)).collect())
}

pub async fn get_proposals_data(
    challenge_id: i32,
    api_token: String,
) -> Result<Vec<Proposal>, Error> {
    request_data(
        api_token,
        BASE_IDEASCALE_URL
            .join(&format!("campaigns/{}/ideas", challenge_id))
            .unwrap(),
    )
    .await
}

pub async fn get_funnels_data_for_fund(
    fund: usize,
    api_token: String,
) -> Result<Vec<Funnel>, Error> {
    let challenges: Vec<Funnel> = request_data(
        api_token,
        BASE_IDEASCALE_URL.join(&format!("funnels")).unwrap(),
    )
    .await?;
    Ok(challenges
        .into_iter()
        .filter(|f| f.title.starts_with(&format!("Fund {}", fund)))
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::ideascale::fetch::{
        get_assessment_id, get_assessments_score, get_funds_data, get_funnels_data_for_fund,
        get_proposals_data,
    };
    const API_TOKEN: &str = "";
    #[tokio::test]
    async fn test_fetch_funds() {
        let results = get_funds_data(API_TOKEN)
            .await
            .expect("All current campaigns data");
        println!("{}", results.len());
        for campaign in results {
            println!("{:?}", campaign);
        }
    }

    #[tokio::test]
    async fn test_assessment_scores() {
        let assessment_id = get_assessment_id(76890, API_TOKEN).await.unwrap();
        let assessments_scores = get_assessments_score(assessment_id, API_TOKEN)
            .await
            .unwrap();
        for score in assessments_scores {
            println!("{:?}", score);
        }
    }

    #[tokio::test]
    async fn test_fetch_proposals() {
        let proposals = get_proposals_data(25939, API_TOKEN).await.unwrap();
        for proposal in proposals {
            println!("{:?}", proposal);
        }
    }

    #[tokio::test]
    async fn test_fetch_funnels() {
        let proposals = get_funnels_data_for_fund(4, API_TOKEN).await.unwrap();
        for proposal in proposals {
            println!("{:?}", proposal);
        }
    }
}

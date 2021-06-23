use serde::Deserialize;

use std::convert::TryInto;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error("Could not get value from json, missing attribute {attribute_name}")]
    MissingAttribute { attribute_name: &'static str },
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

#[derive(Debug, Deserialize)]
pub struct Proposal {
    #[serde(alias = "id")]
    proposal_id: i32,
    proposal_category: String,
    #[serde(alias = "title")]
    proposal_title: String,
    #[serde(alias = "text")]
    proposal_summary: String,
    #[serde(alias = "describe_your_solution_to_the_problem")]
    proposal_solution: String,
    #[serde(alias = "ada_payment_address__must_be_a_shelly_address__starting_with__addr__")]
    proposal_public_key: String,
    #[serde(alias = "requested_funds_in_ada")]
    proposal_funds: i64,
    #[serde(alias = "url")]
    proposal_url: String,
    #[serde(default)]
    proposal_files_url: String,
    #[serde(default)]
    proposal_impact_score: i64,
    #[serde(alias = "relevant_experience")]
    proposal_relevant_experience: String,

    #[serde(alias = "why_is_it_important_")]
    proposal_why: String,
}

#[derive(Debug, Deserialize)]
struct Score {
    #[serde(alias = "ideaId")]
    id: i32,
    #[serde(alias = "avgScoreOfIdea")]
    score: f32,
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

const ASSESSMENT_ID_ATTR: &str = "assessmentId";

async fn get_assessment_id(stage_id: i32, api_token: &str) -> Result<i64, Error> {
    let assesment: serde_json::Value = CLIENT
        .get(
            BASE_IDEASCALE_URL
                .join(&format!("stages/{}", stage_id))
                .unwrap(),
        )
        .header("api_token", api_token)
        .send()
        .await?
        .json()
        .await?;
    // should be safe to unwrap that the value is an i64
    Ok(assesment
        .get(ASSESSMENT_ID_ATTR)
        .ok_or_else(|| Error::MissingAttribute {
            attribute_name: ASSESSMENT_ID_ATTR,
        })?
        .as_i64()
        .unwrap())
}

async fn get_assessments_score(assessment_id: i64, api_token: &str) -> Result<Vec<Score>, Error> {
    CLIENT
        .get(
            BASE_IDEASCALE_URL
                .join(&format!("assessment/{}/results", assessment_id))
                .unwrap(),
        )
        .header("api_token", api_token)
        .send()
        .await?
        .json()
        .await
        .map_err(Error::RequestError)
}

#[cfg(test)]
mod tests {
    use crate::ideascale::fetch::{get_assessment_id, get_assessments_score, get_funds_data};
    const API_TOKEN: &str = "";
    #[tokio::test]
    async fn test_request_campaigns() {
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
}

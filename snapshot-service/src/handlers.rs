use crate::SharedContext;
use jormungandr_lib::crypto::account::Identifier;
use serde_json::json;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

#[tracing::instrument(skip(context))]
pub async fn get_voting_power(
    tag: String,
    voting_key: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    let key = if let Ok(key) = Identifier::from_hex(&voting_key) {
        key
    } else {
        return Ok(warp::reply::with_status(
            "Invalid voting key",
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .into_response());
    };

    match context.get_voting_power(tag, key).await {
        Some(entries) => {
            let results: Vec<_> = entries.into_iter().map(|(voting_group, voting_power)| {
            json!({"voting_power": voting_power, "voting_group": voting_group})
        }).collect();
            Ok(warp::reply::json(&results).into_response())
        }
        None => Err(warp::reject::not_found()),
    }
}

#[tracing::instrument(skip(context))]
pub async fn get_tags(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&context.get_tags().await))
}

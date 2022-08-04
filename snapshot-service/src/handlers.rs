use std::sync::Arc;

use crate::{SharedContext, UpdateHandle, VoterInfo};
use catalyst_toolbox::snapshot::SnapshotInfo;
use jormungandr_lib::crypto::account::Identifier;
use serde_json::json;
use tokio::sync::Mutex;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

#[tracing::instrument(skip(context))]
pub async fn get_voters_info(
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

    match tokio::task::spawn_blocking(move || context.get_voters_info(&tag, &key))
        .await
        .unwrap()
    {
        Ok(Some(entries)) => {
            let results: Vec<_> = entries.into_iter().map(|VoterInfo{group: voting_group, voting_power,delegations_power, delegations_count}| {
            json!({"voting_power": voting_power, "voting_group": voting_group, "delegations_power": delegations_power, "delegations_count": delegations_count})
        }).collect();
            Ok(warp::reply::json(&results).into_response())
        }
        Ok(None) => Err(warp::reject::not_found()),
        Err(_) => Ok(
            warp::reply::with_status("Database error", StatusCode::INTERNAL_SERVER_ERROR)
                .into_response(),
        ),
    }
}

#[tracing::instrument(skip(context))]
pub async fn get_tags(context: SharedContext) -> Result<impl Reply, Rejection> {
    match context.get_tags().map(|tags| warp::reply::json(&tags)) {
        Ok(tags) => Ok(tags.into_response()),
        Err(_) => Ok(warp::reply::with_status(
            "Failed to get tags from database",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response()),
    }
}

#[tracing::instrument(skip(context))]
pub async fn put_tag(
    tag: String,
    snapshot: Vec<SnapshotInfo>,
    context: Arc<Mutex<UpdateHandle>>,
) -> Result<impl Reply, Rejection> {
    let mut handle = context.lock().await;

    match handle.update(&tag, snapshot).await {
        Err(crate::Error::InternalError) => Ok(warp::reply::with_status(
            "Consistency error",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response()),
        Err(e) => Ok(
            warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                .into_response(),
        ),
        Ok(_) => Ok(warp::reply().into_response()),
    }
}

// use crate::{
//     db::{models::snapshot::SnapshotEntry, schema::snapshot},
//     v0::{context::SharedContext, errors::HandleError, result::HandlerResult},
// };
// use diesel::prelude::*;
use crate::SharedContext;
use jormungandr_lib::crypto::account::Identifier;
use serde_json::json;
use warp::{Rejection, Reply};

#[derive(Debug)]
struct InternalError;

impl warp::reject::Reject for InternalError {}

#[tracing::instrument(skip(context))]
pub async fn get_voting_power(
    tag: String,
    voting_key: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    let key = Identifier::from_hex(&voting_key).unwrap();

    match context.get_voting_power(tag, key).await {
        Some(entries) => {
            let results: Vec<_> = entries.into_iter().map(|(voting_group, voting_power)| {
            json!({"voting_power": voting_power, "voting_group": voting_group})
        }).collect();
            Ok(warp::reply::json(&results))
        }
        None => Err(warp::reject::not_found()),
    }
}

#[tracing::instrument(skip(context))]
pub async fn get_tags(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&context.get_tags().await))
}

use crate::{
    db::{models::snapshot::SnapshotEntry, schema::snapshot},
    v0::{context::SharedContext, errors::HandleError, result::HandlerResult},
};
use diesel::prelude::*;
use serde_json::json;
use warp::{Rejection, Reply};

#[tracing::instrument(skip(context))]
pub async fn get_voting_power(
    tag: String,
    voting_key: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    let pool = context.read().await.db_connection_pool.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
        crate::db::schema::snapshot::table
            .filter(snapshot::voting_key.eq(&voting_key))
            .filter(snapshot::tag.eq(&tag))
            .load::<SnapshotEntry>(&db_conn)
            .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))??;

    if result.is_empty() {
        return Err(warp::reject::custom(HandleError::NotFound(
            "voting key not found".to_string(),
        )));
    }

    let selected = result
        .into_iter()
        .map(
            |SnapshotEntry {
                 voting_power,
                 voting_group,
                 ..
             }| json!({"voting_power": voting_power as u64, "voting_group": voting_group}),
        )
        .collect::<Vec<_>>();

    Ok(warp::reply::json(&selected))
}

#[tracing::instrument(skip(context))]
pub async fn get_tags(context: SharedContext) -> Result<impl Reply, Rejection> {
    let pool = context.read().await.db_connection_pool.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
        crate::db::schema::snapshot::table
            .select(crate::db::schema::snapshot::tag)
            .distinct()
            .load::<String>(&db_conn)
            .map_err(|_e| {
                HandleError::InternalError("Couldn't load tags from database".to_string())
            })
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?;

    Ok(HandlerResult(result))
}

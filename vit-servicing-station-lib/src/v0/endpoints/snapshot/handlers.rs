use crate::{
    db::{models::snapshot::SnapshotEntry, schema::snapshot},
    v0::{context::SharedContext, errors::HandleError, result::HandlerResult},
};
use diesel::prelude::*;
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
            .first::<SnapshotEntry>(&db_conn)
            .map_err(|_e| HandleError::NotFound(voting_key))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?;

    Ok(HandlerResult(result.map(|v| v.voting_power as u64)))
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

use crate::{
    db::{models::snapshot::Snapshot, schema::snapshot},
    v0::{context::SharedContext, errors::HandleError, result::HandlerResult},
};
use diesel::prelude::*;
use warp::{Rejection, Reply};

#[tracing::instrument(skip(context))]
pub async fn get_voting_power(
    voting_key: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    let pool = context.read().await.db_connection_pool.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
        crate::db::schema::snapshot::table
            .filter(snapshot::voting_key.eq(&voting_key))
            .first::<Snapshot>(&db_conn)
            .map_err(|_e| HandleError::NotFound(voting_key))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?;

    Ok(HandlerResult(result.map(|v| v.voting_power as u64)))
}

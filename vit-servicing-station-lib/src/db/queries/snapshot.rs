use crate::{
    db::{models::snapshot::Snapshot, schema::snapshot, DbConnection, DbConnectionPool},
    v0::errors::HandleError,
};
use diesel::{ExpressionMethods, Insertable, QueryResult, RunQueryDsl};

pub async fn query_all_snapshots(pool: &DbConnectionPool) -> Result<Vec<Snapshot>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::order_by(snapshot::dsl::snapshot, snapshot::dsl::last_updated.asc())
            .load::<Snapshot>(&db_conn)
            .map_err(|_| HandleError::InternalError("Error retrieving challenges".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_snapshot_by_tag(
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Snapshot, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::filter(snapshot::dsl::snapshot, snapshot::dsl::tag.eq(tag))
            .first::<Snapshot>(&db_conn)
            .map_err(|_e| HandleError::NotFound("Error loading challenge".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn batch_insert_challenges(
    snapshots: &[<Snapshot as Insertable<snapshot::table>>::Values],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(snapshot::table)
        .values(snapshots)
        .execute(db_conn)
}

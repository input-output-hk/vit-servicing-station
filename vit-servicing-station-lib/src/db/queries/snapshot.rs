use crate::{
    db::{
        models::snapshot::{Snapshot, VotingRegistration},
        schema::{snapshot, voting_registration},
        DbConnection, DbConnectionPool,
    },
    v0::errors::HandleError,
};
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl};

pub async fn query_all_snapshots(pool: &DbConnectionPool) -> Result<Vec<Snapshot>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::order_by(snapshot::dsl::snapshot, snapshot::dsl::last_updated.asc())
            .load(&db_conn)
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
            .first(&db_conn)
            .map_err(|_e| HandleError::NotFound("Error loading challenge".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn batch_insert_snapshots(
    snapshots: &[<Snapshot as Insertable<snapshot::table>>::Values],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(snapshot::table)
        .values(snapshots)
        .execute(db_conn)
}

pub async fn query_voting_registrations_by_snapshot_tag(
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Vec<VotingRegistration>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::filter(
            voting_registration::dsl::voting_registration,
            voting_registration::dsl::snapshot_tag.eq(tag),
        )
        .order_by(voting_registration::dsl::voting_power.asc())
        .load(&db_conn)
        .map_err(|_e| HandleError::NotFound("Error loading challenge".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn batch_insert_voting_registrations(
    snapshots: &[<VotingRegistration as Insertable<voting_registration::table>>::Values],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(voting_registration::table)
        .values(snapshots)
        .execute(db_conn)
}

use crate::{
    db::{
        models::snapshot::{Snapshot, Voter},
        schema::{snapshots, voters},
        DbConnection, DbConnectionPool,
    },
    v0::errors::HandleError,
};
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl};

pub async fn query_all_snapshots(pool: &DbConnectionPool) -> Result<Vec<Snapshot>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::order_by(
            snapshots::dsl::snapshots,
            snapshots::dsl::last_updated.asc(),
        )
        .load(&db_conn)
        .map_err(|e| HandleError::InternalError(format!("Error retrieving challenges: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub async fn query_snapshot_by_tag(
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Snapshot, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::filter(snapshots::dsl::snapshots, snapshots::dsl::tag.eq(tag))
            .first(&db_conn)
            .map_err(|e| HandleError::NotFound(format!("Error loading challenge: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub fn put_snapshot(snapshot: Snapshot, pool: &DbConnectionPool) -> Result<(), HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    diesel::replace_into(snapshots::table)
        .values(snapshot.values())
        .execute(&db_conn)
        .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?;
    Ok(())
}

// pub async fn query_voting_registrations_by_snapshot_tag(
//     tag: String,
//     pool: &DbConnectionPool,
// ) -> Result<Vec<VotingRegistration>, HandleError> {
//     let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
//     tokio::task::spawn_blocking(move || {
//         diesel::QueryDsl::filter(
//             voting_registration::dsl::voting_registration,
//             voting_registration::dsl::snapshot_tag.eq(tag),
//         )
//         .order_by(voting_registration::dsl::voting_power.asc())
//         .load(&db_conn)
//         .map_err(|e| HandleError::NotFound(format!("Error loading challenge: {}", e)))
//     })
//     .await
//     .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
// }

// pub fn batch_insert_voting_registrations(
//     snapshots: &[<VotingRegistration as Insertable<voting_registration::table>>::Values],
//     db_conn: &DbConnection,
// ) -> QueryResult<usize> {
//     diesel::insert_into(voting_registration::table)
//         .values(snapshots)
//         .execute(db_conn)
// }

// pub async fn query_delegations_by_snapshot_tag_and_delegator(
//     tag: String,
//     delegator: String,
//     pool: &DbConnectionPool,
// ) -> Result<Vec<Voter>, HandleError> {
//     let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
//     tokio::task::spawn_blocking(move || {
//         diesel::QueryDsl::filter(
//             delegation::dsl::delegation,
//             delegation::dsl::snapshot_tag.eq(tag),
//         )
//         .filter(delegation::dsl::delegator.eq(delegator))
//         .load(&db_conn)
//         .map_err(|e| HandleError::NotFound(format!("Error loading challenge: {}", e)))
//     })
//     .await
//     .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
// }

// pub fn batch_insert_delegations(
//     delegations: &[<Voter as Insertable<delegation::table>>::Values],
//     db_conn: &DbConnection,
// ) -> QueryResult<usize> {
//     diesel::insert_into(delegation::table)
//         .values(delegations)
//         .execute(db_conn)
// }

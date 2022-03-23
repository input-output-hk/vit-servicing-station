use super::super::schema::snapshot;
use diesel::Queryable;

#[derive(Queryable, Insertable)]
#[table_name = "snapshot"]
pub struct SnapshotEntry {
    pub voting_key: String,
    pub voting_power: i64,
}

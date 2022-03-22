use super::super::schema::snapshot;
use diesel::Queryable;

#[derive(Queryable)]
pub struct Snapshot {
    pub voting_key: String,
    pub voting_power: i64,
}

#[derive(Insertable)]
#[table_name = "snapshot"]
pub struct NewSnapshotEntry {
    pub voting_key: String,
    pub voting_power: i64,
}

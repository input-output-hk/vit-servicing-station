use crate::db::{schema::snapshot, Db};
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Tag - a unique identifier of the current snapshot
    pub tag: String,
    /// Timestamp for the latest update of the current snapshot
    #[serde(alias = "lastUpdated")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub last_updated: i64,
}

impl Queryable<snapshot::SqlType, Db> for Snapshot {
    type Row = (
        // 0 -> tag
        String,
        // 1 -> last_updated
        i64,
    );

    fn build(row: Self::Row) -> Self {
        Snapshot {
            tag: row.0,
            last_updated: row.1,
        }
    }
}

impl Insertable<snapshot::table> for Snapshot {
    type Values = (
        diesel::dsl::Eq<snapshot::tag, String>,
        diesel::dsl::Eq<snapshot::last_updated, i64>,
    );

    fn values(self) -> Self::Values {
        (
            snapshot::tag.eq(self.tag),
            snapshot::last_updated.eq(self.last_updated),
        )
    }
}

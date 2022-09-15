use crate::db::{
    schema::{contributions, snapshots, voters},
    Db,
};
use diesel::{ExpressionMethods, Insertable, Queryable};
use jormungandr_lib::crypto::account::Identifier;
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

impl Queryable<snapshots::SqlType, Db> for Snapshot {
    type Row = (
        // 0 -> tag
        String,
        // 1 -> last_updated
        i64,
    );

    fn build(row: Self::Row) -> Self {
        Self {
            tag: row.0,
            last_updated: row.1,
        }
    }
}

impl Insertable<snapshots::table> for Snapshot {
    type Values = (
        diesel::dsl::Eq<snapshots::tag, String>,
        diesel::dsl::Eq<snapshots::last_updated, i64>,
    );

    fn values(self) -> Self::Values {
        (
            snapshots::tag.eq(self.tag),
            snapshots::last_updated.eq(self.last_updated),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voter {
    #[serde(alias = "votingKey")]
    pub voting_key: Identifier,
    #[serde(alias = "votingPower")]
    pub voting_power: i64,
    #[serde(alias = "votingGroup")]
    pub voting_group: String,
    #[serde(alias = "snapshotTag")]
    pub snapshot_tag: String,
}

impl Queryable<voters::SqlType, Db> for Voter {
    type Row = (
        // 0 -> voting_key
        String,
        // 1 -> voting_power
        i64,
        // 2 -> voting_group
        String,
        // 3 -> snapshot_tag
        String,
    );

    fn build(row: Self::Row) -> Self {
        Self {
            voting_key: Identifier::from_hex(&row.0).expect("should hex decoded Identifier"),
            voting_power: row.1,
            voting_group: row.2,
            snapshot_tag: row.3,
        }
    }
}

impl Insertable<voters::table> for Voter {
    type Values = (
        diesel::dsl::Eq<voters::voting_key, String>,
        diesel::dsl::Eq<voters::voting_power, i64>,
        diesel::dsl::Eq<voters::voting_group, String>,
        diesel::dsl::Eq<voters::snapshot_tag, String>,
    );

    fn values(self) -> Self::Values {
        (
            voters::voting_key.eq(self.voting_key.to_hex()),
            voters::voting_power.eq(self.voting_power),
            voters::voting_group.eq(self.voting_group),
            voters::snapshot_tag.eq(self.snapshot_tag),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    #[serde(alias = "rewardAddress")]
    pub reward_address: String,
    pub value: i64,
    #[serde(alias = "votingKey")]
    pub voting_key: Identifier,
    #[serde(alias = "snapshotTag")]
    pub snapshot_tag: String,
}

impl Queryable<contributions::SqlType, Db> for Contribution {
    type Row = (
        // 0 -> reward_address
        String,
        // 1 -> value
        i64,
        // 2 -> voting_key
        String,
        // 3 -> snapshot_tag
        String,
    );

    fn build(row: Self::Row) -> Self {
        Self {
            reward_address: row.0,
            value: row.1,
            voting_key: Identifier::from_hex(&row.2).expect("should hex decoded Identifier"),
            snapshot_tag: row.3,
        }
    }
}

impl Insertable<contributions::table> for Contribution {
    type Values = (
        diesel::dsl::Eq<contributions::reward_address, String>,
        diesel::dsl::Eq<contributions::value, i64>,
        diesel::dsl::Eq<contributions::voting_key, String>,
        diesel::dsl::Eq<contributions::snapshot_tag, String>,
    );

    fn values(self) -> Self::Values {
        (
            contributions::reward_address.eq(self.reward_address),
            contributions::value.eq(self.value),
            contributions::voting_key.eq(self.voting_key.to_hex()),
            contributions::snapshot_tag.eq(self.snapshot_tag),
        )
    }
}

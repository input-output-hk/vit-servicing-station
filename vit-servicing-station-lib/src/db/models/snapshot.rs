use crate::db::{
    schema::{delegation, snapshot, voting_registration},
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

impl Queryable<snapshot::SqlType, Db> for Snapshot {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingRegistration {
    #[serde(alias = "stakePublicKey")]
    pub stake_public_key: String,
    #[serde(alias = "votingPower")]
    pub voting_power: i64,
    #[serde(alias = "rewardAddress")]
    pub reward_address: String,
    #[serde(alias = "votingPurpose")]
    pub voting_purpose: i64,
    #[serde(alias = "snapshotTag")]
    pub snapshot_tag: String,
}

impl Queryable<voting_registration::SqlType, Db> for VotingRegistration {
    type Row = (
        // 0 -> stake_public_key
        String,
        // 1 -> voting_power
        i64,
        // 2 -> reward_address
        String,
        // 3 -> voting_purpose
        i64,
        // 4 -> snapshot_tag
        String,
    );

    fn build(row: Self::Row) -> Self {
        Self {
            stake_public_key: row.0,
            voting_power: row.1,
            reward_address: row.2,
            voting_purpose: row.3,
            snapshot_tag: row.4,
        }
    }
}

impl Insertable<voting_registration::table> for VotingRegistration {
    type Values = (
        diesel::dsl::Eq<voting_registration::stake_public_key, String>,
        diesel::dsl::Eq<voting_registration::voting_power, i64>,
        diesel::dsl::Eq<voting_registration::reward_address, String>,
        diesel::dsl::Eq<voting_registration::voting_purpose, i64>,
        diesel::dsl::Eq<voting_registration::snapshot_tag, String>,
    );

    fn values(self) -> Self::Values {
        (
            voting_registration::stake_public_key.eq(self.stake_public_key),
            voting_registration::voting_power.eq(self.voting_power),
            voting_registration::reward_address.eq(self.reward_address),
            voting_registration::voting_purpose.eq(self.voting_purpose),
            voting_registration::snapshot_tag.eq(self.snapshot_tag),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delegation {
    pub representative: Identifier,
    pub weight: i32,
    pub delegator: String,
    #[serde(alias = "snapshotTag")]
    pub snapshot_tag: String,
}

impl Queryable<delegation::SqlType, Db> for Delegation {
    type Row = (
        // 0 -> representative
        String,
        // 1 -> weight
        i32,
        // 2 -> delegator
        String,
        // 3 -> snapshot_tag
        String,
    );

    fn build(row: Self::Row) -> Self {
        Self {
            representative: Identifier::from_hex(&row.0).expect("should hex decoded Identifier"),
            weight: row.1,
            delegator: row.2,
            snapshot_tag: row.3,
        }
    }
}

impl Insertable<delegation::table> for Delegation {
    type Values = (
        diesel::dsl::Eq<delegation::representative, String>,
        diesel::dsl::Eq<delegation::weight, i32>,
        diesel::dsl::Eq<delegation::delegator, String>,
        diesel::dsl::Eq<delegation::snapshot_tag, String>,
    );

    fn values(self) -> Self::Values {
        (
            delegation::representative.eq(self.representative.to_hex()),
            delegation::weight.eq(self.weight),
            delegation::delegator.eq(self.delegator),
            delegation::snapshot_tag.eq(self.snapshot_tag),
        )
    }
}

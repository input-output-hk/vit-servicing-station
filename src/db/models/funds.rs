use crate::db::{models::voteplans::Voteplan, schema::funds, DB};
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Fund {
    pub id: i32,
    pub fund_name: String,
    pub fund_goal: String,
    pub voting_power_info: String,
    pub rewards_info: String,
    pub fund_start_time: String,
    pub fund_end_time: String,
    pub next_fund_start_time: String,
    pub chain_vote_plans: Vec<Voteplan>,
}

impl Queryable<funds::SqlType, DB> for Fund {
    type Row = (
        // 0 -> id
        i32,
        // 1 -> fund_name
        String,
        // 2-> fund_goal
        String,
        // 3 -> voting_power_info
        String,
        // 4 -> rewards_info
        String,
        // 5 -> fund_start_time
        String,
        // 6 -> fund_end_time
        String,
        // 7 -> next_fund_start_time
        String,
    );

    fn build(row: Self::Row) -> Self {
        Fund {
            id: row.0,
            fund_name: row.1,
            fund_goal: row.2,
            voting_power_info: row.3,
            rewards_info: row.4,
            fund_start_time: row.5,
            fund_end_time: row.6,
            next_fund_start_time: row.7,
            chain_vote_plans: vec![],
        }
    }
}
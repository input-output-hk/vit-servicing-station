use serde::{Deserialize, Serialize};

use crate::db::models::{challenges::Challenge, proposals::FullProposalInfo};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Query {
    pub table: Table,
    pub filter: Vec<Constraint>,
    pub order_by: Vec<OrderBy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Constraint {
    pub search: String,
    pub column: Column,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct OrderBy {
    pub column: Column,
    #[serde(default)]
    pub descending: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Table {
    Challenges,
    Proposals,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Column {
    Title,
    Type,
    Desc,
    Author,
    Funds,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // should serialize as if it is either a `Vec<Challenge>` or `Vec<FullProposalInfo>`
pub enum SearchResponse {
    Challenge(Vec<Challenge>),
    Proposal(Vec<FullProposalInfo>),
}

#[cfg(test)]
mod tests {
    use serde_json::to_string;

    use crate::db::models::proposals::test::get_test_proposal;

    use super::*;

    #[test]
    fn response_serializes_as_vec() {
        let response = SearchResponse::Proposal(vec![get_test_proposal()]);
        let s = to_string(&response).unwrap();
        assert!(s.starts_with("["));
        assert!(s.ends_with("]"));
    }
}

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProposalVoteplanIdAndIndex {
    pub voteplan_id: String,
    pub indexes: Vec<i64>,
}

pub type ProposalsByVoteplanIdAndIndex = Vec<ProposalVoteplanIdAndIndex>;

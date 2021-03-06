mod arbitrary;
mod external;

pub use arbitrary::ArbitraryValidVotingTemplateGenerator;
pub use external::{
    parse_challenges, parse_funds, parse_proposals, ExternalValidVotingTemplateGenerator,
    TemplateLoadError,
};
use serde::{Deserialize, Serialize};
use vit_servicing_station_lib::db::models::proposals::{ChallengeType, ProposalChallengeInfo};

#[derive(Serialize, Deserialize, Clone)]
pub struct FundTemplate {
    pub id: i32,
    pub goal: String,
    pub rewards_info: String,
    pub threshold: Option<u32>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ProposalTemplate {
    pub internal_id: String,
    pub category_name: String,
    pub proposal_id: String,
    pub proposal_title: String,
    #[serde(default)]
    pub proposal_summary: String,
    pub proposal_funds: String,
    pub proposal_url: String,
    pub proposal_impact_score: String,
    #[serde(default)]
    pub files_url: String,
    pub proposer_name: String,
    #[serde(default)]
    pub proposer_url: String,
    #[serde(default)]
    pub proposer_relevant_experience: String,
    pub chain_vote_options: String,
    pub chain_vote_type: String,
    pub challenge_id: Option<String>,
    pub challenge_type: ChallengeType,
    #[serde(flatten)]
    pub proposal_challenge_info: ProposalChallengeInfo,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChallengeTemplate {
    pub id: String,
    pub challenge_type: ChallengeType,
    pub title: String,
    pub description: String,
    pub rewards_total: String,
    pub proposers_rewards: String,
    pub challenge_url: String,
    pub fund_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProposalChallengeInfoTemplate {
    pub id: i32,
}

pub trait ValidVotingTemplateGenerator {
    fn next_proposal(&mut self) -> ProposalTemplate;
    fn next_challenge(&mut self) -> ChallengeTemplate;
    fn next_fund(&mut self) -> FundTemplate;
}

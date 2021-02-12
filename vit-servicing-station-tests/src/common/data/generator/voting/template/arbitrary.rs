use super::{ChallengeTemplate, FundTemplate, ProposalTemplate, ValidVotingTemplateGenerator};
use crate::common::data::ArbitraryGenerator;
use fake::{
    faker::lorem::en::*,
    faker::{
        company::en::{Buzzword, CatchPhase, Industry},
        name::en::Name,
    },
    Fake,
};

#[derive(Clone)]
pub struct ArbitraryValidVotingTemplateGenerator {
    generator: ArbitraryGenerator,
    next_proposal_id: i32,
    next_challenge_id: i32,
}

impl Default for ArbitraryValidVotingTemplateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ArbitraryValidVotingTemplateGenerator {
    pub fn new() -> Self {
        Self {
            generator: ArbitraryGenerator::new(),
            next_proposal_id: 1,
            next_challenge_id: 1,
        }
    }

    pub fn next_challenge_id(&mut self) -> i32 {
        let ret = self.next_challenge_id;
        self.next_challenge_id = ret + 1;
        ret
    }

    pub fn next_proposal_id(&mut self) -> i32 {
        let ret = self.next_proposal_id;
        self.next_proposal_id = ret + 1;
        ret
    }
}

impl ValidVotingTemplateGenerator for ArbitraryValidVotingTemplateGenerator {
    fn next_proposal(&mut self) -> ProposalTemplate {
        let proposal_url = self.generator.gen_http_address();
        let challenge_type = self.generator.challenge_type();
        let proposal_challenge_info = self.generator.proposals_challenge_info(&challenge_type);
        ProposalTemplate {
            proposal_id: self.next_proposal_id().to_string(),
            internal_id: self.generator.id().to_string(),
            category_name: Industry().fake::<String>(),
            proposal_title: CatchPhase().fake::<String>(),
            proposal_summary: CatchPhase().fake::<String>(),

            proposal_funds: self.generator.proposal_fund().to_string(),
            proposal_url: proposal_url.to_string(),
            proposal_impact_score: self.generator.impact_score().to_string(),
            files_url: format!("{}/files", proposal_url),
            proposer_relevant_experience: self.generator.proposer().proposer_relevant_experience,
            chain_vote_options: "blank,yes,no".to_string(),
            proposer_name: Name().fake::<String>(),
            proposer_url: self.generator.gen_http_address(),
            chain_vote_type: "public".to_string(),
            challenge_id: None,
            challenge_type: proposal_challenge_info.challenge_type,
            proposal_metrics: proposal_challenge_info.proposal_metrics,
            proposal_solution: proposal_challenge_info.proposal_solution,
            proposal_brief: proposal_challenge_info.proposal_brief,
            proposal_importance: proposal_challenge_info.proposal_importance,
            proposal_goal: proposal_challenge_info.proposal_goal,
        }
    }

    fn next_challenge(&mut self) -> ChallengeTemplate {
        ChallengeTemplate {
            id: self.next_challenge_id().to_string(),
            challenge_type: self.generator.challenge_type(),
            title: CatchPhase().fake::<String>(),
            description: Buzzword().fake::<String>(),
            rewards_total: "0".to_string(),
            challenge_url: self.generator.gen_http_address(),
            fund_id: None,
        }
    }

    fn next_fund(&mut self) -> FundTemplate {
        FundTemplate {
            id: self.generator.id().abs(),
            goal: "How will we encourage developers and entrepreneurs to build Dapps and businesses on top of Cardano in the next 6 months?".to_string(),
            rewards_info: Sentence(3..5).fake::<String>(),
            threshold: None,
        }
    }
}

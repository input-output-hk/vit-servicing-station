use diesel::expression_methods::ExpressionMethods;
use diesel::query_dsl::RunQueryDsl;
use diesel::{Insertable, SqliteConnection};
use thiserror::Error;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::goals::InsertGoal;
use vit_servicing_station_lib::db::models::groups::Group;
use vit_servicing_station_lib::db::schema::goals;
use vit_servicing_station_lib::db::schema::{groups, proposals_voteplans};
use vit_servicing_station_lib::db::{
    models::{
        api_tokens::ApiTokenData,
        challenges::Challenge,
        funds::Fund,
        proposals::{FullProposalInfo, ProposalChallengeInfo},
    },
    schema::{
        api_tokens, challenges, community_advisors_reviews, funds,
        proposal_community_choice_challenge, proposal_simple_challenge, proposals, voteplans,
    },
};

pub struct DbInserter<'a> {
    connection: &'a SqliteConnection,
}

impl<'a> DbInserter<'a> {
    pub fn new(connection: &'a SqliteConnection) -> Self {
        Self { connection }
    }

    pub fn insert_token(&self, token_data: &ApiTokenData) -> Result<(), DbInserterError> {
        let values = (
            api_tokens::dsl::token.eq(token_data.token.as_ref()),
            api_tokens::dsl::creation_time.eq(token_data.creation_time),
            api_tokens::dsl::expire_time.eq(token_data.expire_time),
        );

        diesel::insert_into(api_tokens::table)
            .values(values)
            .execute(self.connection)
            .map_err(DbInserterError::DieselError)?;

        Ok(())
    }

    pub fn insert_tokens(&self, tokens_data: &[ApiTokenData]) -> Result<(), DbInserterError> {
        for token_data in tokens_data {
            self.insert_token(token_data)?;
        }
        Ok(())
    }

    pub fn insert_proposals(&self, proposals: &[FullProposalInfo]) -> Result<(), DbInserterError> {
        for proposal in proposals {
            let proposal_id = proposal.proposal.proposal_id.clone();
            let values = (
                proposals::id.eq(proposal.proposal.internal_id),
                proposals::proposal_id.eq(proposal.proposal.proposal_id.clone()),
                proposals::proposal_category.eq(proposal
                    .proposal
                    .proposal_category
                    .category_name
                    .clone()),
                proposals::proposal_title.eq(proposal.proposal.proposal_title.clone()),
                proposals::proposal_summary.eq(proposal.proposal.proposal_summary.clone()),
                proposals::proposal_public_key.eq(proposal.proposal.proposal_public_key.clone()),
                proposals::proposal_funds.eq(proposal.proposal.proposal_funds),
                proposals::proposal_url.eq(proposal.proposal.proposal_url.clone()),
                proposals::proposal_files_url.eq(proposal.proposal.proposal_files_url.clone()),
                proposals::proposer_name.eq(proposal.proposal.proposer.proposer_name.clone()),
                proposals::proposer_contact.eq(proposal.proposal.proposer.proposer_email.clone()),
                proposals::proposer_url.eq(proposal.proposal.proposer.proposer_url.clone()),
                proposals::proposal_impact_score.eq(proposal.proposal.proposal_impact_score),
                proposals::proposer_relevant_experience.eq(proposal
                    .proposal
                    .proposer
                    .proposer_relevant_experience
                    .clone()),
                proposals::chain_proposal_id.eq(proposal.proposal.chain_proposal_id.clone()),
                proposals::chain_vote_options
                    .eq(proposal.proposal.chain_vote_options.as_csv_string()),
                proposals::challenge_id.eq(proposal.proposal.challenge_id),
            );
            diesel::insert_into(proposals::table)
                .values(values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;

            let values = (
                proposals_voteplans::proposal_id.eq(proposal_id),
                proposals_voteplans::chain_proposal_index
                    .eq(proposal.voteplan.chain_proposal_index),
                proposals_voteplans::chain_voteplan_id
                    .eq(proposal.voteplan.chain_voteplan_id.clone()),
            );
            diesel::insert_into(proposals_voteplans::table)
                .values(values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;

            let voteplan_values = (
                voteplans::chain_voteplan_id.eq(proposal.voteplan.chain_voteplan_id.clone()),
                voteplans::chain_vote_start_time.eq(proposal.proposal.chain_vote_start_time),
                voteplans::chain_vote_end_time.eq(proposal.proposal.chain_vote_end_time),
                voteplans::chain_committee_end_time.eq(proposal.proposal.chain_committee_end_time),
                voteplans::chain_voteplan_payload
                    .eq(proposal.proposal.chain_voteplan_payload.clone()),
                voteplans::chain_vote_encryption_key
                    .eq(proposal.proposal.chain_vote_encryption_key.clone()),
                voteplans::fund_id.eq(proposal.proposal.fund_id),
                // TODO: hardcoded token, need to set this somewhere else.
                voteplans::token_identifier.eq("token"),
            );

            diesel::insert_or_ignore_into(voteplans::table)
                .values(voteplan_values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;

            match &proposal.challenge_info {
                ProposalChallengeInfo::Simple(data) => {
                    let simple_values = (
                        proposal_simple_challenge::proposal_id
                            .eq(proposal.proposal.proposal_id.clone()),
                        proposal_simple_challenge::proposal_solution
                            .eq(data.proposal_solution.clone()),
                    );
                    diesel::insert_into(proposal_simple_challenge::table)
                        .values(simple_values)
                        .execute(self.connection)
                        .map_err(DbInserterError::DieselError)?;
                }
                ProposalChallengeInfo::CommunityChoice(data) => {
                    let community_values = (
                        proposal_community_choice_challenge::proposal_id
                            .eq(proposal.proposal.proposal_id.clone()),
                        proposal_community_choice_challenge::proposal_brief
                            .eq(data.proposal_brief.clone()),
                        proposal_community_choice_challenge::proposal_importance
                            .eq(data.proposal_importance.clone()),
                        proposal_community_choice_challenge::proposal_goal
                            .eq(data.proposal_goal.clone()),
                        proposal_community_choice_challenge::proposal_metrics
                            .eq(data.proposal_metrics.clone()),
                    );
                    diesel::insert_into(proposal_community_choice_challenge::table)
                        .values(community_values)
                        .execute(self.connection)
                        .map_err(DbInserterError::DieselError)?;
                }
            };
        }
        Ok(())
    }

    pub fn insert_funds(&self, funds: &[Fund]) -> Result<(), DbInserterError> {
        for fund in funds {
            let values = fund.clone().values();

            diesel::insert_into(funds::table)
                .values(values)
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;

            for voteplan in &fund.chain_vote_plans {
                let values = (
                    voteplans::id.eq(voteplan.id),
                    voteplans::chain_voteplan_id.eq(voteplan.chain_voteplan_id.clone()),
                    voteplans::chain_vote_start_time.eq(voteplan.chain_vote_start_time),
                    voteplans::chain_vote_end_time.eq(voteplan.chain_vote_end_time),
                    voteplans::chain_committee_end_time.eq(voteplan.chain_committee_end_time),
                    voteplans::chain_voteplan_payload.eq(voteplan.chain_voteplan_payload.clone()),
                    voteplans::chain_vote_encryption_key
                        .eq(voteplan.chain_vote_encryption_key.clone()),
                    voteplans::fund_id.eq(voteplan.fund_id),
                    voteplans::token_identifier.eq(voteplan.token_identifier.clone()),
                );
                diesel::insert_or_ignore_into(voteplans::table)
                    .values(values)
                    .execute(self.connection)
                    .map_err(DbInserterError::DieselError)?;
            }

            for challenge in &fund.challenges {
                let values = (
                    challenges::id.eq(challenge.id),
                    challenges::challenge_type.eq(challenge.challenge_type.to_string()),
                    challenges::title.eq(challenge.title.clone()),
                    challenges::description.eq(challenge.description.clone()),
                    challenges::rewards_total.eq(challenge.rewards_total),
                    challenges::fund_id.eq(challenge.fund_id),
                    challenges::challenge_url.eq(challenge.challenge_url.clone()),
                );
                diesel::insert_or_ignore_into(challenges::table)
                    .values(values)
                    .execute(self.connection)
                    .map_err(DbInserterError::DieselError)?;
            }

            for goal in &fund.goals {
                diesel::insert_or_ignore_into(goals::table)
                    .values(InsertGoal::from(goal))
                    .execute(self.connection)
                    .map_err(DbInserterError::DieselError)?;
            }
        }
        Ok(())
    }

    pub fn insert_challenges(&self, challenges: &[Challenge]) -> Result<(), DbInserterError> {
        for challenge in challenges {
            diesel::insert_or_ignore_into(challenges::table)
                .values(challenge.clone().values())
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }

    pub fn insert_advisor_reviews(&self, reviews: &[AdvisorReview]) -> Result<(), DbInserterError> {
        for review in reviews {
            diesel::insert_or_ignore_into(community_advisors_reviews::table)
                .values(review.clone().values())
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }

    pub fn insert_groups(&self, groups: &[Group]) -> Result<(), DbInserterError> {
        for group in groups {
            diesel::insert_or_ignore_into(groups::table)
                .values(group.clone().values())
                .execute(self.connection)
                .map_err(DbInserterError::DieselError)?;
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum DbInserterError {
    #[error("internal diesel error")]
    DieselError(#[from] diesel::result::Error),
}

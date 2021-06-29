use structopt::StructOpt;

use crate::ideascale::fetch::{get_assessment_id, Scores};
use crate::ideascale::models::{Challenge, Fund, Funnel, Proposal};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::io;
use vit_servicing_station_lib::db::load_db_connection_pool;
use vit_servicing_station_lib::db::models::proposals::{Category, Proposer};
use vit_servicing_station_lib::db::models::vote_options::{VoteOptions, VoteOptionsMap};
use vit_servicing_station_lib::db::models::voteplans::Voteplan;

mod fetch;
mod models;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FetchError(#[from] fetch::Error),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Import {
    fund: usize,
    funnel_id: usize,
}

#[derive(Debug)]
struct IdeaScaleData {
    funnels: Vec<Funnel>,
    fund: Fund,
    challenges: HashMap<i32, Challenge>,
    proposals: HashMap<i32, Proposal>,
    scores: Scores,
}

#[derive(Debug, StructOpt)]
struct GovernanceParameters {
    fund_goal: String,
    voting_power_info: String,
    voting_power_threshold: i64,
    rewards_info: String,
    registration_snapshot_time: chrono::DateTime<Utc>,
    fund_start_time: chrono::DateTime<Utc>,
    fund_end_time: chrono::DateTime<Utc>,
    next_fund_start_time: chrono::DateTime<Utc>,
}

pub async fn fetch_all(fund: usize, api_token: String) -> Result<IdeaScaleData, Error> {
    let funnels_task = tokio::spawn(fetch::get_funnels_data_for_fund(fund, api_token.clone()));
    let funds_task = tokio::spawn(fetch::get_funds_data(api_token.clone()));
    let funnels = funnels_task.await??;
    let funds = funds_task.await??;
    let challenges: Vec<Challenge> = funds
        .iter()
        .flat_map(|f| f.challenges.iter().cloned())
        .collect();
    let proposals_tasks: Vec<_> = challenges
        .iter()
        .map(|c| tokio::spawn(fetch::get_proposals_data(c.id, api_token.clone())))
        .collect();
    let proposals: Vec<Proposal> = futures::future::try_join_all(proposals_tasks)
        .await?
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    let stage_ids: HashSet<i32> = proposals.iter().map(|p| p.stage_id).collect();

    let scores_tasks: Vec<_> = stage_ids
        .iter()
        .map(|id| {
            tokio::spawn(fetch::get_assessments_scores_by_stage_id(
                *id,
                api_token.clone(),
            ))
        })
        .collect();

    let scores: Scores = futures::future::try_join_all(scores_tasks)
        .await?
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    Ok(IdeaScaleData {
        funnels,
        fund: funds
            .into_iter()
            .filter(|f| f.name.contains(format!("Fund{}", fund)))
            .next()
            .expect(&format!(
                "Selected fund {}, wasn't among the available funds",
                fund
            )),
        challenges: challenges.into_iter().map(|c| (c.id, c)).collect(),
        proposals: proposals.into_iter().map(|p| (p.proposal_id, p)).collect(),
        scores,
    })
}

fn build_proposals_data(
    ideascale_data: &IdeaScaleData,
    scores: &Scores,
    voteplan: &Voteplan,
) -> Vec<vit_servicing_station_lib::db::models::proposals::Proposal> {
    ideascale_data
        .proposals
        .values()
        .map(
            |p| vit_servicing_station_lib::db::models::proposals::Proposal {
                internal_id: 0,
                proposal_id: p.proposal_id.to_string(),
                proposal_category: Category {
                    category_id: "".to_string(),
                    category_name: "".to_string(),
                    category_description: "".to_string(),
                },
                proposal_title: p.proposal_title.clone(),
                proposal_summary: p.proposal_summary.clone(),
                proposal_public_key: p.custom_fields.proposal_public_key.clone(),
                proposal_funds: p.custom_fields.proposal_funds.parse().unwrap(),
                proposal_url: p.proposal_url.clone(),
                proposal_files_url: "".to_string(),
                proposal_impact_score: scores
                    .get(&p.proposal_id)
                    .expect(&format!(
                        "Impact score not found for proposal with id {}",
                        p.proposal_id
                    ))
                    .round() as i64,
                proposer: Proposer {
                    proposer_name: p.proposer.name.clone(),
                    proposer_email: p.proposer.contact.clone(),
                    proposer_url: "".to_string(),
                    proposer_relevant_experience: "".to_string(),
                },
                chain_proposal_id: vec![],
                chain_proposal_index: 0,
                chain_vote_options: VoteOptions(
                    [("yes".to_string(), 0), ("no".to_string(), 1)]
                        .iter()
                        .collect(),
                ),
                chain_voteplan_id: voteplan.chain_voteplan_id.clone(),
                chain_vote_start_time: voteplan.chain_vote_start_time,
                chain_vote_end_time: voteplan.chain_vote_end_time,
                chain_committee_end_time: voteplan.chain_committee_end_time,
                chain_voteplan_payload: voteplan.chain_voteplan_payload.clone(),
                chain_vote_encryption_key: voteplan.chain_vote_encryption_key.clone(),
                fund_id: ideascale_data.fund.id,
                challenge_id: 0,
            },
        )
        .collect()
}

fn push_to_db(
    ideascale_data: &IdeaScaleData,
    voteplans: HashMap<i32, Voteplan>,
    governance_parameters: &GovernanceParameters,
) -> Result<(), Error> {
    let voteplan_data = voteplans
        .values()
        .next()
        .expect("Voteplans should't be empty");

    let voteplans: Vec<Voteplan> = voteplans.values().collect();

    // build fund data
    let fund = vit_servicing_station_lib::db::models::funds::Fund {
        id: 0,
        fund_name: ideascale_data.fund.name.clone(),
        fund_goal: governance_parameters.fund_goal.clone(),
        voting_power_threshold: governance_parameters.voting_power_threshold,
        rewards_info: governance_parameters.rewards_info.clone(),
        fund_start_time: governance_parameters.fund_start_time.timestamp(),
        fund_end_time: governance_parameters.fund_end_time.timestamp(),
        next_fund_start_time: governance_parameters.next_fund_start_time.timestamp(),
        registration_snapshot_time: governance_parameters.registration_snapshot_time.timestamp(),
        chain_vote_plans: voteplans.clone(),
        challenges: ideascale_data.challenges.values().collect(),
    };

    // start db connection
    let pool = load_db_connection_pool(db_url)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))?;
    let db_conn = pool
        .get()
        .map_err(|e| io::Error::new(io::ErrorKind::NotConnected, format!("{}", e)))?;

    // upload fund to db
    vit_servicing_station_lib::db::queries::funds::insert_fund(fund, &db_conn)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

    // upload voteplans
    vit_servicing_station_lib::db::queries::voteplans::batch_insert_voteplans(&voteplans, &db_conn)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ideascale::fetch_all;

    const API_TOKEN: &str = "";

    #[tokio::test]
    async fn test_fetch_funds() {
        let results = fetch_all(4, API_TOKEN.to_string())
            .await
            .expect("All current campaigns data");

        println!("{:?}", results);
    }
}
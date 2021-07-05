mod fetch;
mod models;

use crate::ideascale::fetch::Scores;
use crate::ideascale::models::{Challenge, Fund, Funnel, Proposal};
use crate::task::ExecTask;

use chain_impl_mockchain::certificate::VotePlan;
use chain_impl_mockchain::vote::PayloadType;
use jormungandr_lib::interfaces::VotePlanDef;
use vit_servicing_station_lib::db::load_db_connection_pool;
use vit_servicing_station_lib::db::models as db_models;
use vit_servicing_station_lib::db::models::proposals::{community_choice, simple};
use vit_servicing_station_lib::db::models::proposals::{Category, ChallengeType, Proposer};
use vit_servicing_station_lib::db::models::vote_options::VoteOptions;

use chrono::{DateTime, Utc};
use structopt::StructOpt;

use std::collections::{HashMap, HashSet};
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};

// TODO: set error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FetchError(#[from] fetch::Error),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),

    #[error(transparent)]
    QueryError(#[from] diesel::result::Error),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Import {
    #[structopt(long)]
    fund: usize,

    #[structopt(long)]
    api_token: String,

    #[structopt(long)]
    voteplans: PathBuf,

    #[structopt(long)]
    rewards: PathBuf,

    #[structopt(long)]
    db_url: String,

    #[structopt(flatten)]
    governance_parameters: GovernanceParameters,

    #[structopt(flatten)]
    voting_parameters: VotingParameters,
}

#[derive(Debug)]
struct IdeaScaleData {
    funnels: HashMap<i32, Funnel>,
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

#[derive(Debug, StructOpt)]
struct VotingParameters {
    vote_start_time: DateTime<Utc>,
    vote_end_time: DateTime<Utc>,
    vote_committee_time: DateTime<Utc>,
    chain_vote_encryption_key: String,
}

pub struct DbData {
    voteplans: Vec<db_models::voteplans::Voteplan>,
    challenges: Vec<db_models::challenges::Challenge>,
    fund: db_models::funds::Fund,
    proposals: Vec<db_models::proposals::Proposal>,
    simple_proposal_data: Vec<simple::ChallengeSqlValues>,
    community_proposal_data: Vec<community_choice::ChallengeSqlValues>,
}

pub type Rewards = HashMap<i32, i64>;

async fn fetch_all(fund: usize, api_token: String) -> Result<IdeaScaleData, Error> {
    let funnels_task = tokio::spawn(fetch::get_funnels_data_for_fund(fund, api_token.clone()));
    let funds_task = tokio::spawn(fetch::get_funds_data(api_token.clone()));
    let funnels = funnels_task
        .await??
        .into_iter()
        .map(|f| (f.id, f))
        .collect();
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
            .find(|f| f.name.contains(&format!("Fund{}", fund)))
            .unwrap_or_else(|| panic!("Selected fund {}, wasn't among the available funds", fund)),
        challenges: challenges.into_iter().map(|c| (c.id, c)).collect(),
        proposals: proposals.into_iter().map(|p| (p.proposal_id, p)).collect(),
        scores,
    })
}

fn build_proposals_data(
    ideascale_data: &IdeaScaleData,
    voting_paramters: &VotingParameters,
    voteplan: &VotePlan,
) -> Vec<vit_servicing_station_lib::db::models::proposals::Proposal> {
    let challenges = &ideascale_data.challenges;
    ideascale_data
        .proposals
        .values()
        .map(|p| {
            let challenge = challenges.get(&p.challenge_id).unwrap();
            vit_servicing_station_lib::db::models::proposals::Proposal {
                internal_id: 0,
                proposal_id: p.proposal_id.to_string(),
                proposal_category: Category {
                    category_id: challenge.id.to_string(),
                    category_name: challenge.title.clone(),
                    category_description: challenge.description.clone(),
                },
                proposal_title: p.proposal_title.clone(),
                proposal_summary: p.proposal_summary.clone(),
                proposal_public_key: p.custom_fields.proposal_public_key.clone(),
                proposal_funds: p.custom_fields.proposal_funds.parse().unwrap(),
                proposal_url: p.proposal_url.clone(),
                proposal_files_url: "".to_string(),
                proposal_impact_score: ideascale_data
                    .scores
                    .get(&p.proposal_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "Impact score not found for proposal with id {}",
                            p.proposal_id
                        )
                    })
                    .round() as i64,
                proposer: Proposer {
                    proposer_name: p.proposer.name.clone(),
                    proposer_email: p.proposer.contact.clone(),
                    proposer_url: "".to_string(),
                    proposer_relevant_experience: "".to_string(),
                },
                // TODO: where to get chain proposal id?
                chain_proposal_id: vec![],
                chain_proposal_index: 0,
                chain_vote_options: VoteOptions(
                    [
                        ("blank".to_string(), 0u8),
                        ("yes".to_string(), 1u8),
                        ("no".to_string(), 2u8),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                ),
                chain_voteplan_id: voteplan.to_id().to_string(),
                chain_vote_start_time: voting_paramters.vote_start_time.timestamp(),
                chain_vote_end_time: voting_paramters.vote_end_time.timestamp(),
                chain_committee_end_time: voting_paramters.vote_committee_time.timestamp(),
                chain_voteplan_payload: payload_type_to_string(voteplan.payload_type()),
                chain_vote_encryption_key: voting_paramters.chain_vote_encryption_key.clone(),
                fund_id: ideascale_data.fund.id,
                challenge_id: p.challenge_id,
            }
        })
        .collect()
}

fn build_extra_proposals_data(
    ideascale_data: &IdeaScaleData,
) -> Result<
    (
        Vec<simple::ChallengeSqlValues>,
        Vec<community_choice::ChallengeSqlValues>,
    ),
    Error,
> {
    let funnels = &ideascale_data.funnels;
    let challenges = &ideascale_data.challenges;

    let challenge_type_by_challenge_id = |id: i32| -> ChallengeType {
        let funnel_id = challenges
            .get(&id)
            .unwrap_or_else(|| panic!("Couldn't find a challenge with id: {}", id))
            .funnel_id;
        if funnels
            .get(&funnel_id)
            .unwrap_or_else(|| panic!("Couldn't find a funnel with id: {}", funnel_id))
            .is_community()
        {
            ChallengeType::CommunityChoice
        } else {
            ChallengeType::Simple
        }
    };

    let (mut simple, mut community) = (vec![], vec![]);
    for proposal in ideascale_data.proposals.values() {
        match challenge_type_by_challenge_id(proposal.challenge_id) {
            ChallengeType::Simple => {
                let challenge_info: simple::ChallengeInfo =
                    serde_json::from_value(proposal.custom_fields.extra.clone())?;
                simple.push(
                    challenge_info
                        .to_sql_values_with_proposal_id(&proposal.proposal_id.to_string()),
                );
            }
            ChallengeType::CommunityChoice => {
                let challenge_info: community_choice::ChallengeInfo =
                    serde_json::from_value(proposal.custom_fields.extra.clone())?;
                community.push(
                    challenge_info
                        .to_sql_values_with_proposal_id(&proposal.proposal_id.to_string()),
                );
            }
        }
    }
    Ok((simple, community))
}

fn build_db_data(
    ideascale_data: &IdeaScaleData,
    voteplans: &HashMap<String, VotePlan>,
    governance_parameters: &GovernanceParameters,
    voting_paramters: &VotingParameters,
    rewards: &Rewards,
) -> Result<DbData, Error> {
    let voteplan_data = voteplans
        .values()
        .next()
        .expect("Voteplans should't be empty");

    let voteplans: Vec<db_models::voteplans::Voteplan> = voteplans
        .values()
        .map(|v| db_models::voteplans::Voteplan {
            id: 0,
            chain_voteplan_id: v.to_id().to_string(),
            chain_vote_start_time: voting_paramters.vote_start_time.timestamp(),
            chain_vote_end_time: voting_paramters.vote_end_time.timestamp(),
            chain_committee_end_time: voting_paramters.vote_committee_time.timestamp(),
            chain_voteplan_payload: payload_type_to_string(voteplan_data.payload_type()),
            chain_vote_encryption_key: voting_paramters.chain_vote_encryption_key.clone(),
            fund_id: ideascale_data.fund.id,
        })
        .collect();
    let challenges: Vec<_> = ideascale_data
        .challenges
        .values()
        .map(|c| db_models::challenges::Challenge {
            id: 0,
            challenge_type: ChallengeType::Simple,
            title: c.title.clone(),
            description: c.description.clone(),
            rewards_total: *rewards
                .get(&c.id)
                .unwrap_or_else(|| panic!("Rewards not found for challenge with id: {}", c.id)),
            proposers_rewards: 0,
            fund_id: ideascale_data.fund.id,
            challenge_url: c.challenge_url.clone(),
        })
        .collect();
    // build fund data
    let fund = db_models::funds::Fund {
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
        challenges: challenges.clone(),
    };

    let proposals = build_proposals_data(&ideascale_data, &voting_paramters, &voteplan_data);
    let (simple_data, community_data) = build_extra_proposals_data(&ideascale_data)?;
    Ok(DbData {
        voteplans,
        challenges,
        fund,
        proposals,
        simple_proposal_data: simple_data,
        community_proposal_data: community_data,
    })
}

fn push_to_db(db_data: DbData, db_url: &str) -> Result<(), Error> {
    let DbData {
        voteplans,
        challenges,
        fund,
        proposals,
        simple_proposal_data,
        community_proposal_data,
    } = db_data;

    // start db connection
    let pool = load_db_connection_pool(db_url)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))?;

    let db_conn = pool
        .get()
        .map_err(|e| io::Error::new(io::ErrorKind::NotConnected, format!("{}", e)))?;

    // upload fund to db
    vit_servicing_station_lib::db::queries::funds::insert_fund(fund, &db_conn)?;

    // upload voteplans
    vit_servicing_station_lib::db::queries::voteplans::batch_insert_voteplans(
        &voteplans, &db_conn,
    )?;

    // upload proposals
    vit_servicing_station_lib::db::queries::proposals::batch_insert_proposals(
        &proposals, &db_conn,
    )?;

    // upload challenges
    vit_servicing_station_lib::db::queries::challenges::batch_insert_challenges(
        &challenges,
        &db_conn,
    )?;

    vit_servicing_station_lib::db::queries::proposals::batch_insert_community_choice_challenge_data(
        &community_proposal_data,
        &db_conn,
    )?;

    vit_servicing_station_lib::db::queries::proposals::batch_insert_simple_challenge_data(
        &simple_proposal_data,
        &db_conn,
    )?;

    Ok(())
}

fn payload_type_to_string(payload_type: PayloadType) -> String {
    match payload_type {
        PayloadType::Public => "Public",
        PayloadType::Private => "Private",
    }
    .to_string()
}

fn load_json_from_file_path<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let file = std::fs::File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

impl ExecTask for Import {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<Self::ResultValue> {
        let Import {
            fund,
            voteplans,
            rewards,
            db_url,
            governance_parameters,
            voting_parameters,
            api_token,
        } = self;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

        let idescale_data =
            futures::executor::block_on(runtime.spawn(fetch_all(*fund, api_token.clone())))?
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

        let voteplans: HashMap<String, VotePlan> =
            load_json_from_file_path::<Vec<VotePlanDef>>(voteplans)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?
                .into_iter()
                .map(Into::into)
                .map(|v: VotePlan| (v.to_id().to_string(), v))
                .collect();

        let rewards: Rewards = load_json_from_file_path(rewards)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

        let db_data = build_db_data(
            &idescale_data,
            &voteplans,
            governance_parameters,
            &voting_parameters,
            &rewards,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

        push_to_db(db_data, &db_url)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

        Ok(())
    }
}

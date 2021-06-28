use structopt::StructOpt;

use crate::ideascale::fetch::get_assessment_id;
use crate::ideascale::models::{Challenge, Proposal};

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

pub async fn generate_db(fund: usize, funnel_id: usize, api_token: String) -> Result<(), Error> {
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
    // let assestment_id_task = get_assessment_id()

    Ok(())
}

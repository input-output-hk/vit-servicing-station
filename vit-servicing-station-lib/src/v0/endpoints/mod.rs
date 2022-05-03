mod advisor_reviews;
mod challenges;
mod funds;
mod genesis;
mod health;
pub mod proposals;
pub mod service_version;
mod snapshot;

use crate::v0::context::SharedContext;

use crate::v0::api_token;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
    snapshot_rx: snapshot_service::SharedContext,
    snapshot_tx: snapshot_service::UpdateHandle,
    enable_api_tokens: bool,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // mount health endpoint
    let health_root = warp::path!("health" / ..);
    let health_filter = health::filter(health_root.boxed(), context.clone()).await;

    // mount chain-data endpoint
    let chain_data_root = warp::path!("proposals" / ..);
    let chain_data_filter = proposals::filter(chain_data_root.boxed(), context.clone()).await;

    // mount funds endpoint
    let funds_root = warp::path!("fund" / ..);
    let funds_filter = funds::filter(funds_root.boxed(), context.clone()).await;

    // mount challenges endpoint
    let challenges_root = warp::path!("challenges" / ..);
    let challenges_filter = challenges::filter(challenges_root.boxed(), context.clone()).await;

    // mount genesis endpoint
    let genesis_root = warp::path!("block0" / ..);
    let genesis_filter = genesis::filter(genesis_root.boxed(), context.clone());

    let reviews_root = warp::path!("reviews" / ..);
    let reviews_filter = advisor_reviews::filter(reviews_root.boxed(), context.clone()).await;

    let snapshot_root = warp::path!("snapshot" / ..);
    let snapshot_rx_filter = snapshot_service::filter(snapshot_root.boxed(), snapshot_rx.clone());
    let snapshot_tx_filter = snapshot_service::update_filter(snapshot_root.boxed(), snapshot_tx);

    let api_token_filter = if enable_api_tokens {
        api_token::api_token_filter(context).await.boxed()
    } else {
        warp::any().boxed()
    };

    root.and(
        api_token_filter.and(
            health_filter
                .or(genesis_filter)
                .or(chain_data_filter)
                .or(funds_filter)
                .or(challenges_filter)
                .or(reviews_filter)
                .or(snapshot_rx_filter)
                .or(snapshot_tx_filter),
        ),
    )
    .boxed()
}

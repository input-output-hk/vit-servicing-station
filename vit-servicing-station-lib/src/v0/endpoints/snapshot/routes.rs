use super::handlers::get_voting_power;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let snapshot = warp::path!(String)
        .and(warp::get())
        .and(with_context)
        .and_then(get_voting_power)
        .boxed();

    root.and(snapshot).boxed()
}

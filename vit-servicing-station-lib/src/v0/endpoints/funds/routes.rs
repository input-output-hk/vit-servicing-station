use super::handlers::*;
use super::logic::get_next_fund;
use crate::v0::context::SharedContext;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let fund = warp::path::end()
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_fund);

    let fund_by_id = warp::path!(i32)
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_fund_by_id);

    let next_fund = warp::path!("path")
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_next_fund);

    let all_funds = warp::path::end()
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_all_funds);

    // fund_by_id need to be checked first otherwise requests are swallowed by the fund::any
    root.and(fund_by_id.or(fund).or(all_funds))
}

pub fn admin_filter(
    context: SharedContext,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    warp::path::end()
        .and(warp::put())
        .and(warp::body::json())
        .and(with_context)
        .and_then(put_fund)
}

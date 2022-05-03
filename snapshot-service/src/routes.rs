use std::sync::Arc;

use crate::handlers::put_tag;
use crate::{SharedContext, UpdateHandle};

use super::handlers::{get_tags, get_voting_power};
use tokio::sync::Mutex;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let get_voting_power = warp::path!(String / String)
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_voting_power)
        .boxed();

    let get_tags = warp::path::end()
        .and(warp::get())
        .and(with_context)
        .and_then(get_tags)
        .boxed();

    root.and(get_voting_power.or(get_tags)).boxed()
}

pub fn update_filter(
    root: BoxedFilter<()>,
    context: UpdateHandle,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let ctx = Arc::new(Mutex::new(context));
    let with_context = warp::any().map(move || Arc::clone(&ctx));

    let put = warp::path!(String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_context)
        .and_then(put_tag)
        .boxed();

    root.and(put).boxed()
}

use crate::v0::context::SharedContext;

use super::handlers::{
    get_tags, get_users_info, get_voters_info, put_raw_snapshot, put_snapshot_info,
};
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

pub fn filter(
    root: BoxedFilter<()>,
    context: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let get_voters_info = warp::path!("voter" / String / String)
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_voters_info);

    let get_users_info = warp::path!("user" / String / String)
        .and(warp::get())
        .and(with_context.clone())
        .and_then(get_users_info);

    let get_tags = warp::path::end()
        .and(warp::get())
        .and(with_context)
        .and_then(get_tags);

    root.and(get_voters_info.or(get_users_info).or(get_tags))
}

pub fn update_filter(
    context: SharedContext,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    let snapshot_info = warp::path!("snapshot_info" / String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_context.clone())
        .and_then(put_snapshot_info);

    let raw_snapshot = warp::path!("raw_snapshot" / String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_context)
        .and_then(put_raw_snapshot);

    snapshot_info.or(raw_snapshot)
}

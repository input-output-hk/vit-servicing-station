use diesel::{
    expression_methods::ExpressionMethods,
    r2d2::{ConnectionManager, PooledConnection},
    QueryDsl, RunQueryDsl, TextExpressionMethods,
};

use crate::{
    db::{DbConnection, DbConnectionPool},
    v0::{
        endpoints::search::requests::{
            Column, Constraint, OrderBy, SearchCountQuery, SearchQuery, SearchResponse, Table,
        },
        errors::HandleError,
    },
};

pub async fn search_db(
    query: SearchQuery,
    pool: &DbConnectionPool,
) -> Result<SearchResponse, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || search(query, &db_conn))
        .await
        .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn search_count_db(
    query: SearchCountQuery,
    pool: &DbConnectionPool,
) -> Result<i64, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || search_count(query, &db_conn))
        .await
        .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

fn search(
    SearchQuery {
        query,
        limit,
        offset,
    }: SearchQuery,
    conn: &PooledConnection<ConnectionManager<DbConnection>>,
) -> Result<SearchResponse, HandleError> {
    let SearchCountQuery {
        table,
        filter,
        order_by,
    } = query;

    match table {
        Table::Challenges => {
            use crate::db::schema::challenges::dsl::*;
            use Column::*;

            let mut query = challenges.into_boxed();

            for Constraint { search, column } in filter {
                let search = format!("%{search}%");
                query = match column {
                    Title => query.filter(title.like(search)),
                    Desc => query.filter(description.like(search)),
                    Type => query.filter(challenge_type.like(search)),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }

            for OrderBy { column, descending } in order_by {
                query = match (descending, column) {
                    (false, Title) => query.then_order_by(title),
                    (false, Desc) => query.then_order_by(description),
                    (false, Type) => query.then_order_by(challenge_type),
                    (true, Title) => query.then_order_by(title.desc()),
                    (true, Desc) => query.then_order_by(description.desc()),
                    (true, Type) => query.then_order_by(challenge_type.desc()),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }

            if let Some(limit) = limit {
                query = query.limit(limit)
            }

            if let Some(offset) = offset {
                query = query.offset(offset)
            }

            let vec = query
                .load(conn)
                .map_err(|_| HandleError::InternalError("error searching".to_string()))?;
            Ok(SearchResponse::Challenge(vec))
        }
        Table::Proposals => {
            use crate::db::views_schema::full_proposals_info::dsl::*;
            use full_proposals_info as proposals;
            use Column::*;

            let mut query = proposals.into_boxed();

            for Constraint { search, column } in filter {
                let search = format!("%{search}%");
                query = match column {
                    Title => query.filter(proposal_title.like(search)),
                    Desc => query.filter(proposal_summary.like(search)),
                    Author => query.filter(proposer_name.like(search)),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }

            for OrderBy { column, descending } in order_by {
                query = match (descending, column) {
                    (false, Title) => query.then_order_by(proposal_title),
                    (false, Desc) => query.then_order_by(proposal_summary),
                    (false, Author) => query.then_order_by(proposer_name),
                    (false, Funds) => query.then_order_by(proposal_funds),
                    (true, Title) => query.then_order_by(proposal_title.desc()),
                    (true, Desc) => query.then_order_by(proposal_summary.desc()),
                    (true, Author) => query.then_order_by(proposer_name.desc()),
                    (true, Funds) => query.then_order_by(proposal_funds.desc()),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }

            if let Some(limit) = limit {
                query = query.limit(limit)
            }

            if let Some(offset) = offset {
                query = query.offset(offset)
            }

            let vec = query
                .load(conn)
                .map_err(|_| HandleError::InternalError("error searching".to_string()))?;
            Ok(SearchResponse::Proposal(vec))
        }
    }
}

fn search_count(
    SearchCountQuery {
        table,
        filter,
        order_by,
    }: SearchCountQuery,
    conn: &PooledConnection<ConnectionManager<DbConnection>>,
) -> Result<i64, HandleError> {
    match table {
        Table::Challenges => {
            use crate::db::schema::challenges::dsl::*;
            use Column::*;

            let mut query = challenges.into_boxed();

            for Constraint { search, column } in filter {
                let search = format!("%{search}%");
                query = match column {
                    Title => query.filter(title.like(search)),
                    Desc => query.filter(description.like(search)),
                    Type => query.filter(challenge_type.like(search)),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }

            for OrderBy { column, descending } in order_by {
                query = match (descending, column) {
                    (false, Title) => query.then_order_by(title),
                    (false, Desc) => query.then_order_by(description),
                    (false, Type) => query.then_order_by(challenge_type),
                    (true, Title) => query.then_order_by(title.desc()),
                    (true, Desc) => query.then_order_by(description.desc()),
                    (true, Type) => query.then_order_by(challenge_type.desc()),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }

            let count = query
                .count()
                .get_result(conn)
                .map_err(|_| HandleError::InternalError("error searching".to_string()))?;
            Ok(count)
        }
        Table::Proposals => {
            use crate::db::views_schema::full_proposals_info::dsl::*;
            use full_proposals_info as proposals;
            use Column::*;

            let mut query = proposals.into_boxed();

            for Constraint { search, column } in filter {
                let search = format!("%{search}%");
                query = match column {
                    Title => query.filter(proposal_title.like(search)),
                    Desc => query.filter(proposal_summary.like(search)),
                    Author => query.filter(proposer_name.like(search)),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }

            for OrderBy { column, descending } in order_by {
                query = match (descending, column) {
                    (false, Title) => query.then_order_by(proposal_title),
                    (false, Desc) => query.then_order_by(proposal_summary),
                    (false, Author) => query.then_order_by(proposer_name),
                    (false, Funds) => query.then_order_by(proposal_funds),
                    (true, Title) => query.then_order_by(proposal_title.desc()),
                    (true, Desc) => query.then_order_by(proposal_summary.desc()),
                    (true, Author) => query.then_order_by(proposer_name.desc()),
                    (true, Funds) => query.then_order_by(proposal_funds.desc()),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }

            let count = query
                .count()
                .get_result(conn)
                .map_err(|_| HandleError::InternalError("error searching".to_string()))?;
            Ok(count)
        }
    }
}

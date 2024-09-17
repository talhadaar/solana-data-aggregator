use crate::storage::Database;
use crate::traits::Storage;
use crate::types::Address;
use serde::Deserialize;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use warp;
use warp::Filter;

#[derive(Deserialize)]
pub struct ApiParam {
    pub address: Address,
}

/// Gets all transactions associated witn an account
async fn get_transactions(
    params: ApiParam,
    storage_interface: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::debug!("Get transactions for address: {:?}", params.address);
    match storage_interface.get_transactions(params.address).await {
        Ok(transactions) => Ok(warp::reply::json(&transactions)),
        Err(error) => Ok(warp::reply::json(&error.to_string())),
    }
}

/// Gets all info stored in an account
/// For now, returns account balance only.
async fn get_account(
    params: ApiParam,
    storage_interface: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::debug!("Get account for address: {:?}", params.address);
    match storage_interface.get_account(&params.address).await {
        Ok(accounts) => Ok(warp::reply::json(&accounts)),
        Err(error) => Ok(warp::reply::json(&error.to_string())),
    }
}

pub async fn run_api(address: SocketAddr, db: Database, token: CancellationToken) {
    let db_move = db.clone();
    let get_transactions_route = warp::path!("transactions")
        .and(warp::query::<ApiParam>())
        .and(warp::any().map(move || db_move.clone()))
        .and_then(get_transactions);

    let db_move = db.clone();
    let get_accounts_route = warp::path!("account")
        .and(warp::query::<ApiParam>())
        .and(warp::any().map(move || db_move.clone()))
        .and_then(get_account);

    let routes = get_accounts_route.or(get_transactions_route);
    let (addr, fut) = warp::serve(routes).bind_with_graceful_shutdown(address, async move {
        token.cancelled().await;
        log::info!("Shutting down API server");
    });
    log::debug!("API started at: {}", addr);
    fut.await;
}

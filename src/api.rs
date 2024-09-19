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
    match storage_interface.get_transactions(&params.address).await {
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

/// Starts the API server on the provided socket address
/// The server will run until the token is cancelled
/// The server will provide two routes:
/// - /transactions?address=<address> - returns all transactions associated with the address
/// - /account?address=<address> - returns all info stored in the account
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

#[cfg(test)]
mod api_tests {
    use super::*;
    use crate::storage::Database;
    use crate::types::*;
    use rand::Rng;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use tokio_test::assert_ok;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn sanity_check() {
        let path = "/tmp/api.json";
        let mut db = Database::new(path).unwrap();
        let token = CancellationToken::new();
        let socket = SocketAddr::from_str("127.0.0.1:8080").unwrap();

        // spawn API
        let db_move = db.clone();
        let token_move = token.clone();
        let api_fut = tokio::spawn(async move { run_api(socket, db_move, token_move).await });

        // add a block to the database
        let mut rng = rand::thread_rng();
        let block_height: u64 = rng.gen();
        let source = String::from(format!("source{}", block_height));
        let destination = String::from(format!("destination{}", block_height));
        let block_hash = String::from(format!("block_hash{}", block_height));

        let amount = 100;
        let transaction = Transaction {
            source: source.to_string(),
            destination: destination.to_string(),
            amount,
        };

        let block = Block {
            height: block_height,
            transactions: vec![transaction],
            hash: block_hash.to_string(),
            timestamp: 100100,
        };
        db.add_block(&block).await.unwrap();

        // query the API

        // TODO request has a dependency conflict with solana dependencies
        // let request_url = format!("http://127.0.0.1:8080/account/?address={}", source);
        // let response = reqwest::get(&request_url).await?;
        // let account: Account = response.json().await?;
        // assert_eq!(account, amount);

        // let request_url = format!("http://127.0.0.1:8080/transactions/?address={}", source);
        // let response = reqwest::get(&request_url).await?;
        // let transactions: Vec<Transaction> = response.json().await?;
        // assert_eq!(transactions.len(), 1);
        // assert_eq!(transactions[0].amount, amount);
        // assert_eq!(transactions[0].source, source);
        // assert_eq!(transactions[0].destination, destination);

        // cancel the token
        token.cancel();

        // check the API server is down
        let result = tokio::join!(api_fut).0;
        assert_ok!(result);
    }
}

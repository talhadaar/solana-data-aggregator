extern crate dotenv;
use dotenv::dotenv;

use eyre::Result;
use solana_client::rpc_config::RpcBlockConfig;
use solana_data_aggregator::{fetcher::Fetcher, monitor::SlotMonitor, storage::StorageInterface};
use solana_transaction_status::UiTransactionEncoding;
use tokio::signal::ctrl_c;
use tokio_util::sync::CancellationToken;
#[tokio::main]
async fn main() -> Result<()> {
    // config env
    dotenv().ok();

    let token = CancellationToken::new();
    let provider_rpc = std::env::var("PROVIDER_RPC_URL")?;
    let provider_ws = std::env::var("PROVIDER_WS_URL")?;
    let db_path = std::env::var("DB_PATH")?;

    // create and start slot monitor
    let monitor = SlotMonitor::new(provider_ws.as_str(), token.clone()).await?;
    let monitor_fut = tokio::spawn(async move { monitor.start_monitoring().await });

    // create storage instance
    let storage_interface = StorageInterface::new(&db_path);

    // fetcher
    let block_config = RpcBlockConfig {
        max_supported_transaction_version: Some(0),
        encoding: Some(UiTransactionEncoding::JsonParsed),
        ..RpcBlockConfig::default()
    };
    let fetcher = Fetcher::new(provider_rpc, block_config);

    // create aggregator

    // graceful shutdown monitor
    let shutdown_fut = tokio::spawn(async move {
        ctrl_c().await.unwrap();
        token.cancel();
    });

    tokio::join!(monitor_fut, shutdown_fut);
    Ok(())
}

extern crate dotenv;
use dotenv::dotenv;

use solana_aggregator::{aggregator, error::Result, monitor, storage, streamer};
use solana_client::rpc_config::RpcBlockConfig;
use solana_transaction_status::UiTransactionEncoding;
use tokio::{signal::ctrl_c, sync::mpsc};
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
    let (monitor_tx, monitor_rx) = mpsc::unbounded_channel();
    let monitor_token = token.clone();
    let monitor =
        monitor::SlotMonitor::new(provider_ws.as_str(), monitor_token, monitor_tx).await?;
    let monitor_fut = tokio::spawn(async move { monitor.start_monitoring().await });

    // create storage instance
    let storage = storage::Database::new(&db_path);

    // create streamer
    let block_config = RpcBlockConfig {
        max_supported_transaction_version: Some(0),
        encoding: Some(UiTransactionEncoding::JsonParsed),
        ..RpcBlockConfig::default()
    };
    let streamer_token = token.clone();
    let streamer =
        streamer::Streamer::new(provider_rpc, streamer_token, monitor_rx, block_config).await?;

    // start aggregator
    let aggregator_token = token.clone();
    let mut aggregator = aggregator::Aggregator::new(streamer, aggregator_token, storage);
    let aggregator_fut = tokio::spawn(async move { aggregator.run().await });

    // graceful shutdown monitor
    let shutdown_token = token.clone();
    let shutdown_fut = tokio::spawn(async move {
        ctrl_c().await.unwrap();
        shutdown_token.cancel();
    });

    tokio::join!(monitor_fut, aggregator_fut, shutdown_fut);
    // {
    //     (Err(monitor), Err(aggregator), Err(shutdown)) => {
    //         log::error!("Join Error: Monitor {}, Aggregator {}, Shutdown {}", monitor, aggregator, shutdown);
    //         token.cancel();
    //     }
    //     (_, _, _) => {}
    // }

    Ok(())
}

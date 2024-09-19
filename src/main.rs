use clap::Parser;
use solana_client::rpc_config::RpcBlockConfig;
use solana_data_aggregator::{aggregator, error::Result, monitor, storage, streamer};
use solana_transaction_status::UiTransactionEncoding;
use std::net::SocketAddr;
use tokio::{signal::ctrl_c, sync::mpsc};
use tokio_util::sync::CancellationToken;

/// Solana Data Aggregator CLI
/// Should use .env file instead of command line arguments
/// As API Keys and other sensitive information should not be exposed
#[derive(Parser)]
#[command(version, about, long_about = "Solana Data Aggregator")]
struct Args {
    /// Socket for our REST API
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    socket: SocketAddr,

    /// RPC provider URL
    #[arg(short, long, default_value = None)]
    rpc_provider: String,

    /// WSS Provider URL
    #[arg(short, long, default_value = None)]
    wss_provider: String,

    /// Path for our JSON DB file e.g. /tmp/solana_data_aggregator.json
    #[arg(short, long, default_value = None)]
    db_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // start logger
    env_logger::init();

    let token = CancellationToken::new();
    let args = Args::parse();

    // create and start slot monitor
    let (monitor_tx, monitor_rx) = mpsc::unbounded_channel();
    let monitor_token = token.clone();
    let monitor = monitor::SlotMonitor::new(&args.wss_provider, monitor_token, monitor_tx).await?;
    let monitor_fut = tokio::spawn(async move { monitor.start_monitoring().await });
    log::debug!("Slot monitor started");

    // create storage instance
    let storage = storage::Database::new(&args.db_path)?;
    log::debug!("Storage initialized");

    // create streamer
    let block_config = RpcBlockConfig {
        max_supported_transaction_version: Some(0),
        encoding: Some(UiTransactionEncoding::JsonParsed),
        ..RpcBlockConfig::default()
    };
    let streamer_token = token.clone();
    let streamer =
        streamer::Streamer::new(&args.rpc_provider, streamer_token, monitor_rx, block_config)
            .await?;
    log::debug!("Streamer initialized");

    // start aggregator
    let aggregator_token = token.clone();
    let storage_clone = storage.clone();
    let mut aggregator = aggregator::Aggregator::new(streamer, aggregator_token, storage_clone);
    let aggregator_fut = tokio::spawn(async move { aggregator.run().await });
    log::debug!("Aggregator started");

    // start api
    let api_token = token.clone();
    let api_fut = tokio::spawn(solana_data_aggregator::api::run_api(
        args.socket,
        storage,
        api_token,
    ));
    log::debug!("API started");

    // graceful shutdown monitor
    let shutdown_token = token.clone();
    let shutdown_fut = tokio::spawn(async move {
        ctrl_c().await.expect("failed to listen for ctrl+c event");
        log::info!("TERMINATING");
        shutdown_token.cancel();
    });

    let _ = tokio::join!(shutdown_fut, monitor_fut, aggregator_fut, api_fut);
    Ok(())
}

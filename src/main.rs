extern crate dotenv;
use dotenv::dotenv;

use eyre::Result;
use solana_data_aggregator::monitor::SlotMonitor;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    // config env
    dotenv().ok();

    let token = CancellationToken::new();
    let provider_rpc = std::env::var("PROVIDER_RPC_URL")?;
    let provider_ws = std::env::var("PROVIDER_WS_URL")?;

    // create and start slot monitor
    let monitor = SlotMonitor::new(provider_ws.as_str()).await?;
    let monitor_fut = tokio::spawn(async move { monitor.start_monitoring(token.clone()).await });

    // create and start slot processor
    Ok(())
}

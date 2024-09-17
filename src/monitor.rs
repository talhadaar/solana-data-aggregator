use crate::error::*;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_program::slot_history::Slot;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct SlotMonitor {
    client: Arc<PubsubClient>,
    sender: UnboundedSender<Slot>,
    token: CancellationToken,
}

impl SlotMonitor {
    pub async fn new(
        wss_url: &str,
        token: CancellationToken,
        monitor_tx: UnboundedSender<Slot>,
    ) -> Result<Self> {
        let client = Arc::new(match PubsubClient::new(wss_url).await {
            Ok(client) => {
                log::debug!("PubsubClient created");
                client
            }
            Err(e) => {
                log::error!("SlotMonitor: PubsubClient creation failed: {}", e);
                return Err(Error::PubSubError(e));
            }
        });
        Ok(Self {
            client,
            sender: monitor_tx,
            token,
        })
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        // create subscription
        let (mut sub, unsub) = match self.client.slot_subscribe().await {
            Ok(sub) => {
                log::debug!("Slot Subscription created");
                sub
            }
            Err(e) => {
                log::error!("Slot Subscription failed: {}", e);
                return Err(Error::PubSubError(e));
            }
        };

        log::debug!("Starting slot monitoring");
        loop {
            if self.token.is_cancelled() {
                // If cancellation occurs, unsubscribe and return
                unsub().await;
                log::info!("TERMINATING");
                return Err(Error::Termination);
            }

            if let Some(slot_info) = sub.next().await {
                // If a slot notification is received, queue the slot for processing
                match self.sender.send(slot_info.root) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Channel failure: {}", e);
                        return Err(Error::ChannelFailed(
                            "SlotMonitor".to_string(),
                            e.to_string(),
                        ));
                    }
                }
            }
        }
    }
}

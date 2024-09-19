use crate::error::*;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_program::slot_history::Slot;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct SlotMonitor {
    // this doesn't need to be in an Arc
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

#[cfg(test)]
mod slot_monitor_tests {
    use crate::error::Error;
    use crate::monitor::SlotMonitor;
    use tokio_test::{assert_err, assert_ok};
    use tokio_util::sync::CancellationToken;

    #[tokio::test(flavor = "multi_thread")]
    async fn sanity_check() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let token = CancellationToken::new();

        // invalid url
        let monitor = SlotMonitor::new("amdkasjdkasjh", token.clone(), tx.clone()).await;
        assert_err!(monitor);

        // connection refused
        let monitor = SlotMonitor::new("ws://localhost:8899", token.clone(), tx.clone()).await;
        assert_err!(monitor);

        // successful connection
        let monitor = SlotMonitor::new("wss://api.testnet.solana.com", token, tx).await;
        assert_ok!(monitor);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn functional_test() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let token = CancellationToken::new();
        let monitor = SlotMonitor::new("wss://api.testnet.solana.com", token.clone(), tx)
            .await
            .unwrap();

        let monitor_fut = tokio::spawn(async move { monitor.start_monitoring().await });

        // slots are being received and sent to the channel
        for _i in 0..5 {
            let slot = rx.recv().await.unwrap();
            println!("Slot: {}", slot);
            assert!(slot > 0);
        }

        // initiate termination
        token.cancel();

        // check termination
        let result = tokio::join!(monitor_fut)
            .0
            .unwrap()
            .unwrap_err()
            .to_string();
        assert_eq!(result, Error::Termination.to_string());
    }
}

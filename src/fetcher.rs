use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;

/// Fetches block data and parses it into useable types
pub struct Fetcher(Arc<RpcClient>);

impl Fetcher {
    pub fn new(rpc_url: String) -> Self {
        let rpc_client = RpcClient::new(rpc_url);
        Self(Arc::new(rpc_client))
    }
}

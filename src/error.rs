use nanodb::error::NanoDBError;
use solana_client::client_error::ClientError;
use solana_client::pubsub_client::PubsubClientError;
use std::env::VarError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Solana RPC Client error: {0}")]
    RpcError(ClientError),
    #[error("Solana PubSub Client error: {0}")]
    PubSubError(PubsubClientError),
    #[error("Channel Failed: {0} - Failure: {1}")]
    ChannelFailed(String, String),
    #[error("Termination Occured")]
    Termination,
    #[error("Storage Error: {0}")]
    StorageError(String),
    #[error("Var Error: {0}")]
    VarError(String),
}

impl From<VarError> for Error {
    fn from(e: VarError) -> Self {
        Self::VarError(e.to_string())
    }
}

impl From<NanoDBError> for Error {
    fn from(e: NanoDBError) -> Self {
        Self::StorageError(e.to_string())
    }
}

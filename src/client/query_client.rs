use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;

use anyhow::Result;

use crate::config::{Account, Network};

use super::SigningClient;

#[async_trait]
pub trait Querier {
    /// Send an arbitrary query to the RPC server.
    async fn query<Req, Res>(&self, address: String, message: &Req) -> Result<Res>
    where
        Req: Serialize + ?Sized + Sync,
        Res: for<'de> serde::Deserialize<'de>;
}
pub struct QueryClient {
    network: Network,
}

impl QueryClient {
    pub fn new(network: Network) -> Self {
        Self { network }
    }

    pub fn into_signing(self, account: Account, derivation_path: String) -> SigningClient {
        SigningClient::new(self.network, account, derivation_path)
    }
}

#[async_trait]
impl Querier for QueryClient {
    async fn query<Req, Res>(&self, address: String, message: &Req) -> Result<Res>
    where
        Req: Serialize + ?Sized + Sync,
        Res: for<'de> serde::Deserialize<'de>,
    {
        let encoded = STANDARD.encode(serde_json::to_vec(message)?);
        let path = format!("cosmwasm/wasm/v1/contract/{address}/smart/{encoded}",);
        let res = self.network.get(path).await?;
        Ok(serde_json::from_value(res["data"].clone())?)
    }
}

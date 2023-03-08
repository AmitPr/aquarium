use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;

use anyhow::Result;
use serde_json::Value;

use crate::{account::AccountWithInfo, Network, SigningClient};

#[async_trait]
pub trait Querier {
    /// Send an arbitrary query to the RPC server.
    async fn query<Req, Res>(&self, address: String, message: &Req) -> Result<Res>
    where
        Req: Serialize + ?Sized + Sync,
        Res: for<'de> serde::Deserialize<'de>;

    /// Wait for a transaction to be committed.
    async fn wait_for_transaction(&self, tx_hash: String) -> Result<Value>;
}
pub struct QueryClient {
    network: Network,
}

impl QueryClient {
    pub fn new(network: Network) -> Self {
        Self { network }
    }

    pub fn into_signing(self, account: AccountWithInfo) -> SigningClient {
        SigningClient::new(self.network, account)
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
        let value = serde_json::from_value(res["data"].clone());
        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(anyhow::anyhow!(
                "Encountered error while querying: {err}\nResponse: {res:#}"
            )),
        }
    }

    async fn wait_for_transaction(&self, tx_hash: String) -> Result<Value> {
        let path = format!("cosmos/tx/v1beta1/txs/{tx_hash}",);
        loop {
            let res = self.network.get(path.clone()).await?;
            // Check if not found, and if so, sleep and try again.
            if res["code"].as_u64().map_or(false, |c| c == 5) {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }

            return Ok(res);
        }
    }
}

use reqwest::Client;
use serde::{Deserialize, Serialize};

use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub chain_id: String,
    pub lcd_addr: String,
    pub gas_price: f64,
    pub gas_adjustment: f64,
    pub gas_denom: String,
    pub account_prefix: String,
}

impl Network {
    pub async fn get(&self, path: impl AsRef<str>) -> Result<serde_json::Value> {
        let client = Client::new();
        let path = format!("{}/{}", self.lcd_addr, path.as_ref());
        Ok(client.get(path).send().await?.json().await?)
    }

    pub async fn post(
        &self,
        path: impl AsRef<str>,
        body: impl Serialize,
    ) -> Result<serde_json::Value> {
        let client = Client::new();
        let path = format!("{}/{}", self.lcd_addr, path.as_ref());
        Ok(client.post(path).json(&body).send().await?.json().await?)
    }

    pub async fn current_block_height(&self) -> Result<u32> {
        let response = self
            .get("cosmos/base/tendermint/v1beta1/blocks/latest")
            .await?;
        Ok(response["block"]["header"]["height"]
            .as_str()
            .ok_or(anyhow::anyhow!("Error parsing block height"))?
            .parse::<u32>()?)
    }

    pub async fn account_sequence_numbers(&self, address: String) -> Result<(u64, u64)> {
        let response = self
            .get(format!("cosmos/auth/v1beta1/accounts/{address}"))
            .await?;
        let acc_num = response["account"]["account_number"]
            .as_str()
            .ok_or(anyhow::anyhow!("Error parsing account number"))?
            .parse::<u64>()?;
        let seq_num = response["account"]["sequence"]
            .as_str()
            .ok_or(anyhow::anyhow!("Error parsing sequence number"))?
            .parse::<u64>()?;
        Ok((acc_num, seq_num))
    }
}

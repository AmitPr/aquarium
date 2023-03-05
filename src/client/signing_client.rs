use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use cosmrs::{
    cosmwasm::{MsgExecuteContract, MsgInstantiateContract, MsgMigrateContract, MsgStoreCode},
    tx::{mode_info::Single, Body, Fee, ModeInfo, Msg, SignDoc, SignMode, SignerInfo},
    AccountId, Any, Coin, Denom,
};
use serde::Serialize;
use serde_json::json;

use crate::config::{Account, Network};

use super::{query_client::Querier, QueryClient};

pub const TIMEOUT_BLOCK_AMOUNT: u32 = 100;

#[async_trait]
pub trait Executor {
    /// Send a WASM execute
    async fn execute_smart<Req>(
        &self,
        address: String,
        message: &Req,
        funds: Vec<Coin>,
        memo: Option<String>,
    ) -> Result<String>
    where
        Req: Serialize + ?Sized + Sync + Clone;

    /// Send a storecode
    async fn store_code(&self, bytecode: Vec<u8>, memo: Option<String>) -> Result<String>;

    /// Send a WASM instantiate
    async fn instantiate<Req>(
        &self,
        code_id: u64,
        msg: &Req,
        funds: Vec<Coin>,
        label: Option<String>,
        admin: Option<String>,
        memo: Option<String>,
    ) -> Result<String>
    where
        Req: Serialize + ?Sized + Sync + Clone;

    /// Send a WASM migrate
    async fn migrate<Req>(
        &self,
        address: String,
        code_id: u64,
        msg: &Req,
        memo: Option<String>,
    ) -> Result<String>
    where
        Req: Serialize + ?Sized + Sync + Clone;

    /// Execute generic messages
    async fn execute<Req>(&self, messages: Vec<Req>, memo: Option<String>) -> Result<String>
    where
        Req: Msg + Sync + Send + Clone;
}

pub struct SigningClient {
    network: Network,
    account: Account,
    derivation_path: String,
}

impl SigningClient {
    pub fn new(network: Network, account: Account, derivation_path: String) -> Self {
        Self {
            network,
            account,
            derivation_path,
        }
    }

    pub fn into_query(self) -> QueryClient {
        QueryClient::new(self.network)
    }

    pub async fn estimate_gas<Req>(
        &self,
        messages: Vec<Req>,
        memo: Option<String>,
        block_height: u32,
        acc_num: u64,
        seq_num: u64,
    ) -> Result<u64>
    where
        Req: Msg,
    {
        let body = Body::new(
            messages
                .into_iter()
                .map(|m| m.to_any().unwrap())
                .collect::<Vec<Any>>(),
            memo.unwrap_or_default(),
            block_height + TIMEOUT_BLOCK_AMOUNT,
        );

        let tx_raw = {
            let (pk, sk) = self.account.get_keypair(&self.derivation_path)?;

            let auth_info = SignerInfo {
                public_key: Some(pk.into()),
                mode_info: ModeInfo::Single(Single {
                    mode: SignMode::Unspecified,
                }),
                sequence: seq_num,
            }
            .auth_info(Fee {
                amount: vec![],
                gas_limit: Default::default(),
                payer: None,
                granter: None,
            });

            let sign_doc = SignDoc::new(
                &body,
                &auth_info,
                &self
                    .network
                    .chain_id
                    .clone()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid chain id"))?,
                acc_num,
            )
            .map_err(|e| anyhow::anyhow!(e))?;

            let tx_raw = sign_doc
                .sign(&sk)
                .map_err(|e| anyhow::anyhow!(e))?
                .to_bytes()
                .map_err(|e| anyhow::anyhow!(e))?;

            STANDARD.encode(tx_raw)
        };

        let res = self
            .network
            .post(
                "cosmos/tx/v1beta1/simulate",
                &json!({
                    "tx_bytes": tx_raw,
                }),
            )
            .await?;

        let gas = res["gas_info"]["gas_used"]
            .as_str()
            .ok_or(anyhow::anyhow!("Error simulating transaction: {res:#?}"))?
            .parse::<f64>()?;
        let gas = (gas * self.network.gas_adjustment).ceil() as u64;

        Ok(gas)
    }
}

#[async_trait]
impl Querier for SigningClient {
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

#[async_trait]
impl Executor for SigningClient {
    async fn execute_smart<Req>(
        &self,
        address: String,
        message: &Req,
        funds: Vec<Coin>,
        memo: Option<String>,
    ) -> Result<String>
    where
        Req: Serialize + ?Sized + Sync + Clone,
    {
        let encoded = STANDARD.encode(serde_json::to_vec(message)?);
        let msg = MsgExecuteContract {
            sender: AccountId::from_str(&self.account.address)
                .map_err(|_| anyhow::anyhow!("Invalid sender address: {}", self.account.address))?,
            contract: AccountId::from_str(&address)
                .map_err(|_| anyhow::anyhow!("Invalid contract address"))?,
            msg: encoded.into_bytes(),
            funds,
        };

        self.execute(vec![msg], memo).await
    }

    async fn store_code(&self, bytecode: Vec<u8>, memo: Option<String>) -> Result<String> {
        let msg = MsgStoreCode {
            sender: AccountId::from_str(&self.account.address)
                .map_err(|_| anyhow::anyhow!("Invalid sender address: {}", self.account.address))?,
            wasm_byte_code: bytecode,
            instantiate_permission: None,
        };

        self.execute(vec![msg], memo).await
    }

    async fn instantiate<Req>(
        &self,
        code_id: u64,
        msg: &Req,
        funds: Vec<Coin>,
        label: Option<String>,
        admin: Option<String>,
        memo: Option<String>,
    ) -> Result<String>
    where
        Req: Serialize + ?Sized + Sync + Clone,
    {
        let encoded = STANDARD.encode(serde_json::to_vec(msg)?);

        let msg = MsgInstantiateContract {
            sender: AccountId::from_str(&self.account.address)
                .map_err(|_| anyhow::anyhow!("Invalid sender address: {}", self.account.address))?,
            msg: encoded.into_bytes(),
            code_id,
            admin: admin
                .map(|a| {
                    AccountId::from_str(&a).map_err(|_| anyhow::anyhow!("Invalid admin address"))
                })
                .transpose()?,
            label,
            funds,
        };

        self.execute(vec![msg], memo).await
    }

    async fn migrate<Req>(
        &self,
        address: String,
        code_id: u64,
        msg: &Req,
        memo: Option<String>,
    ) -> Result<String>
    where
        Req: Serialize + ?Sized + Sync + Clone,
    {
        let encoded = STANDARD.encode(serde_json::to_vec(msg)?);

        let msg = MsgMigrateContract {
            sender: AccountId::from_str(&self.account.address)
                .map_err(|_| anyhow::anyhow!("Invalid sender address: {}", self.account.address))?,
            contract: AccountId::from_str(&address)
                .map_err(|_| anyhow::anyhow!("Invalid contract address"))?,
            code_id,
            msg: encoded.into_bytes(),
        };

        self.execute(vec![msg], memo).await
    }

    async fn execute<Req>(&self, messages: Vec<Req>, memo: Option<String>) -> Result<String>
    where
        Req: Msg + Sync + Send + Clone,
    {
        let block_height = self.network.current_block_height().await?;
        let body = Body::new(
            messages
                .clone()
                .into_iter()
                .map(|m| m.to_any().unwrap())
                .collect::<Vec<Any>>(),
            memo.clone().unwrap_or_default(),
            block_height + TIMEOUT_BLOCK_AMOUNT,
        );

        let (acc, sequence) = self
            .network
            .account_sequence_numbers(self.account.address.clone())
            .await?;

        let estimated_gas = self
            .estimate_gas(messages, memo, block_height, acc, sequence)
            .await?;

        let gas_fee = u128::from((estimated_gas as f64 * self.network.gas_price).ceil() as u64);
        let gas_fee = Fee::from_amount_and_gas(
            Coin {
                denom: Denom::from_str(self.network.gas_denom.as_str()).unwrap(),
                amount: gas_fee,
            },
            estimated_gas,
        );

        let tx_raw = {
            let (pk, sk) = self.account.get_keypair(&self.derivation_path)?;
            let auth_info = SignerInfo::single_direct(Some(pk), sequence).auth_info(gas_fee);

            let sign_doc = SignDoc::new(
                &body,
                &auth_info,
                &self
                    .network
                    .chain_id
                    .clone()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid chain id"))?,
                acc,
            )
            .map_err(|e| anyhow::anyhow!(e))?;

            let tx_raw = sign_doc
                .sign(&sk)
                .map_err(|e| anyhow::anyhow!(e))?
                .to_bytes()
                .map_err(|e| anyhow::anyhow!(e))?;
            STANDARD.encode(tx_raw)
        };

        let res = self
            .network
            .post(
                "cosmos/tx/v1beta1/simulate",
                &json!({
                    "tx_bytes": tx_raw,
                    "mode": "BROADCAST_MODE_SYNC",
                }),
            )
            .await?;

        if res["tx_response"]["code"]
            .as_u64()
            .map_or(false, |c| c == 11)
        {
            return Err(anyhow::anyhow!("Insufficient gas"));
        }

        Ok(res["tx_response"]["txhash"]
            .as_str()
            .ok_or(anyhow::anyhow!("Error parsing txhash"))?
            .to_string())
    }
}

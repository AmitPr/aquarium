use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{Account, Network};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: String,
    pub hd_path: String,
    pub networks: HashMap<String, Network>,
    pub accounts: HashMap<String, Account>,
}

impl ProjectConfig {
    pub fn load(path: PathBuf) -> Result<Self> {
        let config = std::fs::read_to_string(path)?;
        toml::from_str(&config).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn save(&self, path: PathBuf) -> Result<()> {
        let config = toml::to_string(self).map_err(|e| anyhow::anyhow!(e))?;
        std::fs::write(path, config)?;
        Ok(())
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project: "Aquarium Project".to_string(),
            networks: HashMap::from([
                (
                    "devnet".to_string(),
                    Network {
                        chain_id: "harpoon-4".to_string(),
                        lcd_addr: "http://localhost:1317".to_string(),
                        gas_price: 0.00125,
                        gas_adjustment: 1.25,
                        gas_denom: "ukuji".to_string(),
                        account_prefix: "kujira".to_string(),
                    },
                ),
                (
                    "testnet".to_string(),
                    Network {
                        chain_id: "harpoon-4".to_string(),
                        lcd_addr: "https://test-lcd-kujira.mintthemoon.xyz".to_string(), //TODO
                        gas_price: 0.00125,
                        gas_adjustment: 1.25,
                        gas_denom: "ukuji".to_string(),
                        account_prefix: "kujira".to_string(),
                    },
                ),
                (
                    "mainnet".to_string(),
                    Network {
                        chain_id: "kaiyo-1".to_string(),
                        lcd_addr: "https://lcd-kujira.mintthemoon.xyz".to_string(), //TODO
                        gas_price: 0.00125,
                        gas_adjustment: 1.25,
                        gas_denom: "ukuji".to_string(),
                        account_prefix: "kujira".to_string(),
                    },
                ),
            ]),
            accounts: HashMap::new(),
            hd_path: "m/44'/118'/0'/0/0".to_string(),
        }
    }
}

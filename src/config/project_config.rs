use config::Map;
use serde::{Deserialize, Serialize};

use super::{Account, Network};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    project: String,
    networks: Map<String, Network>,
    accounts: Map<String, Account>,
    hd_path: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project: "Aquarium Project".to_string(),
            networks: Map::from([
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
            accounts: Map::new(),
            hd_path: "m/44'/118'/0'/0/0".to_string(),
        }
    }
}

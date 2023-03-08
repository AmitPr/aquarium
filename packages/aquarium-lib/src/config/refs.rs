use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContractRefs {
    pub networks: HashMap<String, NetworkSpecificRefs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkSpecificRefs {
    pub categories: HashMap<String, Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    pub code_ids: Vec<u64>,
    pub contracts: Vec<Contract>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub code_id: u64,
    pub address: String,
    #[serde(flatten)]
    pub attrs: HashMap<String, serde_json::Value>,
}

impl ContractRefs {
    pub fn load(path: PathBuf) -> Result<Self> {
        serde_json::from_str(&std::fs::read_to_string(path)?).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn load_or_default(path: PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path);
        match contents {
            Ok(contents) => serde_json::from_str(&contents).map_err(|e| anyhow::anyhow!(e)),
            Err(_) => Ok(Self::default()),
        }
    }

    pub fn save(&self, path: PathBuf) -> Result<()> {
        Ok(std::fs::write(
            path,
            serde_json::to_string(self).map_err(|e| anyhow::anyhow!(e))?,
        )?)
    }
}

impl NetworkSpecificRefs {
    pub fn get_contract(&mut self, category: &str, address: &str) -> Option<&mut Contract> {
        self.categories
            .get_mut(category)
            .and_then(|category| {
                category
                    .contracts
                    .iter_mut()
                    .find(|contract| contract.address == address)
            })
    }
}

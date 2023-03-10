use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContractRefs {
    pub networks: HashMap<String, NetworkSpecificRefs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkSpecificRefs {
    #[serde(flatten)]
    pub contracts: HashMap<String, Contract>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contract {
    pub code_ids: Vec<u64>,
    pub instances: Vec<ContractInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInstance {
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
            serde_json::to_string_pretty(self).map_err(|e| anyhow::anyhow!(e))?,
        )?)
    }
}

impl NetworkSpecificRefs {
    pub fn get_contract_instance(
        &mut self,
        contract: &str,
        address: &str,
    ) -> Option<&mut ContractInstance> {
        self.contracts.get_mut(contract).and_then(|category| {
            category
                .instances
                .iter_mut()
                .find(|instance| instance.address == address)
        })
    }

    pub fn add_contract_instance(&mut self, contract: &str, instance: ContractInstance) {
        let category = self.contracts.entry(contract.to_string()).or_default();
        category.instances.push(instance);
    }

    pub fn add_code_id(&mut self, contract: &str, code_id: u64) {
        let category = self.contracts.entry(contract.to_string()).or_default();
        category.code_ids.push(code_id);
    }

    pub fn get_code_ids(&mut self, contract: &str) -> Option<&mut Vec<u64>> {
        self.contracts
            .get_mut(contract)
            .map(|category| &mut category.code_ids)
    }
}

impl ContractInstance {
    pub fn new(code_id: u64, address: String) -> Self {
        Self {
            code_id,
            address,
            attrs: HashMap::new(),
        }
    }
}

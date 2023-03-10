use std::{collections::HashMap, path::PathBuf, process::Command};

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{
    account::{AccountWithInfo, SerializableAccount},
    cli::task::TaskArgs,
    ContractRefs, Env, Network, QueryClient, SigningClient,
};

pub const CONFIG_FILE_NAME: &str = "Aquarium.toml";

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub config: Config,
}

impl Project {
    #[allow(clippy::field_reassign_with_default)]
    pub fn init(name: String, dir: Option<PathBuf>) -> Result<Self> {
        let root = dir.unwrap_or(std::env::current_dir()?.join(name.clone()));
        if root.exists() {
            if root.is_file() {
                return Err(anyhow::anyhow!("Project root exists as file"));
            } else {
                let config_file = root.join(CONFIG_FILE_NAME);
                if config_file.exists() {
                    return Err(anyhow::anyhow!("Project already exists"));
                }
            }
        } else {
            std::fs::create_dir_all(&root)?;
        }

        // run dotenv config in project root
        dotenv::from_path(root.join(".env")).ok();

        let mut config = Config::default();
        config.project = name;

        let scripts_dir = root.join(config.scripts_path.clone());
        std::fs::create_dir_all(&scripts_dir)?;
        let status = Command::new("cargo")
            .arg("init")
            .arg("--name")
            .arg("scripts")
            .arg("--bin")
            .current_dir(&scripts_dir)
            .status()?;

        println!("Initializing scripts crate finished: {}", status);

        let project = Self { root, config };
        project.save()?;

        Ok(project)
    }

    pub fn load() -> Result<Self> {
        let config_file = Self::find_config_file()?;
        let root = config_file
            .parent()
            .ok_or(anyhow::anyhow!("Could not find project root"))?
            .to_path_buf();

        // run dotenv config in project root
        dotenv::from_path(root.join(".env")).ok();
        
        let config = Config::load(config_file)?;
        Ok(Self { root, config })
    }

    pub fn save(&self) -> Result<()> {
        let config_file = self.root.join(CONFIG_FILE_NAME);
        self.config.save(config_file)?;
        Ok(())
    }

    fn find_config_file() -> Result<PathBuf> {
        let cwd = std::env::current_dir()?;
        let ancestors = cwd.ancestors();
        for ancestor in ancestors {
            let config_file = ancestor.join(CONFIG_FILE_NAME);
            if config_file.exists() {
                return Ok(config_file);
            }
        }
        Err(anyhow::anyhow!("Could not find config file"))
    }

    pub fn env(&self) -> Result<Env> {
        let args = TaskArgs::parse();

        let network_name = args
            .network
            .or_else(|| match self.config.default_network {
                Some(ref n) => Some(n.clone()),
                None => self.config.networks.keys().next().cloned(),
            })
            .ok_or(anyhow::anyhow!("No networks specified"))?;
        let network = self
            .config
            .networks
            .get(&network_name)
            .ok_or(anyhow::anyhow!("Could not find network"))?;

        let account = args.account.or(self.config.accounts.keys().next().cloned());
        let account = account
            .map(|a| {
                self.config
                    .accounts
                    .get(&a)
                    .ok_or(anyhow::anyhow!("Could not find account"))
            })
            .transpose()?
            .ok_or(anyhow::anyhow!("No accounts specified"))?;
        let account = AccountWithInfo::new(
            account.clone(),
            self.config.hd_path.clone(),
            &network.account_prefix,
        )?;

        let querier = QueryClient::new(network.clone());
        let executor = SigningClient::new(network.clone(), account);

        let refs_path = self.root.join("contracts.json");
        let refs = ContractRefs::load_or_default(refs_path.clone())?
            .networks
            .get(&network_name)
            .cloned()
            .unwrap_or_default();

        let env = Env::new(network_name, querier, executor, refs, refs_path)?;

        Ok(env)
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            root: std::env::current_dir().unwrap(),
            config: Config::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project: String,
    pub scripts_path: String,
    pub hd_path: String,
    pub default_network: Option<String>,
    pub networks: HashMap<String, Network>,
    pub accounts: HashMap<String, SerializableAccount>,
}

impl Config {
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

impl Default for Config {
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
            default_network: Some("devnet".to_string()),
            scripts_path: "scripts".to_string(),
        }
    }
}

use std::path::PathBuf;

use anyhow::Result;
use config::Config;

use crate::config::ProjectConfig;

pub const CONFIG_FILE_NAME: &str = "Aquarium.toml";

pub struct Project {
    pub root: PathBuf,
    pub config: ProjectConfig,
}

impl Project {
    pub fn load_or_default() -> Result<Self> {
        let cfg = Config::builder().add_source(Config::try_from(&ProjectConfig::default())?);
        let (cfg, root) = match Self::find_config_file() {
            Ok(config) => {
                let cfg = cfg.add_source(config::File::from(config.clone()));
                let root = config
                    .parent()
                    .ok_or(anyhow::anyhow!("Could not find project root"))?
                    .to_path_buf();
                (cfg, root)
            }
            Err(_) => (cfg, std::env::current_dir()?),
        };
        let config = cfg.build()?.try_deserialize::<ProjectConfig>()?;
        Ok(Self { root, config })
    }

    pub fn load() -> Result<Self> {
        let config_file = Self::find_config_file()?;
        let root = config_file
            .parent()
            .ok_or(anyhow::anyhow!("Could not find project root"))?
            .to_path_buf();
        let config = Config::builder()
            .add_source(config::File::from(config_file))
            .build()?
            .try_deserialize::<ProjectConfig>()?;
        Ok(Self { root, config })
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
}

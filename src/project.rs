use std::path::PathBuf;

use anyhow::Result;

use crate::config::ProjectConfig;

pub const CONFIG_FILE_NAME: &str = "Aquarium.toml";

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub config: ProjectConfig,
}

impl Project {
    #[allow(clippy::field_reassign_with_default)]
    pub fn init(name: String, dir: Option<PathBuf>) -> Result<Self> {
        let root = dir.unwrap_or(std::env::current_dir()?.join(name.clone()));
        // make sure the project root is empty
        if root.exists() && (root.is_file() || root.read_dir()?.next().is_some()) {
            return Err(anyhow::anyhow!("Project root already exists"));
        }
        std::fs::create_dir_all(&root)?;

        let mut config = ProjectConfig::default();
        config.project = name;

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
        let config = ProjectConfig::load(config_file)?;
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
}

impl Default for Project {
    fn default() -> Self {
        Self {
            root: std::env::current_dir().unwrap(),
            config: ProjectConfig::default(),
        }
    }
}

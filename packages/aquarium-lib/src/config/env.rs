use std::path::PathBuf;

use crate::{ContractRefs, QueryClient, SigningClient, refs::NetworkSpecificRefs};

pub struct Env {
    pub network: String,
    pub querier: QueryClient,
    pub executor: SigningClient,
    refs_path: PathBuf,
    pub refs: NetworkSpecificRefs,
}

impl Env {
    pub fn new(
        network: String,
        querier: QueryClient,
        executor: SigningClient,
        refs: NetworkSpecificRefs,
        refs_path: PathBuf,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            network,
            querier,
            executor,
            refs,
            refs_path,
        })
    }

    pub fn save_refs(&self) -> Result<(), anyhow::Error> {
        let mut crefs = ContractRefs::load(self.refs_path.clone())?;
        crefs.networks.insert(self.network.clone(), self.refs.clone());
        crefs.save(self.refs_path.clone())
    }
}
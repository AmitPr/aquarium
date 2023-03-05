use anyhow::Result;
use cosmrs::{
    bip32::Mnemonic,
    crypto::{secp256k1::SigningKey, PublicKey},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub mnemonic: String,
    pub address: String,
}

impl Account {
    pub fn get_keypair(&self, derivation_path: impl AsRef<str>) -> Result<(PublicKey, SigningKey)> {
        let seed = Mnemonic::new(self.mnemonic.clone(), Default::default())?.to_seed("");
        let sk = SigningKey::derive_from_path(&seed, &derivation_path.as_ref().parse()?)?;
        let pk = sk.public_key();
        Ok((pk, sk))
    }
}

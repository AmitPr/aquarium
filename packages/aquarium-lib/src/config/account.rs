use anyhow::Result;
use cosmrs::{
    bip32::Mnemonic,
    crypto::{secp256k1::SigningKey, PublicKey},
    AccountId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAccount {
    pub mnemonic: String,
}

impl SerializableAccount {
    pub fn get_keypair(&self, derivation_path: impl AsRef<str>) -> Result<(PublicKey, SigningKey)> {
        let seed = Mnemonic::new(self.mnemonic.clone(), Default::default())?.to_seed("");
        let sk = SigningKey::derive_from_path(&seed, &derivation_path.as_ref().parse()?)?;
        let pk = sk.public_key();
        Ok((pk, sk))
    }
}

#[derive(Debug, Clone)]
pub struct AccountWithInfo {
    inner: SerializableAccount,
    pub derivation_path: String,
    pub address: AccountId,
}

impl AccountWithInfo {
    pub fn new(
        account: SerializableAccount,
        derivation_path: impl AsRef<str>,
        prefix: impl AsRef<str>,
    ) -> Result<Self> {
        let (pk, _) = account.get_keypair(&derivation_path)?;
        let address = pk
            .account_id(prefix.as_ref())
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(Self {
            inner: account,
            derivation_path: derivation_path.as_ref().to_string(),
            address,
        })
    }

    pub fn get_keypair(&self) -> Result<(PublicKey, SigningKey)> {
        self.inner.get_keypair(&self.derivation_path)
    }
}

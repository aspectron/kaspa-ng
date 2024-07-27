use crate::imports::*;
use kaspa_wallet_core::wallet::EncryptedMnemonic;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
struct LegacyWalletJSONInner {
    mnemonic: String,
}

#[derive(Debug, Deserialize)]
struct LegacyWalletJSON {
    wallet: LegacyWalletJSONInner,
}

// #[derive(Debug, Deserialize)]
// struct KNGWalletJSON {
//     payload: String,
// }

#[derive(Debug)]
pub struct SingleWalletFileV0 {
    pub num_threads: u32,
    pub encrypted_mnemonic: EncryptedMnemonic<Vec<u8>>,
    pub xpublic_key: String,
    pub ecdsa: bool,
}
impl Clone for SingleWalletFileV0 {
    fn clone(&self) -> Self {
        Self {
            num_threads: self.num_threads,
            encrypted_mnemonic: EncryptedMnemonic {
                cipher: self.encrypted_mnemonic.cipher.clone(),
                salt: self.encrypted_mnemonic.salt.clone(),
            },
            xpublic_key: self.xpublic_key.clone(),
            ecdsa: self.ecdsa,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WalletType {
    SingleV0(SingleWalletFileV0),
    // SingleV1(SingleWalletFileV1<'a, Vec<u8>>),
    // MultiV0(MultisigWalletFileV0<'a, Vec<u8>>),
    // MultiV1(MultisigWalletFileV1<'a, Vec<u8>>),
}

#[derive(Debug, Default, Deserialize)]
struct EncryptedMnemonicIntermediate {
    #[serde(with = "kaspa_utils::serde_bytes")]
    cipher: Vec<u8>,
    #[serde(with = "kaspa_utils::serde_bytes")]
    salt: Vec<u8>,
}
impl From<EncryptedMnemonicIntermediate> for EncryptedMnemonic<Vec<u8>> {
    fn from(value: EncryptedMnemonicIntermediate) -> Self {
        Self {
            cipher: value.cipher,
            salt: value.salt,
        }
    }
}

//golang wallet file
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnifiedWalletIntermediate {
    version: u32,
    num_threads: Option<u8>,
    encrypted_mnemonics: Vec<EncryptedMnemonicIntermediate>,
    public_keys: Vec<String>,
    //minimum_signatures: u16,
    //cosigner_index: u8,
    ecdsa: bool,
}

impl UnifiedWalletIntermediate {
    fn into_wallet_type(mut self) -> Result<WalletType> {
        let single = self.encrypted_mnemonics.len() == 1 && self.public_keys.len() == 1;
        let wallet = match (single, self.version) {
            (true, 0) | (true, 1) => {
                WalletType::SingleV0(SingleWalletFileV0 {
                    num_threads: self
                    .num_threads
                    .unwrap_or(8)
                    //.ok_or(Error::custom("num_threads must present in case of v0 wallet"))?
                    as u32,
                    encrypted_mnemonic: std::mem::take(&mut self.encrypted_mnemonics[0]).into(),
                    xpublic_key: self.public_keys[0].to_string(),
                    ecdsa: self.ecdsa,
                })
            }
            _ => return Err(Error::custom("Multisig wallet import is not supported.")),
        };
        // (true, WalletVersion::One) => WalletType::SingleV1(SingleWalletFileV1 {
        //     num_threads: 8,
        //     encrypted_mnemonic: std::mem::take(&mut self.encrypted_mnemonics[0]).into(),
        //     xpublic_key: self.public_keys[0],
        //     ecdsa: self.ecdsa,
        // }),
        // (false, WalletVersion::Zero) => WalletType::MultiV0(MultisigWalletFileV0 {
        //     num_threads: self
        //         .num_threads
        //         .expect("num_threads must present in case of v0")
        //         as u32,
        //     encrypted_mnemonics: self
        //         .encrypted_mnemonics
        //         .into_iter()
        //         .map(
        //             |EncryptedMnemonicIntermediate { cipher, salt }| EncryptedMnemonic {
        //                 cipher,
        //                 salt,
        //             },
        //         )
        //         .collect(),
        //     xpublic_keys: self.public_keys,
        //     required_signatures: self.minimum_signatures,
        //     cosigner_index: self.cosigner_index,
        //     ecdsa: self.ecdsa,
        // }),
        // (false, WalletVersion::One) => WalletType::MultiV1(MultisigWalletFileV1 {
        //     encrypted_mnemonics: self
        //         .encrypted_mnemonics
        //         .into_iter()
        //         .map(
        //             |EncryptedMnemonicIntermediate { cipher, salt }| EncryptedMnemonic {
        //                 cipher,
        //                 salt,
        //             },
        //         )
        //         .collect(),
        //     xpublic_keys: self.public_keys,
        //     required_signatures: self.minimum_signatures,
        //     cosigner_index: self.cosigner_index,
        //     ecdsa: self.ecdsa,
        // }),

        Ok(wallet)
    }
}

#[derive(Debug, Clone)]
pub enum WalletFileData {
    Legacy(String),
    GoWallet(WalletType),
    Core(String),
}

#[derive(Debug, Clone)]
pub enum WalletFileDecryptedData {
    Legacy(String),
    //GoWallet(WalletType),
    Core(String),
}
impl Display for WalletFileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Legacy(data) => f.write_str(&format!("Legacy: {data}")),
            Self::GoWallet(data) => f.write_str(&format!("Go Wallet: {data:?}")),
            Self::Core(data) => f.write_str(&format!("Core BIP-44: {data}")),
        }
    }
}

pub fn parse_wallet_file(contents: &str) -> Result<WalletFileData> {
    if let Ok(data) = serde_json::from_str::<LegacyWalletJSON>(contents) {
        Ok(WalletFileData::Legacy(data.wallet.mnemonic))
    } else if let Ok(data) = serde_json::from_str::<LegacyWalletJSONInner>(contents) {
        Ok(WalletFileData::Legacy(data.mnemonic))
    } else if let Ok(data) = serde_json::from_str::<UnifiedWalletIntermediate>(contents) {
        Ok(WalletFileData::GoWallet(data.into_wallet_type()?))
    } else {
        Err(Error::Custom("Unable to parse wallet file".into()))
    }
}

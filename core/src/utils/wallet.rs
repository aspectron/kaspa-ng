use crate::imports::*;
use kaspa_wallet_core::wallet::{
    EncryptedMnemonic, MultisigWalletFileV0, MultisigWalletFileV1, SingleWalletFileV0,
    SingleWalletFileV1,
};
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
pub enum WalletType<'a> {
    SingleV0(SingleWalletFileV0<'a, Vec<u8>>),
    SingleV1(SingleWalletFileV1<'a, Vec<u8>>),
    MultiV0(MultisigWalletFileV0<'a, Vec<u8>>),
    MultiV1(MultisigWalletFileV1<'a, Vec<u8>>),
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

#[derive(serde_repr::Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
enum WalletVersion {
    Zero = 0,
    One = 1,
}

//golang wallet file
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnifiedWalletIntermediate<'a> {
    version: WalletVersion,
    num_threads: Option<u8>,
    encrypted_mnemonics: Vec<EncryptedMnemonicIntermediate>,
    #[serde(borrow)]
    public_keys: Vec<&'a str>,
    minimum_signatures: u16,
    cosigner_index: u8,
    ecdsa: bool,
}

impl<'a> UnifiedWalletIntermediate<'a> {
    fn into_wallet_type(mut self) -> WalletType<'a> {
        let single = self.encrypted_mnemonics.len() == 1 && self.public_keys.len() == 1;
        match (single, self.version) {
            (true, WalletVersion::Zero) => WalletType::SingleV0(SingleWalletFileV0 {
                num_threads: self
                    .num_threads
                    .expect("num_threads must present in case of v0")
                    as u32,
                encrypted_mnemonic: std::mem::take(&mut self.encrypted_mnemonics[0]).into(),
                xpublic_key: self.public_keys[0],
                ecdsa: self.ecdsa,
            }),
            (true, WalletVersion::One) => WalletType::SingleV1(SingleWalletFileV1 {
                encrypted_mnemonic: std::mem::take(&mut self.encrypted_mnemonics[0]).into(),
                xpublic_key: self.public_keys[0],
                ecdsa: self.ecdsa,
            }),
            (false, WalletVersion::Zero) => WalletType::MultiV0(MultisigWalletFileV0 {
                num_threads: self
                    .num_threads
                    .expect("num_threads must present in case of v0")
                    as u32,
                encrypted_mnemonics: self
                    .encrypted_mnemonics
                    .into_iter()
                    .map(
                        |EncryptedMnemonicIntermediate { cipher, salt }| EncryptedMnemonic {
                            cipher,
                            salt,
                        },
                    )
                    .collect(),
                xpublic_keys: self.public_keys,
                required_signatures: self.minimum_signatures,
                cosigner_index: self.cosigner_index,
                ecdsa: self.ecdsa,
            }),
            (false, WalletVersion::One) => WalletType::MultiV1(MultisigWalletFileV1 {
                encrypted_mnemonics: self
                    .encrypted_mnemonics
                    .into_iter()
                    .map(
                        |EncryptedMnemonicIntermediate { cipher, salt }| EncryptedMnemonic {
                            cipher,
                            salt,
                        },
                    )
                    .collect(),
                xpublic_keys: self.public_keys,
                required_signatures: self.minimum_signatures,
                cosigner_index: self.cosigner_index,
                ecdsa: self.ecdsa,
            }),
        }
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
    GoWallet(WalletType),
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
        Ok(WalletFileData::GoWallet(data.into_wallet_type()))
    } else {
        Err(Error::Custom("Unable to parse wallet file".into()))
    }
}

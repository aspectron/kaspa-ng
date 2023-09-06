

pub struct Secret(pub kaspa_wallet_core::secret::Secret);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret")
            .field("secret", &"********")
            .finish()
    }
}

impl Secret {
    pub fn new(secret: kaspa_wallet_core::secret::Secret) -> Self {
        Self(secret)
    }
}

impl From<kaspa_wallet_core::secret::Secret> for Secret {
    fn from(secret: kaspa_wallet_core::secret::Secret) -> Self {
        Self(secret)
    }
}


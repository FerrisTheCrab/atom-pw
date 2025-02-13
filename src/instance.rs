use std::path::Path;

use argon2::Argon2;
use mongodb::Collection;

use crate::{Account, MasterConfig};

#[derive(Clone)]
pub struct PwInstance {
    pub config: MasterConfig,
    pub accounts: Collection<Account>,
}

impl PwInstance {
    pub fn load(config: &Path) -> Self {
        let config = MasterConfig::read(config);
        let accounts = config.mongodb.load();
        PwInstance { config, accounts }
    }

    pub fn argon2(&self) -> Argon2 {
        self.config.argon2.to_argon2()
    }
}

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use argon2::{Algorithm, Argon2, Params};
use mongodb::{
    options::{AuthMechanism, ClientOptions, Credential},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::Account;

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone)]
pub struct MasterConfig {
    #[serde_inline_default(8080)]
    pub port: u16,
    #[serde(default)]
    pub mongodb: MongoConfig,
    #[serde(default)]
    pub argon2: Argon2Config,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone)]
pub struct MongoConfig {
    #[serde_inline_default("mongodb://localhost:27017".to_string())]
    pub address: String,
    #[serde_inline_default("bob".to_string())]
    pub username: String,
    #[serde_inline_default("cratchit".to_string())]
    pub password: String,
    #[serde_inline_default("admin".to_string())]
    #[serde(rename = "authDB")]
    pub auth_db: String,
    #[serde_inline_default("atomics".to_string())]
    #[serde(rename = "pwDB")]
    pub pw_db: String,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone)]
pub struct Argon2Config {
    #[serde_inline_default("change me".to_string())]
    pub pepper: String,
    #[serde_inline_default("Argon2id".to_string())]
    pub algorithm: String,
    #[serde_inline_default(19)]
    pub version: u8,
    #[serde_inline_default(Params::DEFAULT_M_COST)]
    #[serde(rename = "mCost")]
    pub m_cost: u32,
    #[serde_inline_default(Params::DEFAULT_T_COST)]
    #[serde(rename = "tCost")]
    pub t_cost: u32,
    #[serde_inline_default(Params::DEFAULT_P_COST)]
    #[serde(rename = "pCost")]
    pub p_cost: u32,
    #[serde_inline_default(Params::DEFAULT_OUTPUT_LEN)]
    #[serde(rename = "outputLen")]
    pub output_len: usize,
}

impl MasterConfig {
    fn create(path: &Path) {
        let ser = serde_json::to_vec_pretty(&Self::default()).unwrap();

        if !path.parent().unwrap().exists() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
        }

        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap()
            .write_all(&ser)
            .unwrap();
    }

    pub fn read(path: &Path) -> Self {
        if !path.exists() {
            Self::create(path);
        }

        let content = fs::read(path).unwrap();
        serde_json::from_slice(&content).expect("bad JSON")
    }
}

impl MongoConfig {
    pub fn load(&self) -> Collection<Account> {
        futures::executor::block_on(async { self.get_collection().await })
    }

    async fn get_collection(&self) -> Collection<Account> {
        let mut client_opts = ClientOptions::parse(&self.address).await.unwrap();

        let scram_sha_1_cred = Credential::builder()
            .username(self.username.clone())
            .password(self.password.clone())
            .mechanism(AuthMechanism::ScramSha1)
            .source(self.auth_db.clone())
            .build();

        client_opts.credential = Some(scram_sha_1_cred);
        let client = Client::with_options(client_opts).unwrap();
        client.database(&self.pw_db).collection("pw")
    }
}

impl Argon2Config {
    pub fn to_argon2(&self) -> Argon2<'_> {
        let algorithm = match self.algorithm.as_str() {
            "Argon2d" => Algorithm::Argon2d,
            "Argon2i" => Algorithm::Argon2i,
            "Argon2id" => Algorithm::Argon2id,
            x => panic!("`{x}` is not a valid algorithm"),
        };

        let version = match self.version {
            16 => argon2::Version::V0x10,
            19 => argon2::Version::V0x13,
            x => panic!("`{x}` is not a valid version"),
        };

        let params = Params::new(self.m_cost, self.t_cost, self.p_cost, Some(self.output_len))
            .expect("bad params");

        Argon2::new_with_secret(self.pepper.as_bytes(), algorithm, version, params)
            .expect("could not create argon2")
    }
}

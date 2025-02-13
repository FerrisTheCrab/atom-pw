use argon2::{password_hash::SaltString, PasswordHasher};
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use mongodb::bson::{doc, spec::BinarySubtype, Binary, Bson};
use serde::{Deserialize, Serialize};

use crate::instance::PwInstance;

macro_rules! not_found {
    () => {
        mongodb::error::Error::custom("pw entry not found".to_string())
    };
}

macro_rules! cond_not_found {
    ($x: expr) => {
        if $x == 0 {
            return Err(not_found!());
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "_id")]
    id: u64,
    #[serde(rename = "pwHash")]
    pw_hash: Binary,
}

impl Account {
    fn hash(instance: &PwInstance, pw: &str, id: u64) -> Vec<u8> {
        let b64 = BASE64_STANDARD_NO_PAD.encode(id.to_le_bytes());

        let salt = SaltString::from_b64(&b64).unwrap();
        let hashed = instance
            .argon2()
            .hash_password(pw.as_bytes(), &salt)
            .unwrap();

        hashed.hash.unwrap().as_bytes().to_vec()
    }

    fn verify_pw(&self, instance: &PwInstance, pw: &str) -> bool {
        Self::hash(instance, pw, self.id) == self.pw_hash.bytes
    }
}

impl Account {
    async fn get_with_id(
        instance: &PwInstance,
        id: u64,
    ) -> Result<Option<Account>, mongodb::error::Error> {
        instance
            .accounts
            .find_one(doc! { "_id": Bson::Int64(id as i64) })
            .await
    }

    async fn update(&self, instance: &PwInstance) -> Result<(), mongodb::error::Error> {
        if instance
            .accounts
            .update_one(
                doc! { "_id": Bson::Int64(self.id as i64)},
                doc! { "$set": {"pwHash": &self.pw_hash}},
            )
            .await?
            .matched_count
            == 0
        {
            Err(not_found!())
        } else {
            Ok(())
        }
    }

    #[allow(clippy::wrong_self_convention, clippy::new_ret_no_self)]
    async fn new(&self, instance: &PwInstance) -> Result<(), mongodb::error::Error> {
        instance.accounts.insert_one(self).await?;
        Ok(())
    }

    async fn delete(instance: &PwInstance, id: u64) -> Result<bool, mongodb::error::Error> {
        Ok(instance
            .accounts
            .delete_one(doc! { "_id": Bson::Int64(id as i64) })
            .await?
            .deleted_count
            == 1)
    }

    async fn get_bump_counter(instance: &PwInstance) -> Result<u64, mongodb::error::Error> {
        match Self::get_with_id(instance, 0).await? {
            Some(mut res) => {
                let count =
                    u64::from_le_bytes(res.pw_hash.bytes.as_slice().try_into().unwrap()) + 1;
                res.pw_hash = Binary {
                    subtype: BinarySubtype::Generic,
                    bytes: count.to_le_bytes().to_vec(),
                };
                res.update(instance).await?;
                Ok(count)
            }
            None => {
                let bytes = 1_u64.to_le_bytes().to_vec();
                let counter = Self {
                    id: 0,
                    pw_hash: Binary {
                        subtype: BinarySubtype::Generic,
                        bytes,
                    },
                };
                counter.new(instance).await?;
                Ok(1)
            }
        }
    }
}

impl Account {
    #[allow(clippy::new_ret_no_self)]
    pub async fn create(instance: &PwInstance, pw: String) -> Result<u64, mongodb::error::Error> {
        let counter = Self::get_bump_counter(instance).await?;

        let instance_clone = instance.clone();
        let hash = tokio::task::spawn_blocking(move || Self::hash(&instance_clone, &pw, counter))
            .await
            .unwrap();
        let account = Account {
            id: counter,
            pw_hash: Binary {
                subtype: BinarySubtype::Generic,
                bytes: hash,
            },
        };
        account.new(instance).await?;
        Ok(counter)
    }

    pub async fn set(
        instance: &PwInstance,
        id: u64,
        pw: &str,
    ) -> Result<(), mongodb::error::Error> {
        cond_not_found!(id);
        let mut account = Self::get_with_id(instance, id).await?.ok_or(not_found!())?;
        account.pw_hash = Binary {
            subtype: BinarySubtype::Generic,
            bytes: Self::hash(instance, pw, id),
        };
        account.update(instance).await
    }

    pub async fn remove(instance: &PwInstance, id: u64) -> Result<(), mongodb::error::Error> {
        cond_not_found!(id);

        if Self::delete(instance, id).await? {
            Ok(())
        } else {
            Err(not_found!())
        }
    }

    pub async fn check(
        instance: &PwInstance,
        id: u64,
        pw: String,
    ) -> Result<bool, mongodb::error::Error> {
        cond_not_found!(id);
        let account = Self::get_with_id(instance, id).await?.ok_or(not_found!())?;
        let instance_clone = instance.clone();
        let res = tokio::task::spawn_blocking(move || account.verify_pw(&instance_clone, &pw))
            .await
            .unwrap();
        Ok(res)
    }
}

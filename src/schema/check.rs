#[cfg(feature = "core")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[cfg(feature = "core")]
use crate::{
    instance::PwInstance,
    router::{InternalRouter, Router},
    Account,
};

#[derive(Serialize, Deserialize)]
pub struct CheckReq {
    pub id: u64,
    pub pw: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CheckRes {
    #[serde(rename = "checked")]
    Checked { matches: bool },
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl CheckRes {
    pub fn success(b: bool) -> Self {
        if b {
            Self::Checked { matches: true }
        } else {
            Self::Checked { matches: false }
        }
    }

    pub fn failure(e: mongodb::error::Error) -> Self {
        Self::Error {
            reason: e
                .get_custom::<String>()
                .cloned()
                .unwrap_or(e.kind.to_string()),
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            CheckRes::Checked { .. } => StatusCode::OK,
            CheckRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn check(instance: &PwInstance, payload: CheckReq) -> CheckRes {
        Account::check(instance, payload.id, payload.pw)
            .await
            .map(CheckRes::success)
            .unwrap_or_else(CheckRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn check(
        State(instance): State<PwInstance>,
        Json(payload): Json<CheckReq>,
    ) -> (StatusCode, Json<CheckRes>) {
        let res = InternalRouter::check(&instance, payload).await;
        (res.status(), Json(res))
    }
}

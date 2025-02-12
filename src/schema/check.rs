use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
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
                .map(|x| x.to_string())
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

impl InternalRouter {
    pub async fn check(payload: CheckReq) -> CheckRes {
        Account::check(payload.id, payload.pw)
            .await
            .map(CheckRes::success)
            .unwrap_or_else(CheckRes::failure)
    }
}

impl Router {
    pub async fn check(Json(payload): Json<CheckReq>) -> (StatusCode, Json<CheckRes>) {
        let res = InternalRouter::check(payload).await;
        (res.status(), Json(res))
    }
}

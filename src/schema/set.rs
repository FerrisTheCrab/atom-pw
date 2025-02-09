use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    router::{InternalRouter, Router},
    Account,
};

#[derive(Serialize, Deserialize)]
pub struct SetReq {
    pub id: u64,
    pub pw: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SetRes {
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "error")]
    Error { reason: String },
}

impl SetRes {
    pub fn success(_: ()) -> Self {
        Self::Set
    }

    pub fn failure(e: mongodb::error::Error) -> Self {
        Self::Error {
            reason: e
                .get_custom::<Box<dyn std::fmt::Display>>()
                .map(|x| x.to_string())
                .unwrap_or(e.kind.to_string()),
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Set => StatusCode::OK,
            Self::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn set(payload: SetReq) -> SetRes {
        Account::set(payload.id, &payload.pw)
            .await
            .map(SetRes::success)
            .unwrap_or_else(SetRes::failure)
    }
}

impl Router {
    pub async fn set(Json(payload): Json<SetReq>) -> (StatusCode, Json<SetRes>) {
        let res = InternalRouter::set(payload).await;
        (res.status(), Json(res))
    }
}

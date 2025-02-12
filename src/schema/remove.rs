use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    router::{InternalRouter, Router},
    Account,
};

#[derive(Serialize, Deserialize)]
pub struct RemoveReq {
    pub id: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RemoveRes {
    #[serde(rename = "removed")]
    Removed,
    #[serde(rename = "error")]
    Error { reason: String },
}

impl RemoveRes {
    pub fn success(_: ()) -> Self {
        Self::Removed
    }

    pub fn failure(e: mongodb::error::Error) -> Self {
        Self::Error {
            reason: e
                .get_custom::<String>()
                .map(String::clone)
                .unwrap_or(e.kind.to_string()),
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Removed => StatusCode::OK,
            Self::Error { reason } if reason == "not found" => StatusCode::NOT_FOUND,
            Self::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn remove(payload: RemoveReq) -> RemoveRes {
        Account::remove(payload.id)
            .await
            .map(RemoveRes::success)
            .unwrap_or_else(RemoveRes::failure)
    }
}

impl Router {
    pub async fn remove(Json(payload): Json<RemoveReq>) -> (StatusCode, Json<RemoveRes>) {
        let res = InternalRouter::remove(payload).await;
        (res.status(), Json(res))
    }
}

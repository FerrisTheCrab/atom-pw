use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    instance::PwInstance,
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
                .cloned()
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
    pub async fn remove(instance: &PwInstance, payload: RemoveReq) -> RemoveRes {
        Account::remove(instance, payload.id)
            .await
            .map(RemoveRes::success)
            .unwrap_or_else(RemoveRes::failure)
    }
}

impl Router {
    pub async fn remove(
        State(instance): State<Arc<PwInstance>>,
        Json(payload): Json<RemoveReq>,
    ) -> (StatusCode, Json<RemoveRes>) {
        let res = InternalRouter::remove(&instance, payload).await;
        (res.status(), Json(res))
    }
}

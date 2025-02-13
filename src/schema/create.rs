use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    instance::PwInstance,
    router::{InternalRouter, Router},
    Account,
};

#[derive(Serialize, Deserialize)]
pub struct CreateReq {
    pub pw: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CreateRes {
    #[serde(rename = "created")]
    Created { id: u64 },
    #[serde(rename = "error")]
    Error { reason: String },
}

impl CreateRes {
    pub fn success(id: u64) -> Self {
        Self::Created { id }
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
            Self::Created { .. } => StatusCode::CREATED,
            Self::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn create(instance: &PwInstance, payload: CreateReq) -> CreateRes {
        Account::create(instance, payload.pw)
            .await
            .map(CreateRes::success)
            .unwrap_or_else(CreateRes::failure)
    }
}

impl Router {
    pub async fn create(
        State(instance): State<Arc<PwInstance>>,
        Json(payload): Json<CreateReq>,
    ) -> (StatusCode, Json<CreateRes>) {
        let res = InternalRouter::create(&instance, payload).await;
        (res.status(), Json(res))
    }
}

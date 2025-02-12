use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
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
                .map(|x| x.to_string())
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
    pub async fn create(payload: CreateReq) -> CreateRes {
        Account::create(payload.pw)
            .await
            .map(CreateRes::success)
            .unwrap_or_else(CreateRes::failure)
    }
}

impl Router {
    pub async fn create(Json(payload): Json<CreateReq>) -> (StatusCode, Json<CreateRes>) {
        let res = InternalRouter::create(payload).await;
        (res.status(), Json(res))
    }
}

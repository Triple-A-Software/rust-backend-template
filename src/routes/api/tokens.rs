use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use macros::JsonErrorResponse;
use serde::Deserialize;
use serde_json::json;

use crate::{
    model::user::User,
    repo::token::TokenRepo,
    utils::{error::ErrorResponse, extractors::Session, response::Metadata},
    AppState,
};

pub async fn get(State(state): State<AppState>, Session(user): Session<User>) -> TokenResult {
    if let Some(user) = user {
        let conn = &mut state.db.acquire().await.unwrap();
        let tokens = TokenRepo::list_for_user(user.id, conn)
            .await
            .map_err(|_| TokenError::DatabaseError)?;
        Ok(Json(json!({
            "tokens": tokens.into_iter().map(|t| json!({
                "id": t.id,
                "name": t.name,
                "createdAt": t.created_at,
                "expiration": t.expiration,
            })).collect::<Vec<_>>(),
            "_metadata": Metadata::default(),
        }))
        .into_response())
    } else {
        Err(TokenError::Unauthorized)
    }
}

#[derive(Deserialize)]
pub struct TokenPostBody {
    name: String,
}
pub async fn post(
    State(state): State<AppState>,
    Session(user): Session<User>,
    Json(body): Json<TokenPostBody>,
) -> TokenResult {
    if let Some(user) = user {
        let conn = &mut state.db.acquire().await.unwrap();
        let created = TokenRepo::create_one_access_token(user.id, body.name, conn)
            .await
            .map_err(|_| TokenError::DatabaseError)?;
        Ok(Json(json!({
            "created": {
                "id": created.id,
                "name": created.name,
                "token": created.token,
                "userId": created.user_id,
                "createdAt": created.created_at,
                "updatedAt": created.updated_at,
            },
            "_metadata": Metadata::default(),
        }))
        .into_response())
    } else {
        Err(TokenError::Unauthorized)
    }
}

pub async fn delete_by_id(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Session(user): Session<User>,
) -> TokenResult {
    if let Some(user) = user {
        let conn = &mut state.db.acquire().await.unwrap();
        let _ = TokenRepo::delete_by_id(id, user.id, conn)
            .await
            .map_err(|_| TokenError::DatabaseError)?;
        Ok(Json(json!({
            "deleted": true,
            "_metadata": Metadata::default(),
        }))
        .into_response())
    } else {
        Err(TokenError::Unauthorized)
    }
}

type TokenResult = Result<Response, TokenError>;

#[derive(thiserror::Error, Debug, JsonErrorResponse)]
pub enum TokenError {
    #[error("Database error")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError,

    #[error("Unauthorized")]
    #[status_code(StatusCode::UNAUTHORIZED)]
    Unauthorized,
}

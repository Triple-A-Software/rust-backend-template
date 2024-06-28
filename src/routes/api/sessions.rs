use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use macros::JsonErrorResponse;
use serde_json::json;

use crate::{
    model::user::User,
    repo::session::SessionRepo,
    utils::extractors::Session,
    utils::{error::ErrorResponse, response::Metadata},
    AppState,
};

pub async fn list(
    State(state): State<AppState>,
    Session(user): Session<User>,
) -> impl IntoResponse {
    if let Some(user) = user {
        let conn = &mut state.db.acquire().await.unwrap();
        let sessions = SessionRepo::get_sessions_for_user(user.id, conn)
            .await
            .map_err(|_| SessionError::DatabaseError)?;
        Ok(Json(json!({
            "sessions": sessions,
            "_metadata": Metadata::default(),
        }))
        .into_response())
    } else {
        Err(SessionError::Unauthorized)
    }
}

pub async fn delete_by_id(
    Session(user): Session<User>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> SessionResult {
    if let Some(user) = user {
        let conn = &mut state.db.acquire().await.unwrap();
        SessionRepo::delete_by_id_for_user(id, user.id, conn)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => SessionError::NotFound,
                _ => SessionError::DatabaseError,
            })?;
        Ok(Json(json!({
            "deleted": true,
            "_metadata": Metadata::default(),
        }))
        .into_response())
    } else {
        Err(SessionError::Unauthorized)
    }
}

#[derive(thiserror::Error, Debug, JsonErrorResponse)]
pub enum SessionError {
    #[error("Unauthorized")]
    #[status_code(StatusCode::UNAUTHORIZED)]
    Unauthorized,

    #[error("Database error")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError,

    #[error("Session not found")]
    #[status_code(StatusCode::NOT_FOUND)]
    NotFound,
}
pub type SessionResult = Result<Response, SessionError>;

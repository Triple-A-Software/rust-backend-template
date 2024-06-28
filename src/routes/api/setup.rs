use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use macros::JsonErrorResponse;
use serde::Deserialize;
use serde_json::json;

use crate::{
    config::{self, system_user_uuid},
    model::{auth::Role, user::UserCreateInput, USER_TABLE_NAME},
    repo::activity::{ActivityEntry, ActivityRepo},
    service::{auth::AuthService, setup::SetupService},
    utils::{error::ErrorResponse, response::Metadata},
    AppState,
};

pub async fn is_setup_finished(State(state): State<AppState>) -> impl IntoResponse {
    let conn = &mut state.db.acquire().await.unwrap();
    let result = SetupService::is_setup_finished(conn).await.unwrap();
    (
        StatusCode::OK,
        Json(json!({
            "setupFinished": result,
            "_metadata": Metadata::default()
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct CreateAdminUserPayload {
    email: String,
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
    password: String,
    #[serde(rename = "confirmPassword")]
    confirm_password: String,
}
pub async fn create_admin_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Json(payload): Json<CreateAdminUserPayload>,
) -> Result<Response, Response> {
    let conn = &mut state.db.acquire().await.unwrap();
    if SetupService::is_setup_finished(conn).await.unwrap() {
        return Err(SetupError::AlreadySetup.into_response());
    }
    if payload.password != payload.confirm_password {
        return Err(SetupError::PasswordsDontMatch.into_response());
    }

    let created = AuthService::create_user(
        UserCreateInput {
            email: payload.email,
            first_name: payload.first_name,
            last_name: payload.last_name,
            role: Some(Role::Admin),
            ..Default::default()
        },
        vec![],
        payload.password,
        config::system_user_uuid(),
        conn,
    )
    .await
    .map_err(|e| SetupError::FailedToCreateUser(e.to_string()).into_response())?;
    let _ = ActivityRepo::create_one(
        ActivityEntry::Create {
            ip_address: Some(addr.ip().into()),
            user_agent: Some(user_agent.to_string()),
            action_by_id: system_user_uuid(),
            table_name: USER_TABLE_NAME.to_string(),
            item_id: created.id.to_string(),
            new_data: serde_json::to_string(&created).unwrap(),
        },
        conn,
    )
    .await;
    SetupService::finish_setup(conn)
        .await
        .map_err(|_| SetupError::FailedToFinishSetup.into_response())?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "_metadata": Metadata::default()
        })),
    )
        .into_response())
}

#[derive(thiserror::Error, Debug, JsonErrorResponse)]
pub enum SetupError {
    #[error("Already setup")]
    #[status_code(StatusCode::CONFLICT)]
    AlreadySetup,
    #[error("Failed to create user: {0}")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    FailedToCreateUser(String),
    #[error("Failed to finish setup")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    FailedToFinishSetup,
    #[error("Passwords don't match")]
    #[status_code(StatusCode::BAD_REQUEST)]
    PasswordsDontMatch,
}

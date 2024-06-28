use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    repo::{token::TokenRepo, user::UserRepo},
    service::{auth::AuthService, email::EmailService},
    utils::response::Metadata,
    AppState,
};

#[derive(Deserialize)]
pub struct PasswordResetRequestBody {
    email: String,
}
pub async fn request(
    State(state): State<AppState>,
    Json(body): Json<PasswordResetRequestBody>,
) -> Result<Response, Response> {
    let conn = &mut state.db.acquire().await.unwrap();
    let user = UserRepo::get_by_email(body.email, conn).await;
    if let Ok(user) = user {
        let token = TokenRepo::create_one_password_reset_token(user.id, conn)
            .await
            .map_err(|_| {
                Json(json!({
                    "success": false,
                    "_metadata": Metadata::default(),
                }))
                .into_response()
            })?;
        EmailService::send_password_reset_email(user.email, token.token)
            .await
            .map_err(|_| {
                Json(json!({
                    "success": false,
                    "_metadata": Metadata::default(),
                }))
                .into_response()
            })?;
    }
    Ok(Json(json!({
        "success": true,
        "_metadata": Metadata::default(),
    }))
    .into_response())
}

#[derive(Deserialize)]
pub struct PasswordResetCheckTokenBody {
    token: String,
}
pub async fn token_check(
    State(state): State<AppState>,
    Query(query): Query<PasswordResetCheckTokenBody>,
) -> impl IntoResponse {
    let conn = &mut state.db.acquire().await.unwrap();
    let valid = AuthService::check_password_reset_token(&query.token, conn).await;
    Json(json!({
        "isValid": valid,
        "_metadata": Metadata::default(),
    }))
}

#[derive(Deserialize)]
pub struct PasswordResetBody {
    token: String,
    password: String,
    confirm_password: String,
}
pub async fn reset(
    State(state): State<AppState>,
    Json(body): Json<PasswordResetBody>,
) -> impl IntoResponse {
    let conn = &mut state.db.acquire().await.unwrap();
    let valid = AuthService::check_password_reset_token(&body.token, conn).await;
    if !valid || body.password != body.confirm_password {
        return Json(json!({
            "success": false,
            "_metadata": Metadata::default(),
        }));
    }
    let success = AuthService::reset_password(&body.token, body.password, conn)
        .await
        .unwrap_or(false);
    Json(json!({
        "success": success,
        "_metadata": Metadata::default(),
    }))
}

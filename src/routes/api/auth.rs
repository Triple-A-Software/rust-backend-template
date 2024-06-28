use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::{
    extract::{cookie::Cookie, CookieJar},
    headers::UserAgent,
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::Duration;

use crate::{
    config::SESSION_COOKIE,
    model::user::User,
    repo::activity::{ActivityEntry, ActivityRepo},
    service::auth::AuthService,
    utils::{extractors::Session, response::Metadata},
    AppState,
};

pub async fn check(Session(user): Session<User>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "authenticated": user.is_some(),
            "_metadata": Metadata::default(),
        })),
    )
}

#[derive(Serialize)]
pub struct AuthResponse {
    success: bool,
    _metadata: Metadata,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let conn = &mut state.db.acquire().await.unwrap();
    match AuthService::login(
        payload.email,
        payload.password,
        Some(addr.ip().into()),
        user_agent.to_string(),
        conn,
    )
    .await
    {
        Ok((user, session_with_token)) => {
            let cookie = Cookie::build((SESSION_COOKIE, session_with_token.token.token))
                .path("/")
                .http_only(true)
                .max_age(Duration::days(30));
            let cookies = jar.add(cookie);
            let _ = ActivityRepo::create_one(
                ActivityEntry::Login {
                    ip_address: Some(addr.ip().into()),
                    user_agent: Some(user_agent.to_string()),
                    action_by_id: user.id,
                },
                conn,
            )
            .await;
            (
                cookies,
                Json(AuthResponse {
                    success: true,
                    _metadata: Default::default(),
                }),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Session(cookie): Session<String>,
    Session(user): Session<User>,
    jar: CookieJar,
) -> impl IntoResponse {
    let conn = &mut state.db.acquire().await.unwrap();
    let jar = jar.remove(Cookie::build(SESSION_COOKIE));
    if let Some(cookie) = cookie {
        let _ = AuthService::logout(cookie, &state.db).await;
        if let Some(user) = user {
            let _ = ActivityRepo::create_one(
                ActivityEntry::Logout {
                    ip_address: Some(addr.ip().into()),
                    user_agent: Some(user_agent.to_string()),
                    action_by_id: user.id,
                },
                conn,
            )
            .await;
        }
    }
    (
        jar,
        Json(AuthResponse {
            success: true,
            _metadata: Default::default(),
        }),
    )
        .into_response()
}

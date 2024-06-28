use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use serde::Deserialize;
use serde_json::json;

use crate::{
    model::{
        auth::{Language, PreferencesInput, Theme},
        user::User,
        USER_TABLE_NAME,
    },
    repo::{
        activity::{ActivityEntry, ActivityRepo},
        user::UserRepo,
    },
    utils::extractors::Session,
    AppState,
};

#[derive(Deserialize)]
pub struct PostPreferencesBody {
    theme: Option<Theme>,
    language: Option<Language>,
}
pub async fn post_preferences(
    Session(user): Session<User>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Json(body): Json<PostPreferencesBody>,
) -> impl IntoResponse {
    if let Some(user) = user {
        let conn = &mut state.db.acquire().await.unwrap();
        let updated = UserRepo::update_preferences(
            user.id,
            PreferencesInput {
                theme: body.theme.unwrap_or(user.theme.clone()),
                language: body.language.unwrap_or(user.language.clone()),
            },
            conn,
        )
        .await
        .unwrap();
        let _ = ActivityRepo::create_one(
            ActivityEntry::Update {
                table_name: USER_TABLE_NAME.to_string(),
                item_id: user.id.to_string(),
                ip_address: Some(addr.ip().into()),
                user_agent: Some(user_agent.to_string()),
                old_data: serde_json::to_string(&user).unwrap(),
                new_data: serde_json::to_string(&updated).unwrap(),
                action_by_id: user.id,
            },
            conn,
        )
        .await;
        Json(json!({
            "success": true,
            "_metadata": {},
        }))
        .into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "_metadata": {},
            })),
        )
            .into_response()
    }
}

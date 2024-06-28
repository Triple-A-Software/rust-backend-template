use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;

use crate::{repo::tag::TagRepo, utils::response::Metadata, AppState};

pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let conn = &mut state.db.acquire().await.unwrap();
    let tags = TagRepo::list_all(conn).await.unwrap();
    Json(json!({
        "tags": tags,
        "_metadata": Metadata {
            total_count: Some(tags.len() as i64),
            ..Default::default()
        }
    }))
}

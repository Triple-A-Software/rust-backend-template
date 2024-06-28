use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use macros::JsonErrorResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    model::Activity,
    repo::{activity::ActivityRepo, DatabasePagination},
    utils::{error::ErrorResponse, response::Metadata},
    AppState,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetActivityQuery {
    user_id: Uuid,
    limit: i64,
    page: i64,
}
#[derive(Serialize)]
pub struct GetActivityResponse {
    activity: Vec<Activity>,
    _metadata: Metadata,
}
pub async fn get(
    Query(query): Query<GetActivityQuery>,
    State(state): State<AppState>,
) -> Result<Response, Response> {
    let mut conn = state.db.acquire().await.unwrap();
    let activity = ActivityRepo::list_all_for_user_id(
        query.user_id,
        DatabasePagination {
            limit: query.limit,
            offset: (query.page - 1) * query.limit,
        },
        &mut conn,
    )
    .await
    .map_err(|_| ActivityError::DatabaseError.into_response())?;
    let count = ActivityRepo::count_all_for_user_id(query.user_id, &mut conn)
        .await
        .map_err(|_| ActivityError::DatabaseError.into_response())?;

    Ok(Json(GetActivityResponse {
        activity,
        _metadata: Metadata {
            total_count: Some(count),
            ..Default::default()
        },
    })
    .into_response())
}

#[derive(thiserror::Error, Debug, JsonErrorResponse)]
pub enum ActivityError {
    #[error("Database Error")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError,
}

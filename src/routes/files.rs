use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::AppState;

pub async fn avatar(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Response, Response> {
    let avatar_path = state.upload_path.join("user-avatar").join(user_id);
    let file = File::open(avatar_path)
        .await
        .map_err(|e| Response::new(Body::from(e.to_string())))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    Ok(body.into_response())
}

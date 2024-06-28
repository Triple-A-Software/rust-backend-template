use serde::Serialize;

use super::response::Metadata;

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error_message: String,
    pub _metadata: Metadata,
}

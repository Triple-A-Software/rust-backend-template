use chrono::Utc;
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_index_on_page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_index_on_page: Option<i64>,
    pub timestamp: i64,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            total_count: None,
            first_index_on_page: None,
            last_index_on_page: None,
            timestamp: Utc::now().timestamp(),
        }
    }
}

use chrono::{
    serde::{ts_milliseconds, ts_milliseconds_option},
    DateTime, Utc,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::ipnetwork::IpNetwork};
use uuid::Uuid;

pub mod auth;
pub mod implementation;
pub mod user;

pub const USER_TABLE_NAME: &str = "auth.user";
#[allow(dead_code)]
pub const TAG_TABLE_NAME: &str = "tag";
#[allow(dead_code)]
pub const ACTIVITY_TABLE_NAME: &str = "activity";
#[allow(dead_code)]
pub const SETTINGS_TABLE_NAME: &str = "settings";
#[allow(dead_code)]
pub const SESSION_TABLE_NAME: &str = "auth.session";
#[allow(dead_code)]
pub const TOKEN_TABLE_NAME: &str = "auth.token";

#[derive(Deserialize, Clone, Debug, Serialize, FromRow)]
pub struct Settings {
    pub id: String,
    pub setup_finished: bool,
}

#[derive(Deserialize, Clone, Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i32,
    pub title: String,
    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    #[serde(with = "ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
    #[serde(with = "ts_milliseconds_option")]
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum UpdateTag {
    Existing { id: i32 },
    New { label: String },
}

#[derive(FromRow, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: i32,
    pub action: String,
    pub action_by_id: Uuid,
    #[serde(with = "ts_milliseconds")]
    pub action_at: DateTime<Utc>,
    pub ip_address: Option<IpNetwork>,
    pub user_agent: Option<String>,
    pub table_name: Option<String>,
    pub item_id: Option<String>,
    /// The data before the change if it is an update (without secrets, as json)
    pub old_data: Option<String>,
    /// The data after the change if it is an update (without secrets, as json)
    pub new_data: Option<String>,
}

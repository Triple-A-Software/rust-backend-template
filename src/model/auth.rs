use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::ipnetwork::IpNetwork};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    #[default]
    Offline,
    Online,
    Away,
    DoNotDisturb,
}

#[derive(Serialize)]
pub struct UserStatusUpdate {
    pub user_id: String,
    pub new_status: UserStatus,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, Copy, Eq, PartialEq)]
pub enum Role {
    #[default]
    Admin,
    Editor,
    Author,
    Contributor,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
    English,
    #[serde(rename = "de")]
    German,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

#[derive(FromRow, Serialize)]
pub struct Session {
    pub id: i32,
    pub token_id: i32,
    pub user_agent: String,
    pub ip_address: Option<IpNetwork>,
    pub created_at: DateTime<Utc>,
}

pub struct SessionWithToken {
    pub session: Session,
    pub token: Token,
}

#[derive(Serialize)]
pub struct TokenWithSession {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub expiration: Option<DateTime<Utc>>,
    pub session: Session,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub enum TokenType {
    #[serde(rename = "password_reset")]
    PasswordReset,
    #[serde(rename = "static_access")]
    StaticAccess,
    Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: i32,
    pub name: Option<String>,
    pub token: String,
    #[serde(rename = "type")]
    pub token_type: TokenType,
    pub expiration: Option<DateTime<Utc>>,
    pub user_id: Uuid,
    pub session_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct PreferencesInput {
    pub language: Language,
    pub theme: Theme,
}

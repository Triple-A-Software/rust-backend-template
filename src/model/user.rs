use chrono::{
    serde::{ts_milliseconds, ts_milliseconds_option},
    DateTime, Utc,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use uuid::Uuid;

use super::{
    auth::{Language, Role, Theme, UserStatus},
    Tag, UpdateTag,
};

#[derive(FromRow, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(skip_serializing)]
    pub salt: Vec<u8>,
    #[serde(skip_serializing)]
    pub hash: Vec<u8>,
    pub description: Option<String>,
    pub title: Option<String>,
    pub location: Option<String>,

    pub language: Language,
    pub role: Role,
    pub theme: Theme,
    pub avatar: Option<String>,

    pub online_status: UserStatus,
    pub last_active_at: Option<DateTime<Utc>>,

    #[serde(with = "ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    #[serde(with = "ts_milliseconds_option")]
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
}

#[derive(Serialize)]
pub struct UserWithTags {
    #[serde(flatten)]
    pub user: User,
    pub tags: Vec<Tag>,
}

#[derive(Deserialize)]
pub struct UserUpdateInput {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub title: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub language: Option<Language>,
    pub role: Option<Role>,
    pub theme: Option<Theme>,
    pub tags: Vec<UpdateTag>,
}

#[derive(Default)]
pub struct UserCreateInput<'a> {
    pub email: String,
    pub salt: &'a [u8],
    pub hash: &'a [u8],
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<Role>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
}

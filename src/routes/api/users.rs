use std::{io, net::SocketAddr, path::PathBuf};

use axum::{
    extract::{ConnectInfo, Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use futures::TryStreamExt;
use macros::JsonErrorResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Acquire;
use tokio::fs::File;
use tokio_util::io::StreamReader;
use uuid::Uuid;

use crate::{
    model::{
        auth::Role,
        user::{User, UserCreateInput, UserUpdateInput, UserWithTags},
        UpdateTag, USER_TABLE_NAME,
    },
    repo::{
        activity::{ActivityEntry, ActivityRepo},
        tag::TagRepo,
        user::UserRepo,
        DatabaseListOptions, SortDirection,
    },
    service::auth::AuthService,
    utils::{error::ErrorResponse, extractors::Session, response::Metadata},
    AppState,
};

fn default_sort_by() -> String {
    "email".to_string()
}

fn validate_user_id(
    id: String,
    user: Option<&User>,
    allowed_roles: Option<&[Role]>,
) -> Result<Uuid, UserError> {
    let user_id = match id.as_str() {
        "me" => {
            if let Some(user) = user {
                if allowed_roles
                    .map(|rs| rs.contains(&user.role))
                    .unwrap_or(true)
                {
                    user.id
                } else {
                    return Err(UserError::Forbidden);
                }
            } else {
                return Err(UserError::Unauthorized);
            }
        }
        v => v
            .parse::<Uuid>()
            .map_err(|_| UserError::InvalidId(v.to_string()))?,
    };
    Ok(user_id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUsersQuery {
    /// which page, starts at 1
    page: i64,
    limit: i64,
    /// A key of the user table to sort by
    #[serde(default = "default_sort_by")]
    sort_by: String,
    #[serde(default)]
    sort_direction: SortDirection,
    /// a csv of roles to filter by (filter with 'or', not 'and')
    roles: Option<Vec<String>>,
}
pub async fn get(Query(query): Query<GetUsersQuery>, State(state): State<AppState>) -> UserResult {
    let query_roles = query.roles.map(|rs| {
        rs.iter()
            .map(|r| Role::from(r.to_string()))
            .collect::<Vec<Role>>()
    });
    let mut conn = state.db.acquire().await.unwrap();
    let db_list_options = DatabaseListOptions {
        limit: query.limit,
        offset: (query.page - 1) * query.limit,
        sort_by: query.sort_by,
        sort_direction: query.sort_direction,
    };
    let (users, count) = if let Some(roles) = query_roles {
        // TODO: filtering by roles does not work currently
        let result = UserRepo::list_for_roles(&roles, db_list_options, &mut conn)
            .await
            .map_err(|_| UserError::DatabaseError)?;
        let count = UserRepo::count_for_roles(&roles, &mut conn)
            .await
            .map_err(|_| UserError::DatabaseError)?;
        (result, count)
    } else {
        let result = UserRepo::list(db_list_options, &mut conn)
            .await
            .map_err(|_| UserError::DatabaseError)?;
        let count = UserRepo::count_all(&mut conn)
            .await
            .map_err(|_| UserError::DatabaseError)?;
        (result, count)
    };
    Ok(Json(json!({
        "users": users,
        "_metadata": Metadata {
            total_count: Some(count),
            ..Default::default()
        }
    }))
    .into_response())
}

pub async fn get_by_id(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Session(user): Session<User>,
) -> UserResult {
    let user_id = validate_user_id(id, user.as_ref(), None)?;
    let mut conn = state.db.acquire().await.unwrap();
    let user = UserRepo::get_by_id(user_id, &mut conn)
        .await
        .map_err(|_| UserError::NotFound)?;
    let tags = TagRepo::list_by_user_id(user_id, &mut conn)
        .await
        .unwrap_or(vec![]);
    Ok(Json(json!({
        "user": UserWithTags { user, tags },
        "_metadata": Metadata::default(),
    }))
    .into_response())
}

pub async fn put(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Session(user): Session<User>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Json(payload): Json<UserUpdateInput>,
) -> UserResult {
    if user.is_none() {
        return Err(UserError::Unauthorized);
    }
    let conn = &mut state.db.acquire().await.unwrap();
    let user_id = validate_user_id(id, user.as_ref(), Some(&[Role::Admin]))?;
    let user = user.unwrap();
    let mut tx = conn.begin().await.unwrap();
    let before_update = UserRepo::get_by_id(user_id, &mut tx)
        .await
        .map_err(|_| UserError::NotFound)?;
    let updated = UserRepo::update_one(user_id, payload, &mut tx)
        .await
        .map_err(|_| UserError::DatabaseError)?;
    let _ = ActivityRepo::create_one(
        ActivityEntry::Update {
            table_name: USER_TABLE_NAME.to_string(),
            item_id: user_id.to_string(),
            ip_address: Some(addr.ip().into()),
            user_agent: Some(user_agent.to_string()),
            old_data: serde_json::to_string(&before_update).unwrap(),
            new_data: serde_json::to_string(&updated).unwrap(),
            action_by_id: user.id,
        },
        &mut tx,
    )
    .await;
    tx.commit().await.unwrap();
    Ok(Json(json!({
        "updated": updated,
        "_metadata": Metadata::default(),
    }))
    .into_response())
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Session(current_user): Session<User>,
) -> UserResult {
    if let Some(current_user) = current_user {
        let conn = &mut state.db.acquire().await.unwrap();
        UserRepo::delete_one(id, current_user.id, conn)
            .await
            .map_err(|_| UserError::DatabaseError)?;
        let _ = ActivityRepo::create_one(
            ActivityEntry::Delete {
                ip_address: Some(addr.ip().into()),
                user_agent: Some(user_agent.to_string()),
                action_by_id: current_user.id,
                table_name: USER_TABLE_NAME.to_string(),
                item_id: id.to_string(),
            },
            conn,
        )
        .await;
        return Ok(Json(json!({
            "deleted": {
                "id": id,
            },
            "_metadata": Metadata::default()
        }))
        .into_response());
    }
    Err(UserError::Unauthorized)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserPostBody {
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    location: Option<String>,
    title: Option<String>,
    description: Option<String>,
    password: String,
    tags: Vec<UpdateTag>,
    role: Option<String>,
}
#[derive(Serialize)]
pub struct UserPostResponse {
    created: User,
    _metadata: Metadata,
}
pub async fn post(
    State(state): State<AppState>,
    Session(session_user): Session<User>,
    Json(body): Json<UserPostBody>,
) -> UserResult {
    if let Some(current_user) = session_user {
        let conn = &mut state.db.acquire().await.unwrap();
        let created = AuthService::create_user(
            UserCreateInput {
                email: body.email,
                first_name: body.first_name,
                last_name: body.last_name,
                role: body.role.map(|r| r.into()),
                location: body.location,
                description: body.description,
                title: body.title,
                ..Default::default()
            },
            body.tags,
            body.password,
            current_user.id,
            conn,
        )
        .await
        .map_err(|_| UserError::DatabaseError)?;
        Ok(Json(UserPostResponse {
            created,
            _metadata: Metadata::default(),
        })
        .into_response())
    } else {
        Err(UserError::Unauthorized)
    }
}

#[derive(Deserialize)]
pub struct UserSearchQuery {
    q: String,
}
#[derive(Serialize, Default)]
pub struct UserSearchResponse {
    users: Vec<User>,
    _metadata: Metadata,
}
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<UserSearchQuery>,
) -> UserResult {
    let conn = &mut state.db.acquire().await.unwrap();
    let result = UserRepo::search(query.q, conn)
        .await
        .map_err(|_| UserError::DatabaseError)?;
    Ok(Json(UserSearchResponse {
        users: result,
        ..Default::default()
    })
    .into_response())
}

#[derive(Deserialize)]
pub struct UpdatePasswordPayload {
    current_password: String,
    new_password: String,
    confirm_new_password: String,
}
pub async fn update_password(
    Session(user): Session<User>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Json(payload): Json<UpdatePasswordPayload>,
) -> UserResult {
    if let Some(user) = user {
        let conn = &mut state.db.acquire().await.unwrap();
        if payload.new_password != payload.confirm_new_password {
            return Err(UserError::PasswordsDontMatch);
        }
        let updated = AuthService::update_password(
            user.id,
            payload.current_password,
            payload.new_password,
            conn,
        )
        .await
        .map_err(|e| match e {
            crate::service::auth::AuthError::InvalidCredentials => UserError::InvalidCredentials,
            e => UserError::InternalServerError(e.to_string()),
        })?;
        let _ = ActivityRepo::create_one(
            ActivityEntry::PasswordChange {
                item_id: updated.id,
                ip_address: Some(addr.ip().into()),
                user_agent: Some(user_agent.to_string()),
                action_by_id: user.id,
            },
            conn,
        )
        .await;
        Ok(Json(json!({
            "updated": updated,
            "_metadata": Metadata::default(),
        }))
        .into_response())
    } else {
        Err(UserError::Unauthorized)
    }
}

pub async fn update_avatar(
    Session(user): Session<User>,
    State(state): State<AppState>,
    mut body: Multipart,
) -> UserResult {
    if let Some(user) = user {
        let upload_destination = state.upload_path.join("user-avatar");
        std::fs::create_dir_all(&upload_destination)
            .map_err(|_| UserError::AvatarFileWriteError)?;
        let upload_destination = upload_destination.join(user.id.to_string());
        while let Some(field) = body.next_field().await.unwrap() {
            let conn = &mut state.db.acquire().await.unwrap();
            let name = field.name().unwrap();
            if name != "avatar" {
                continue;
            }
            let mime = field.content_type().unwrap();
            if !mime.starts_with("image") {
                return Err(UserError::WrongAvatarFileType);
            }
            let field_with_io_err = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
            let mut stream = StreamReader::new(field_with_io_err);
            dbg!(&upload_destination);
            let mut file = File::create(&upload_destination)
                .await
                .map_err(|_| UserError::AvatarFileWriteError)?;
            tokio::io::copy(&mut stream, &mut file)
                .await
                .map_err(|_| UserError::AvatarFileWriteError)?;
            let http_path = PathBuf::from("files")
                .join("avatar")
                .join(user.id.to_string());
            UserRepo::update_avatar_path(user.id, Some(&http_path), conn)
                .await
                .map_err(|_| UserError::DatabaseError)?;
            return Ok(Json(json!({
                "url": http_path,
                "_metadata": Metadata::default(),
            }))
            .into_response());
        }
        Err(UserError::MissingAvatarField)
    } else {
        Err(UserError::Unauthorized)
    }
}

pub async fn delete_avatar(
    Session(user): Session<User>,
    State(state): State<AppState>,
) -> UserResult {
    if let Some(user) = user {
        let avatar_path = state
            .upload_path
            .join("user-avatar")
            .join(user.id.to_string());
        let conn = &mut state.db.acquire().await.unwrap();
        tokio::fs::remove_file(avatar_path)
            .await
            .map_err(|_| UserError::AvatarFileWriteError)?;
        UserRepo::update_avatar_path(user.id, None, conn)
            .await
            .map_err(|_| UserError::DatabaseError)?;
        Ok(Json(json!({})).into_response())
    } else {
        Err(UserError::Unauthorized)
    }
}

#[derive(thiserror::Error, Debug, JsonErrorResponse)]
pub enum UserError {
    #[error("Database error")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError,

    #[error("User not found")]
    #[status_code(StatusCode::NOT_FOUND)]
    NotFound,

    #[error("Unauthorized")]
    #[status_code(StatusCode::UNAUTHORIZED)]
    Unauthorized,

    #[error("Forbidden")]
    #[status_code(StatusCode::FORBIDDEN)]
    Forbidden,

    #[error("Invalid id {0}")]
    #[status_code(StatusCode::BAD_REQUEST)]
    InvalidId(String),

    #[error("Passwords don't match")]
    #[status_code(StatusCode::BAD_REQUEST)]
    PasswordsDontMatch,

    #[error("Internal server error: {0}")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    InternalServerError(String),

    #[error("Invalid credentials")]
    #[status_code(StatusCode::BAD_REQUEST)]
    InvalidCredentials,

    #[error("Wrong avatar file type. Expected 'image/...'")]
    #[status_code(StatusCode::BAD_REQUEST)]
    WrongAvatarFileType,

    #[error("Error writing avatar file")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    AvatarFileWriteError,

    #[error("Missing avatar field in multipart")]
    #[status_code(StatusCode::BAD_REQUEST)]
    MissingAvatarField,
}

pub type UserResult = Result<Response, UserError>;

use std::{num::NonZeroU32, str::FromStr};

use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use macros::JsonErrorResponse;
use rand::Rng;
use ring::{digest, pbkdf2};
use sqlx::{types::ipnetwork::IpNetwork, Acquire, PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    model::{
        auth::{SessionWithToken, TokenType},
        user::{User, UserCreateInput},
        UpdateTag,
    },
    repo::{session::SessionRepo, tag::TagRepo, token::TokenRepo, user::UserRepo},
    utils::error::ErrorResponse,
};

#[derive(Clone)]
pub struct AuthService {}

impl AuthService {
    pub async fn create_user(
        data: UserCreateInput<'_>,
        tags: Vec<UpdateTag>,
        password: String,
        current_user_id: uuid::Uuid,
        db: &mut PgConnection,
    ) -> AuthResult<User> {
        let (salt, hash) = hash_password(password, None);
        let mut tx = db.begin().await.unwrap();
        let created = UserRepo::create_one(
            UserCreateInput {
                email: data.email,
                salt: &salt,
                hash: &hash,
                first_name: data.first_name,
                last_name: data.last_name,
                role: data.role,
                location: data.location,
                description: data.description,
                title: data.title,
            },
            current_user_id,
            &mut tx,
        )
        .await
        .map_err(|_| AuthError::DatabaseError)?;
        let tags = TagRepo::create_missing(tags, created.id, &mut tx)
            .await
            .map_err(|_| AuthError::DatabaseError)?;
        UserRepo::update_tags(
            created.id,
            tags.into_iter().map(|t| t.id).collect(),
            &mut tx,
        )
        .await
        .map_err(|_| AuthError::DatabaseError)?;
        tx.commit().await.unwrap();
        Ok(created)
    }

    pub async fn update_password(
        user_id: Uuid,
        current_password: String,
        new_password: String,
        db: &mut PgConnection,
    ) -> AuthResult<User> {
        let from_db = UserRepo::get_by_id(user_id, db)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;
        let (_, hash) = hash_password(current_password, Some(from_db.salt.clone()));
        if hash != *from_db.hash {
            return Err(AuthError::InvalidCredentials);
        }
        let (salt, hash) = hash_password(new_password, Some(from_db.salt));
        let updated = UserRepo::update_hash_salt(user_id, &hash, &salt, db)
            .await
            .map_err(|e| AuthError::InternalServerError(e.to_string()))?;
        Ok(updated)
    }

    pub async fn reset_password(
        token: &str,
        new_password: String,
        db: &mut PgConnection,
    ) -> AuthResult<bool> {
        let token = TokenRepo::get_by_token(token, db)
            .await
            .map_err(|_| AuthError::DatabaseError)?;
        let user = UserRepo::get_by_id(token.user_id, db)
            .await
            .map_err(|_| AuthError::DatabaseError)?;
        let (salt, hash) = hash_password(new_password, Some(user.salt));
        let _ = UserRepo::update_hash_salt(user.id, &hash, &salt, db)
            .await
            .map_err(|e| AuthError::InternalServerError(e.to_string()))?;
        let _ = TokenRepo::delete_by_id(token.id, token.user_id, db).await;
        Ok(true)
    }

    pub async fn login(
        email: String,
        password: String,
        ip: Option<IpNetwork>,
        user_agent: String,
        db: &mut PgConnection,
    ) -> AuthResult<(User, SessionWithToken)> {
        let user = UserRepo::get_by_email(email, db)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;
        // compare
        if pbkdf2::verify(
            PBKDF2_ALG,
            NonZeroU32::new(600_000).unwrap(),
            &user.salt,
            password.as_bytes(),
            user.hash.as_slice(),
        )
        .is_ok()
        {
            let session_with_token = SessionRepo::create_one_with_token(&user, ip, user_agent, db)
                .await
                .map_err(|e| {
                    tracing::error!("{:?}", e);
                    AuthError::SessionCreateFailed
                })?;
            Ok((user, session_with_token))
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    pub async fn check_password_reset_token(token: &str, db: &mut PgConnection) -> bool {
        let token = match TokenRepo::get_by_token(token, db).await {
            Ok(t) => t,
            Err(_) => return false,
        };
        if let Some(expiration) = token.expiration {
            if expiration >= Utc::now() && matches!(token.token_type, TokenType::PasswordReset) {
                return true;
            } else if expiration < Utc::now() {
                let _ = TokenRepo::delete_by_id(token.id, token.user_id, db).await;
                return false;
            }
        }
        false
    }

    pub async fn logout(token: String, db: &PgPool) -> AuthResult<()> {
        SessionRepo::delete_with_token(
            sqlx::types::Uuid::from_str(&token).unwrap().to_string(),
            &mut db.acquire().await.unwrap(),
        )
        .await
        .map_err(|e| AuthError::InternalServerError(e.to_string()))?;
        Ok(())
    }
}

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA512;
pub const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
pub const SALT_LEN: usize = 32;
pub type Credential = [u8; CREDENTIAL_LEN];

fn hash_password(password: String, salt: Option<Vec<u8>>) -> (Vec<u8>, Credential) {
    let generated = rand::thread_rng().gen::<[u8; SALT_LEN]>().to_vec();
    let salt = salt.unwrap_or(generated);
    let mut hash: Credential = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        PBKDF2_ALG,
        NonZeroU32::new(600_000).unwrap(),
        &salt,
        password.as_bytes(),
        &mut hash,
    );
    (salt, hash)
}

#[derive(thiserror::Error, Debug, JsonErrorResponse)]
pub enum AuthError {
    #[error("Invalid credentials")]
    #[status_code(StatusCode::UNAUTHORIZED)]
    InvalidCredentials,

    #[error("Internal server error: {0}")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    InternalServerError(String),

    #[error("Creating session failed")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    SessionCreateFailed,

    #[error("Database error")]
    #[status_code(StatusCode::INTERNAL_SERVER_ERROR)]
    DatabaseError,
}

pub type AuthResult<T> = Result<T, AuthError>;

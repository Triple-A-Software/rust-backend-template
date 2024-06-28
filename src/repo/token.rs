use crate::utils;
use chrono::Utc;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::model::auth::{Token, TokenType};

#[derive(Clone)]
pub struct TokenRepo {}

impl TokenRepo {
    pub async fn delete_one_by_token<'c>(
        token: String,
        db: &mut PgConnection,
    ) -> sqlx::Result<Token> {
        sqlx::query_as!(
            Token,
            r#"DELETE FROM auth.token WHERE token = $1 RETURNING *"#,
            token,
        )
        .fetch_one(db)
        .await
    }

    pub async fn get_by_token(token: &str, db: &mut PgConnection) -> sqlx::Result<Token> {
        sqlx::query_as!(Token, r#"SELECT * FROM auth.token WHERE token = $1"#, token)
            .fetch_one(db)
            .await
    }

    pub async fn list_for_user(user_id: Uuid, db: &mut PgConnection) -> sqlx::Result<Vec<Token>> {
        sqlx::query_as!(
            Token,
            r#"SELECT * FROM auth.token WHERE user_id = $1 and token_type = 'static_access'"#,
            user_id
        )
        .fetch_all(db)
        .await
    }

    pub async fn delete_by_id(id: i32, user_id: Uuid, db: &mut PgConnection) -> sqlx::Result<i32> {
        Ok(sqlx::query!(
            r#"DELETE FROM auth.token WHERE id = $1 and user_id = $2 RETURNING id"#,
            id,
            user_id
        )
        .fetch_one(db)
        .await?
        .id)
    }

    pub async fn create_one_session_token(
        user_id: Uuid,
        db: &mut PgConnection,
    ) -> sqlx::Result<Token> {
        sqlx::query_as!(
            Token,
            r#"INSERT INTO auth.token 
                (user_id, token_type, expiration, token) 
            VALUES ($1, $2, $3, $4) 
            RETURNING *"#,
            user_id,
            String::from(TokenType::Session),
            Utc::now() + chrono::Duration::days(30),
            utils::auth::generate_session_token(),
        )
        .fetch_one(db)
        .await
    }

    pub async fn create_one_password_reset_token(
        user_id: Uuid,
        db: &mut PgConnection,
    ) -> sqlx::Result<Token> {
        sqlx::query_as!(
            Token,
            r#"INSERT INTO auth.token 
                (user_id, token_type, expiration, token) 
            VALUES ($1, $2, $3, $4) 
            RETURNING *"#,
            user_id,
            String::from(TokenType::PasswordReset),
            Utc::now() + chrono::Duration::minutes(30),
            utils::auth::generate_session_token()
        )
        .fetch_one(db)
        .await
    }

    pub async fn create_one_access_token(
        user_id: Uuid,
        name: String,
        db: &mut PgConnection,
    ) -> sqlx::Result<Token> {
        sqlx::query_as!(
            Token,
            r#"INSERT INTO auth.token 
                (user_id, token_type, token, name)
            VALUES ($1, $2, $3, $4)
            RETURNING *"#,
            user_id,
            String::from(TokenType::StaticAccess),
            utils::auth::generate_session_token(),
            name,
        )
        .fetch_one(db)
        .await
    }
}

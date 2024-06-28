use sqlx::{types::ipnetwork::IpNetwork, Acquire, PgConnection};
use uuid::Uuid;

use crate::model::{
    auth::{Session, SessionWithToken, TokenWithSession},
    user::User,
};

use super::token::TokenRepo;

#[derive(Clone)]
pub struct SessionRepo {}

impl SessionRepo {
    pub async fn create_one_with_token(
        user: &User,
        ip: Option<IpNetwork>,
        user_agent: String,
        db: &mut PgConnection,
    ) -> sqlx::Result<SessionWithToken> {
        let mut tx = db.begin().await?;
        let token = TokenRepo::create_one_session_token(user.id, &mut tx).await?;
        let session = sqlx::query_as!(
            Session,
            "INSERT INTO auth.session (token_id, ip_address, user_agent) VALUES ($1, $2, $3) RETURNING *",
            &token.id,
            ip,
            user_agent,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(SessionWithToken { session, token })
    }

    pub async fn delete_with_token(token: String, db: &mut PgConnection) -> sqlx::Result<()> {
        let mut tx = db.begin().await?;
        let deleted_token = TokenRepo::delete_one_by_token(token, &mut tx)
            .await
            .unwrap();

        sqlx::query!(
            "DELETE FROM auth.session WHERE token_id = $1",
            deleted_token.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn delete_by_id_for_user(
        id: i32,
        user_id: Uuid,
        db: &mut PgConnection,
    ) -> sqlx::Result<()> {
        let mut tx = db.begin().await?;
        let deleted = sqlx::query!(
            "DELETE FROM auth.session WHERE id = $1 RETURNING token_id",
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        let _ = TokenRepo::delete_by_id(deleted.token_id, user_id, &mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn get_sessions_for_user(
        user_id: Uuid,
        db: &mut PgConnection,
    ) -> sqlx::Result<Vec<TokenWithSession>> {
        let result = sqlx::query!(
            r#"SELECT 
                auth.session.id as session_id,
                auth.session.user_agent as session_user_agent,
                auth.session.ip_address as session_ip_address,
                auth.session.created_at as session_created_at,
                auth.token.id as token_id,
                auth.token.created_at as token_created_at,
                auth.token.expiration as token_expiration
            from auth.session 
                LEFT JOIN auth.token ON auth.session.token_id = auth.token.id 
                WHERE auth.token.user_id = $1"#,
            user_id
        )
        .fetch_all(db)
        .await?;
        Ok(result
            .into_iter()
            .map(|s| TokenWithSession {
                id: s.token_id,
                created_at: s.token_created_at,
                expiration: s.token_expiration,
                session: Session {
                    id: s.session_id,
                    token_id: s.token_id,
                    user_agent: s.session_user_agent,
                    ip_address: Some(s.session_ip_address),
                    created_at: s.session_created_at,
                },
            })
            .collect())
    }
}

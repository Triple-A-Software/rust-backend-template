use std::path::PathBuf;

use chrono::{DateTime, Utc};
use sqlx::{Acquire, PgConnection};
use uuid::Uuid;

use crate::model::{
    auth::{PreferencesInput, Role, UserStatus},
    user::{User, UserCreateInput, UserUpdateInput},
};

use super::{tag::TagRepo, DatabaseListOptions, SortDirection};

#[derive(Clone)]
pub struct UserRepo {}

impl UserRepo {
    pub async fn create_one(
        new_user: UserCreateInput<'_>,
        current_user_id: Uuid,
        db: &mut PgConnection,
    ) -> sqlx::Result<User> {
        sqlx::query_as!(
            User,
            r#"INSERT INTO auth.user (email, first_name, last_name, salt, hash, created_by, updated_by, role) VALUES ($1, $2, $3, $4, $5, $6, $6, $7) returning *"#,
            new_user.email,
            new_user.first_name,
            new_user.last_name,
            new_user.salt,
            new_user.hash,
            current_user_id,
            String::from(new_user.role.unwrap_or(Role::Author)),
        )
        .fetch_one(db)
        .await
    }

    pub async fn get_by_email(email: String, db: &mut PgConnection) -> sqlx::Result<User> {
        sqlx::query_as!(User, r#"SELECT * FROM auth.user WHERE email = $1"#, email,)
            .fetch_one(db)
            .await
    }

    pub async fn get_by_id(id: Uuid, db: &mut PgConnection) -> sqlx::Result<User> {
        sqlx::query_as!(User, r#"SELECT * FROM auth.user WHERE id = $1"#, id,)
            .fetch_one(db)
            .await
    }

    pub async fn list(
        options: DatabaseListOptions,
        db: &mut PgConnection,
    ) -> sqlx::Result<Vec<User>> {
        match options.sort_direction {
            SortDirection::Asc => {
                sqlx::query_as!(
                    User,
                    r#"SELECT * FROM auth.user ORDER BY $1 ASC LIMIT $2 OFFSET $3"#,
                    options.sort_by,
                    options.limit,
                    options.offset
                )
                .fetch_all(db)
                .await
            }
            SortDirection::Desc => {
                sqlx::query_as!(
                    User,
                    r#"SELECT * FROM auth.user ORDER BY $1 DESC LIMIT $2 OFFSET $3"#,
                    options.sort_by,
                    options.limit,
                    options.offset
                )
                .fetch_all(db)
                .await
            }
        }
    }

    pub async fn count_all(db: &mut PgConnection) -> sqlx::Result<i64> {
        let result = sqlx::query!(r#"SELECT COUNT(*) FROM auth.user"#)
            .fetch_one(db)
            .await;
        result.map(|r| r.count.unwrap_or(0))
    }

    pub async fn list_for_roles(
        roles: &[Role],
        options: DatabaseListOptions,
        db: &mut PgConnection,
    ) -> sqlx::Result<Vec<User>> {
        let roles = format!(
            "'{}'",
            roles
                .iter()
                .map(String::from)
                .collect::<Vec<_>>()
                .join("','")
        );
        match options.sort_direction {
            SortDirection::Asc => {
                sqlx::query_as!(
                    User,
                    r#"SELECT * FROM auth.user WHERE auth.user.role IN ($1) ORDER BY $2 ASC LIMIT $3 OFFSET $4"#,
                    roles,
                    options.sort_by,
                    options.limit,
                    options.offset
                )
                .fetch_all(db)
                .await
            }
            SortDirection::Desc => {
                sqlx::query_as!(
                    User,
                    r#"SELECT * FROM auth.user WHERE auth.user.role IN ($1) ORDER BY $2 DESC LIMIT $3 OFFSET $4"#,
                    roles,
                    options.sort_by,
                    options.limit,
                    options.offset
                )
                .fetch_all(db)
                .await
            },
        }
    }

    pub async fn count_for_roles(roles: &[Role], db: &mut PgConnection) -> sqlx::Result<i64> {
        let roles = format!(
            "'{}'",
            roles
                .iter()
                .map(String::from)
                .collect::<Vec<_>>()
                .join("','")
        );
        let result = sqlx::query!(
            r#"SELECT COUNT(*) FROM auth.user WHERE auth.user.role in ($1)"#,
            roles
        )
        .fetch_one(db)
        .await;
        result.map(|r| r.count.unwrap_or(0))
    }

    pub async fn get_from_session_token(token: &str, db: &mut PgConnection) -> sqlx::Result<User> {
        sqlx::query_as!(
            User,
            r#"SELECT auth.user.*
            FROM auth.user 
            LEFT JOIN auth.token ON auth.user.id = auth.token.user_id 
                WHERE token = $1"#,
            token
        )
        .fetch_one(db)
        .await
    }

    pub async fn update_status(
        id: Uuid,
        online_status: UserStatus,
        last_active_at: Option<DateTime<Utc>>,
        db: &mut PgConnection,
    ) -> sqlx::Result<sqlx::postgres::PgQueryResult> {
        if let Some(last_active_at) = last_active_at {
            sqlx::query!(
                r#"UPDATE auth.user SET online_status = $1, last_active_at = $2 WHERE id = $3"#,
                String::from(online_status),
                last_active_at,
                id
            )
            .execute(db)
            .await
        } else {
            sqlx::query!(
                r#"UPDATE auth.user SET online_status = $1 WHERE id = $2"#,
                String::from(online_status),
                id
            )
            .execute(db)
            .await
        }
    }

    pub async fn update_preferences(
        id: Uuid,
        preferences: PreferencesInput,
        db: &mut PgConnection,
    ) -> sqlx::Result<User> {
        sqlx::query_as!(
            User,
            r#"UPDATE auth.user SET theme = $1, language = $2 WHERE id = $3 RETURNING *"#,
            String::from(preferences.theme),
            String::from(preferences.language),
            id
        )
        .fetch_one(db)
        .await
    }

    pub async fn update_avatar_path(
        user_id: Uuid,
        http_path: Option<&PathBuf>,
        db: &mut PgConnection,
    ) -> sqlx::Result<User> {
        let http_path = http_path.as_ref().and_then(|p| p.to_str());
        sqlx::query_as!(
            User,
            r#"UPDATE auth.user SET avatar = $2 WHERE id = $1 RETURNING *"#,
            user_id,
            http_path
        )
        .fetch_one(db)
        .await
    }

    pub async fn update_hash_salt(
        id: Uuid,
        hash: &[u8],
        salt: &[u8],
        db: &mut PgConnection,
    ) -> sqlx::Result<User> {
        sqlx::query_as!(
            User,
            r#"UPDATE auth.user SET hash = $1, salt = $2 WHERE id = $3 RETURNING *"#,
            hash,
            salt,
            id,
        )
        .fetch_one(db)
        .await
    }

    pub async fn update_one(
        id: Uuid,
        data: UserUpdateInput,
        db: &mut PgConnection,
    ) -> sqlx::Result<User> {
        let mut tx = db.begin().await?;
        let selected = sqlx::query_as!(User, r#"SELECT * from auth.user WHERE id = $1"#, id)
            .fetch_one(&mut *tx)
            .await?;
        let tags = TagRepo::create_missing(data.tags, selected.id, &mut tx).await?;
        let result = sqlx::query_as!(
            User,
            r#"UPDATE auth.user 
            SET
                email = $1,
                first_name = $2,
                last_name = $3,
                title = $4,
                location = $5,
                description = $6,
                language = $7,
                role = $8,
                theme = $9
            WHERE id = $10 RETURNING *"#,
            data.email.unwrap_or(selected.email),
            data.first_name.or(selected.first_name),
            data.last_name.or(selected.last_name),
            data.title.or(selected.title),
            data.location.or(selected.location),
            data.description.or(selected.description),
            String::from(data.language.unwrap_or(selected.language)),
            String::from(data.role.unwrap_or(selected.role)),
            String::from(data.theme.unwrap_or(selected.theme)),
            id,
        )
        .fetch_one(&mut *tx)
        .await?;
        UserRepo::update_tags(
            selected.id,
            tags.into_iter().map(|t| t.id).collect(),
            &mut tx,
        )
        .await?;
        tx.commit().await?;
        Ok(result)
    }

    pub async fn update_tags(
        id: Uuid,
        tag_ids: Vec<i32>,
        db: &mut PgConnection,
    ) -> sqlx::Result<()> {
        let mut tx = db.begin().await.unwrap();
        sqlx::query!(r#"DELETE FROM auth.user_to_tag WHERE user_id = $1"#, id)
            .execute(&mut *tx)
            .await?;
        if !tag_ids.is_empty() {
            for tag_id in tag_ids {
                sqlx::query!(
                    r#"INSERT INTO auth.user_to_tag (tag_id, user_id) VALUES ($1, $2)"#,
                    tag_id,
                    id
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await
    }

    pub async fn delete_one(
        id: Uuid,
        current_user_id: Uuid,
        db: &mut PgConnection,
    ) -> sqlx::Result<sqlx::postgres::PgQueryResult> {
        sqlx::query!(
            r#"UPDATE auth.user SET deleted_at = $1, deleted_by = $2 WHERE id = $3"#,
            Utc::now(),
            current_user_id,
            id
        )
        .execute(db)
        .await
    }

    pub async fn search(term: String, db: &mut PgConnection) -> sqlx::Result<Vec<User>> {
        sqlx::query_as!(
            User,
            r#"SELECT * FROM auth.user WHERE email ilike $1 OR first_name ilike $1 OR last_name ilike $1"#,
            format!("%{term}%")
        ).fetch_all(db).await
    }
}

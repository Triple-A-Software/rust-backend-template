use sqlx::{types::ipnetwork::IpNetwork, PgConnection};
use uuid::Uuid;

use crate::model::Activity;

use super::DatabasePagination;

#[derive(Clone)]
pub struct ActivityRepo;

#[allow(dead_code)]
pub enum ActivityEntry {
    Update {
        table_name: String,
        item_id: String,
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        old_data: String,
        new_data: String,
        action_by_id: Uuid,
    },
    PasswordResetRequest {
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        /// The id of the user whose password wants be changed
        item_id: Uuid,
    },
    PasswordReset {
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        /// The id of the user whose password was changed
        item_id: Uuid,
    },
    PasswordChange {
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        /// The id of the user whose password was changed
        item_id: Uuid,
        /// the id of the current logged in user
        action_by_id: Uuid,
    },
    Login {
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        /// The id of the user logging in
        action_by_id: Uuid,
    },
    Logout {
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        /// The id of the user logging out
        action_by_id: Uuid,
    },
    Delete {
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        action_by_id: Uuid,
        table_name: String,
        item_id: String,
    },
    HardDelete {
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        action_by_id: Uuid,
        table_name: String,
        item_id: String,
    },
    Create {
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
        action_by_id: Uuid,
        table_name: String,
        item_id: String,
        /// The data of the created item without secrets and in json format
        new_data: String,
    },
    Comment,
}

impl ActivityRepo {
    pub async fn create_one(data: ActivityEntry, db: &mut PgConnection) -> sqlx::Result<Activity> {
        match data {
            ActivityEntry::Update {
                table_name,
                item_id,
                ip_address,
                user_agent,
                old_data,
                new_data,
                action_by_id,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, action_by_id, ip_address, user_agent, table_name, item_id, old_data, new_data) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#,
                    "update".to_string(),
                    action_by_id,
                    ip_address,
                    user_agent,
                    table_name,
                    item_id,
                    old_data,
                    new_data,
                )
                .fetch_one(db)
                .await
            }
            ActivityEntry::PasswordResetRequest {
                ip_address,
                user_agent,
                item_id,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, ip_address, user_agent, item_id) VALUES ($1, $2, $3, $4) RETURNING *"#,
                    "password_reset_request".to_string(),
                    ip_address,
                    user_agent,
                    item_id.to_string(),
                )
                .fetch_one(db)
                .await
            },
            ActivityEntry::PasswordReset {
                ip_address,
                user_agent,
                item_id,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, ip_address, user_agent, item_id) VALUES ($1, $2, $3, $4) RETURNING *"#,
                    "password_reset".to_string(),
                    ip_address,
                    user_agent,
                    item_id.to_string(),
                )
                .fetch_one(db)
                .await
            },
            ActivityEntry::PasswordChange {
                ip_address,
                user_agent,
                item_id,
                action_by_id,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, action_by_id, ip_address, user_agent, item_id) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
                    "password_change".to_string(),
                    action_by_id,
                    ip_address,
                    user_agent,
                    item_id.to_string(),
                )
                .fetch_one(db)
                .await
            },
            ActivityEntry::Login {
                ip_address,
                user_agent,
                action_by_id,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, action_by_id, ip_address, user_agent) VALUES ($1, $2, $3, $4) RETURNING *"#,
                    "login".to_string(),
                    action_by_id,
                    ip_address,
                    user_agent,
                )
                .fetch_one(db)
                .await
            },
            ActivityEntry::Logout {
                ip_address,
                user_agent,
                action_by_id,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, action_by_id, ip_address, user_agent) VALUES ($1, $2, $3, $4) RETURNING *"#,
                    "logout".to_string(),
                    action_by_id,
                    ip_address,
                    user_agent,
                )
                .fetch_one(db)
                .await
            },
            ActivityEntry::Delete {
                ip_address,
                user_agent,
                action_by_id,
                table_name,
                item_id,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, action_by_id, ip_address, user_agent, table_name, item_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
                    "delete".to_string(),
                    action_by_id,
                    ip_address,
                    user_agent,
                    table_name,
                    item_id,
                )
                .fetch_one(db)
                .await
            },
            ActivityEntry::HardDelete {
                ip_address,
                user_agent,
                action_by_id,
                table_name,
                item_id,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, action_by_id, ip_address, user_agent, table_name, item_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
                    "hard_delete".to_string(),
                    action_by_id,
                    ip_address,
                    user_agent,
                    table_name,
                    item_id,
                )
                .fetch_one(db)
                .await
            },
            ActivityEntry::Create {
                ip_address,
                user_agent,
                action_by_id,
                table_name,
                item_id,
                new_data,
            } => {
                sqlx::query_as!(
                    Activity,
                    r#"INSERT INTO activity (action, action_by_id, ip_address, user_agent, table_name, item_id, new_data) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#,
                    "update".to_string(),
                    action_by_id,
                    ip_address,
                    user_agent,
                    table_name,
                    item_id,
                    new_data,
                )
                .fetch_one(db)
                .await
            },
            ActivityEntry::Comment => todo!(),
        }
    }

    pub async fn list_all_for_user_id(
        user_id: Uuid,
        options: DatabasePagination,
        db: &mut PgConnection,
    ) -> sqlx::Result<Vec<Activity>> {
        sqlx::query_as!(
            Activity,
            r#"SELECT * from activity WHERE action_by_id = $1 ORDER BY action_at DESC LIMIT $2 OFFSET $3"#,
            user_id,
            options.limit,
            options.offset,
        )
        .fetch_all(db)
        .await
    }

    pub async fn count_all_for_user_id(user_id: Uuid, db: &mut PgConnection) -> sqlx::Result<i64> {
        let result = sqlx::query!(
            r#"SELECT COUNT(*) FROM activity WHERE action_by_id = $1"#,
            user_id
        )
        .fetch_one(db)
        .await;
        result.map(|r| r.count.unwrap_or(0))
    }
}

use sqlx::PgConnection;

use crate::model::Settings;

#[derive(Clone)]
pub struct SettingsRepo {}

impl SettingsRepo {
    pub async fn upsert(settings: Settings, db: &mut PgConnection) -> sqlx::Result<Settings> {
        sqlx::query_as!(
            Settings,
            r#"INSERT INTO settings (setup_finished) VALUES ($1) ON CONFLICT (id) DO UPDATE SET setup_finished = $1 RETURNING *"#,
            settings.setup_finished
        )
        .fetch_one(db)
        .await
    }

    pub async fn get(db: &mut PgConnection) -> sqlx::Result<Settings> {
        sqlx::query_as!(Settings, "SELECT * FROM settings where id = 'settings'")
            .fetch_one(db)
            .await
    }
}

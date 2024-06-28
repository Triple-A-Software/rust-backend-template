use sqlx::PgConnection;

use crate::{model::Settings, repo::settings::SettingsRepo};

#[derive(Clone)]
pub struct SetupService {}

impl SetupService {
    pub async fn finish_setup(db: &mut PgConnection) -> Result<Settings, sqlx::Error> {
        SettingsRepo::upsert(
            Settings {
                setup_finished: true,
                ..Default::default()
            },
            db,
        )
        .await
    }

    pub async fn is_setup_finished(db: &mut PgConnection) -> Result<bool, sqlx::Error> {
        let settings = match SettingsRepo::get(db).await {
            Ok(settings) => settings,
            Err(_) => Settings::default(),
        };
        Ok(settings.setup_finished)
    }
}

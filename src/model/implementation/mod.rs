use super::Settings;

pub mod auth;

impl Default for Settings {
    fn default() -> Self {
        Self {
            id: "settings".to_string(),
            setup_finished: false,
        }
    }
}

use crate::model::auth::{Language, Role, Theme, TokenType, UserStatus};

impl From<TokenType> for String {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::PasswordReset => "password_reset".to_string(),
            TokenType::StaticAccess => "static_access".to_string(),
            TokenType::Session => "session".to_string(),
        }
    }
}

impl From<String> for UserStatus {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "offline" => Self::Offline,
            "online" => Self::Online,
            "away" => Self::Away,
            "do_not_disturb" => Self::DoNotDisturb,
            _ => Default::default(),
        }
    }
}

impl From<UserStatus> for String {
    fn from(value: UserStatus) -> Self {
        match value {
            UserStatus::Offline => "offline".to_string(),
            UserStatus::Online => "online".to_string(),
            UserStatus::Away => "away".to_string(),
            UserStatus::DoNotDisturb => "do_not_disturb".to_string(),
        }
    }
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "admin" => Self::Admin,
            "editor" => Self::Editor,
            "author" => Self::Author,
            "contributor" => Self::Contributor,
            _ => Default::default(),
        }
    }
}

impl From<Role> for String {
    fn from(value: Role) -> Self {
        match value {
            Role::Admin => "admin".to_string(),
            Role::Editor => "editor".to_string(),
            Role::Author => "author".to_string(),
            Role::Contributor => "contributor".to_string(),
        }
    }
}

impl From<&Role> for String {
    fn from(value: &Role) -> Self {
        String::from(*value)
    }
}

impl From<String> for Language {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "en" => Self::English,
            "de" => Self::German,
            _ => Default::default(),
        }
    }
}

impl From<Language> for String {
    fn from(value: Language) -> Self {
        match value {
            Language::English => "en".to_string(),
            Language::German => "de".to_string(),
        }
    }
}

impl From<String> for Theme {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Default::default(),
        }
    }
}

impl From<Theme> for String {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Light => "light".to_string(),
            Theme::Dark => "dark".to_string(),
        }
    }
}

impl From<String> for TokenType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "password_reset" => Self::PasswordReset,
            "static_access" => Self::StaticAccess,
            "session" => Self::Session,
            _ => Self::Session,
        }
    }
}

// impl<'r> FromRow<'r, PgRow> for Token {
//     fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
//         let id = row.try_get("id")?;
//         let name = row.try_get("name")?;
//         let token = row.try_get("token")?;
//         let token_type: Option<String> = row.try_get("token_type")?;
//         let token_type = token_type.map(TokenType::from);
//         let expiration = row.try_get("expiration")?;
//         let user_id = row.try_get("user_id")?;
//         let session_id = row.try_get("session_id")?;
//         let created_at = row.try_get("created_at")?;
//         let updated_at = row.try_get("updated_at")?;
//         Ok(Token {
//             id,
//             name,
//             token,
//             token_type,
//             expiration,
//             user_id,
//             session_id,
//             created_at,
//             updated_at,
//         })
//     }
// }

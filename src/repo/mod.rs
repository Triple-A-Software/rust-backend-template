use std::fmt::Display;

use serde::Deserialize;

pub mod activity;
pub mod session;
pub mod settings;
pub mod tag;
pub mod token;
pub mod user;

#[derive(Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asc => write!(f, "asc"),
            Self::Desc => write!(f, "desc"),
        }
    }
}

pub struct DatabaseListOptions {
    pub limit: i64,
    pub offset: i64,
    pub sort_by: String,
    pub sort_direction: SortDirection,
}

pub struct DatabasePagination {
    pub limit: i64,
    pub offset: i64,
}

use std::str::FromStr;

use uuid::Uuid;

pub const SESSION_COOKIE: &str = "session";
pub const APP_NAME: &str = "Your app name here";
const SYSTEM_USER_ID: &str = "00000000-0000-4000-0000-000000000000"; // You shouldn't change this

pub fn system_user_uuid() -> Uuid {
    Uuid::from_str(SYSTEM_USER_ID).expect("Couldn't parse SYSTEM_USER_ID")
}

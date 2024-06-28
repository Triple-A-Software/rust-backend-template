pub fn generate_session_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

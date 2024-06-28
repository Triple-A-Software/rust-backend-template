use std::convert::Infallible;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::extract::CookieJar;

use crate::{config::SESSION_COOKIE, model::user::User, repo::user::UserRepo, AppState};

pub struct Session<T>(pub Option<T>);

#[async_trait]
impl<S> FromRequestParts<S> for Session<User>
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let jar: CookieJar = match parts.extract().await {
            Ok(jar) => jar,
            Err(_) => CookieJar::new(),
        };
        let session_cookie = jar.get(SESSION_COOKIE).map(|cookie| cookie.value());
        let user = if let Some(session) = session_cookie {
            let conn = &mut state.db.acquire().await.unwrap();
            UserRepo::get_from_session_token(session, conn).await.ok()
        } else {
            None
        };
        Ok(Self(user))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Session<String>
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let jar: CookieJar = match parts.extract().await {
            Ok(jar) => jar,
            Err(_) => CookieJar::new(),
        };
        let session_cookie = jar
            .get(SESSION_COOKIE)
            .map(|cookie| cookie.value().to_string());
        Ok(Self(session_cookie))
    }
}

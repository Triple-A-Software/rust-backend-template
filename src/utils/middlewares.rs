use crate::{config::SESSION_COOKIE, repo::user::UserRepo, service::setup::SetupService, AppState};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json, RequestExt,
};
use axum_extra::extract::CookieJar;
use futures::future::BoxFuture;
use serde_json::json;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct SetupFinishedLayer {
    should_be_finished: bool,
    state: AppState,
}

impl SetupFinishedLayer {
    pub fn with_state(state: AppState) -> Self {
        Self {
            state,
            should_be_finished: false,
        }
    }

    pub fn finished(mut self, finished: bool) -> Self {
        self.should_be_finished = finished;
        self
    }
}

impl<S> Layer<S> for SetupFinishedLayer {
    type Service = SetupFinishedMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SetupFinishedMiddleware {
            inner,
            should_be_finished: self.should_be_finished,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SetupFinishedMiddleware<S> {
    inner: S,
    should_be_finished: bool,
    state: AppState,
}

impl<S> Service<Request> for SetupFinishedMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let future = self.inner.call(req);
        let should_be_finished = self.should_be_finished;
        let conn = self.state.db.acquire();
        Box::pin(async move {
            let conn = &mut conn.await.unwrap();
            let is_finished = match SetupService::is_setup_finished(conn).await {
                Ok(v) => v,
                Err(_) => {
                    return Ok((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "errorMessage": "Setup not finished"
                        })),
                    )
                        .into_response())
                }
            };
            if is_finished == should_be_finished {
                future.await
            } else {
                Ok((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "errorMessage": "Setup not finished"
                    })),
                )
                    .into_response())
            }
        })
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let conn = &mut state.db.acquire().await.unwrap();
    let jar = req.extract_parts::<CookieJar>().await.unwrap();
    let session_cookie = jar.get(SESSION_COOKIE).map(|cookie| cookie.value());
    let user = if let Some(session) = session_cookie {
        UserRepo::get_from_session_token(session, conn).await.ok()
    } else {
        None
    };
    if user.is_none() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "errorMessage": "unauthorized"
            })),
        )
            .into_response());
    }
    Ok(next.run(req).await)
}

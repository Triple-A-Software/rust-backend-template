use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    utils::middlewares::{auth_middleware, SetupFinishedLayer},
    AppState,
};

pub mod api;
pub mod files;

pub fn create_router(state: AppState) -> Router<AppState> {
    let authenticated_router = Router::new()
        .route("/users", get(api::users::get).post(api::users::post))
        .route(
            "/users/:id",
            get(api::users::get_by_id)
                .put(api::users::put)
                .delete(api::users::delete),
        )
        .route("/users/:id/password", put(api::users::update_password))
        .route(
            "/users/:id/avatar",
            post(api::users::update_avatar).delete(api::users::delete_avatar),
        )
        .route("/users/search", get(api::users::search))
        .route("/activity", get(api::activity::get))
        .route(
            "/settings/preferences",
            post(api::settings::post_preferences),
        )
        .route("/tags", get(api::tags::get))
        .route("/tokens", post(api::tokens::post).get(api::tokens::get))
        .route("/tokens/:token_id", delete(api::tokens::delete_by_id))
        .route("/sessions", get(api::sessions::list))
        .route("/sessions/:session_id", delete(api::sessions::delete_by_id))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));
    let setup_finished_router = Router::new()
        .nest("", authenticated_router)
        .route("/auth/check", get(api::auth::check))
        .route("/auth/logout", post(api::auth::logout))
        .route("/auth/login", post(api::auth::login))
        .route(
            "/password_reset/request",
            post(api::password_reset::request),
        )
        .route(
            "/password_reset/token_check",
            get(api::password_reset::token_check),
        )
        .route("/password_reset/reset", post(api::password_reset::reset))
        .layer(SetupFinishedLayer::with_state(state.clone()).finished(true));
    Router::new()
        .nest(
            "/api/rest",
            Router::new()
                .nest(
                    "/setup",
                    Router::new()
                        .route("/is_setup_finished", get(api::setup::is_setup_finished))
                        .route("/create_admin_user", post(api::setup::create_admin_user)),
                )
                .nest("", setup_finished_router),
        )
        .nest(
            "/api/ws",
            Router::new()
                .route("/user/status", get(api::ws::user_status))
                .route("/user/me/status", get(api::ws::user_me_status))
                .layer(SetupFinishedLayer::with_state(state.clone()).finished(true))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                )),
        )
        .nest(
            "/files",
            Router::new()
                .route("/avatar/:user_id", get(files::avatar))
                .layer(SetupFinishedLayer::with_state(state.clone()).finished(true))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                )),
        )
}

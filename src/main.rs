use std::{env, net::SocketAddr, path::PathBuf};

use axum::{routing::get, Router};
use events::EventChannel;
use tokio::net::TcpListener;
use tower_http::{
    services::ServeDir,
    trace::{self, TraceLayer},
};
use tracing::Level;

use sqlx::PgPool;

mod config;
mod events;
mod model;
mod repo;
mod routes;
mod service;
mod utils;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    event_channel: EventChannel,
    upload_path: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await.unwrap();
    init_db(&pool).await;

    let state = AppState {
        db: pool.clone(),
        event_channel: EventChannel::new(),
        upload_path: PathBuf::from(env::var("UPLOAD_PATH").unwrap_or("./upload".to_string())),
    };

    let static_files = ServeDir::new("static");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/", routes::create_router(state.clone()))
        .nest_service("/static", static_files)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state);

    let listener = TcpListener::bind((
        env::var("LISTEN_ADDRESS").unwrap_or("0.0.0.0".to_string()),
        env::var("PORT")
            .map(|p| p.parse().unwrap_or(3000))
            .unwrap_or(3000),
    ))
    .await
    .unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct Table {
    tablename: Option<String>,
    schemaname: Option<String>,
    rowsecurity: Option<bool>,
    hastriggers: Option<bool>,
    hasindexes: Option<bool>,
    tablespace: Option<String>,
    tableowner: Option<String>,
    hasrules: Option<bool>,
}

async fn init_db(db: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(db)
        .await
        .expect("Failed to run migrations");

    // Create triggers to update updated_at
    let tables: Vec<Table> = sqlx::query_as("SELECT * from pg_catalog.pg_tables where schemaname != 'pg_catalog' and schemaname != 'information_schema' and tablename != '_sqlx_migrations'").fetch_all(db).await.unwrap();
    for table in tables {
        let mut table_name = table.tablename.unwrap();
        if let Some(schema_name) = table.schemaname {
            table_name = format!("{}.{table_name}", schema_name);
        }
        sqlx::query(
            format!(
                r"
CREATE OR REPLACE TRIGGER update_updated_at_trigger
BEFORE UPDATE ON {}
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_function();
        ",
                table_name,
            )
            .as_str(),
        )
        .execute(db)
        .await
        .unwrap();
    }
}

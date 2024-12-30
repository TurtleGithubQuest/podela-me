use crate::page::{index, user, WebData};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use common::database::{create_pool, migrate};
use common::PodelError;
use sqlx::postgres::PgPool;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub mod page;

#[derive(Clone)]
pub struct AppState {
    pub data: WebData<'static>,
    pub pool: PgPool,
}

impl AppState {
    fn new(pool: PgPool) -> Self {
        Self {
            pool,
            data: WebData {
                title: "Podela.me",
                visitors: 0,
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), PodelError> {
    let pool = create_pool().await?;

    migrate(&pool).await.expect("Database migration failed");

    let state = AppState::new(pool);

    let app = Router::new()
        .route("/", get(index::get))
        .route("/user/{id}", get(user::get_profile))
        .with_state(state)
        .nest_service("/assets", ServeDir::new("assets"));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::Postgres>);

impl<S> FromRequestParts<S> for DatabaseConnection
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.acquire().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

async fn using_connection_extractor(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&mut *conn)
        .await
        .map_err(internal_error)
}

/// Utility function for mapping any error into a `500 Internal Server Error` response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

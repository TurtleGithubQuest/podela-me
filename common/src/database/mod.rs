use crate::{args, PodelError};
use clap::Parser;
use log::{info, warn};
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use std::time::Duration;

pub mod user;
pub mod reviewable;

pub type Ulid = String;
static MIGRATOR: Migrator = sqlx::migrate!(".\\migrations");

#[derive(Parser, Debug)]
pub struct DbCredentials {
    #[clap(long, env = "DB_HOST", default_value = "localhost")]
    pub host: String,
    #[clap(long, env = "DB_PORT", default_value = "5432")]
    pub port: u16,
    #[clap(long, env = "DB_USERNAME", default_value = "username")]
    pub username: String,
    #[clap(long, env = "DB_PASSWORD", default_value = "password")]
    pub password: String,
    #[clap(long, env = "DB_NAME", default_value = "development_db")]
    pub name: String,
}

pub async fn create_pool() -> Result<Pool<Postgres>, PodelError> {
    let args = args::CliArgs::parse();
    let db_credentials = &args.db;

    let db_connection_str = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_credentials.username,
        db_credentials.password,
        db_credentials.host,
        db_credentials.port,
        db_credentials.name
    );

    Ok(PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await?)
}

pub async fn migrate(pool: &PgPool) -> Result<(), MigrateError> {
    info!("Running database migrations...");

    match MIGRATOR.run(pool).await {
        Ok(_) => {
            info!("Database migrations completed successfully!");
            Ok(())
        }
        Err(e) => {
            warn!("Migration error: {}", e);
            Err(e)
        }
    }
}

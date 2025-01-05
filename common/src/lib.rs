extern crate core;

use crate::database::user::{verify_password, Credentials, User};
use log::info;
use sqlx::PgPool;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

pub mod args;
pub mod database;

#[derive(Error, Debug)]
pub enum PodelError {
    #[error("DbError: {0}")]
    DatabaseError(String),
    #[error("UserError: {0}")]
    UserError(String),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    Argon2PasswordHashError(#[from] argon2::password_hash::Error),
    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct AppState {
    pub title: &'static str,
    pub visitors: u64,
    pub pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            title: "Podela.me",
            visitors: 0,
        }
    }
}

pub async fn load_config() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Failed to find workspace root")
        .unwrap()
        .to_owned();

    let config_path = workspace_root.join("config").join("log4rs.yaml");

    log4rs::init_file(config_path, Default::default()).unwrap();
    info!("Config loaded");
}
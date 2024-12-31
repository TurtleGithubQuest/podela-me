use crate::database::user::{verify_password, Credentials, User};
use axum_login::axum::async_trait;
use axum_login::{AuthnBackend, UserId};
use sqlx::PgPool;
use std::str::FromStr;
use thiserror::Error;

pub mod args;
pub mod database;

pub struct Website {
    url: String,
    name: String,
}

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
            /*user: None,
            user_language: LanguageIdentifier::from_str("en-US").unwrap(),*/
        }
    }
}

#[async_trait]
impl AuthnBackend for AppState {
    type User = User;
    type Credentials = Credentials;
    type Error = PodelError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> =
            sqlx::query_as("SELECT * FROM auth.user WHERE name = $1 LIMIT 1")
                .bind(creds.username)
                .fetch_optional(&self.pool)
                .await?;

        tokio::task::spawn_blocking(|| {
            Ok(user.filter(|user| verify_password(creds.password, &user.password_hash).is_ok()))
        })
        .await?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("SELECT * FROM auth.user WHERE id = $1 LIMIT 1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }
}

pub type AuthSession = axum_login::AuthSession<AppState>;

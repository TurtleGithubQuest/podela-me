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
}

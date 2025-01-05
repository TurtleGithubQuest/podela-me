use crate::database::{Ulid, UserId};
use crate::PodelError;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{Executor, Pool, Postgres};
use std::fmt::Debug;
use chrono::Days;

#[derive(sqlx::FromRow, sqlx::Type, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Ulid,
    pub name: String,
    pub email: Option<String>,
    pub language: String,
    pub is_admin: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing)]
    pub(crate) password_hash: String,
}

impl User {
    pub async fn find(id: impl Into<String>, pool: &Pool<Postgres>) -> Result<User, PodelError> {
        let user = sqlx::query_as::<Postgres, User>(
            r#"
            SELECT id, name, language, email, is_admin, created_at, password_hash
            FROM auth.user
            WHERE id = $1 OR name = $1
        "#,
        )
        .bind(id.into())
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn register(
        pool: &Pool<Postgres>,
        username: impl Into<String>,
        email: Option<impl Into<String>>,
        password: impl Into<String>,
        is_admin: bool,
    ) -> Result<User, PodelError> {
        let password_hash = hash_password(password)?;

        let user = User {
            id: ulid::Ulid::new().to_string(),
            name: username.into(),
            language: "en-US".into(),
            email: email.map(|e| e.into()),
            is_admin,
            created_at: chrono::Utc::now(),
            password_hash: password_hash.clone(),
        };

        let query = r#"
            INSERT INTO auth.user (
                id,
                email,
                password_hash,
                name,
                is_admin
            )
            VALUES (
                $1, $2, $3, $4, $5
            )
        "#;

        let result = sqlx::query(query)
            .bind(&user.id)
            .bind(&user.email)
            .bind(password_hash)
            .bind(&user.name)
            .bind(user.is_admin)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(PodelError::DatabaseError(
                "Failed to insert user".to_string(),
            ));
        }

        Ok(user)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Session {
    pub token: Ulid,
    /// logged-in user id
    pub user_id: Option<String>,
    /// logged-in user
    pub user: Option<User>,
    /// user's ip address on creation
    pub ip: String,
    /// shall we invalidate the session on ip change?
    pub enforce_ip: bool,
    pub expiration: chrono::DateTime<chrono::Utc>
}

impl Session {
    pub fn new(user_id: Option<impl Into<crate::database::Ulid>>, ip: impl Into<String>) -> Session {
        let now = chrono::Utc::now();
        let expiration = now.checked_add_days(Days::new(4));
        Self {
            token: Ulid::new().into(),
            user_id: user_id.map(Into::into),
            user: None,
            ip: ip.into(),
            enforce_ip: false, //todo
            expiration: expiration.unwrap_or(now),
        }
    }
}

pub fn verify_password(password: impl Into<String>, hash: &str) -> Result<(), PodelError> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    Ok(argon2.verify_password(password.into().as_bytes(), &parsed_hash)?)
}

pub fn hash_password(password: impl Into<String>) -> Result<String, PodelError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.into().as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}
/*
impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}*/

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Credentials {
    #[serde(default, deserialize_with = "deserialize_checkbox")]
    pub authentication: bool,
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

/// This will accept any value (including "on", which is what HTML forms send for checked checkboxes)
/// and return true if the field is present, false if it's absent
fn deserialize_checkbox<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)
        .ok()
        .flatten()
        .is_some())
}

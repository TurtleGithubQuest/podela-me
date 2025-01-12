use crate::database::{Ulid, UserId};
use crate::PodelError;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{Executor, Pool, Postgres, Row};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use chrono::Days;
use poem::session::Session;
use sqlx::postgres::PgRow;
use crate::database::reviewable::website::Website;

#[derive(sqlx::FromRow, sqlx::Type, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Ulid,
    pub name: String,
    pub email: Option<String>,
    pub language: String,
    pub is_admin: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing, skip_deserializing)]
    pub password_hash: String,
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

    pub async fn create_session(self, pool: &Pool<Postgres>, session: &Session, ip: Option<impl Into<String>>) -> Result<(), PodelError> {
        let arc = Arc::new(self);
        let session_data = SessionData::new(arc, ip);
        session_data.save(pool).await?;
        match session_data.to_base64() {
            Ok(base64) => Ok(session.set("data", base64)),
            Err(err) => Err(PodelError::UserError(err.to_string()))
        }
    }

    pub fn from_session(session: &Session) -> Result<Arc<Self>, PodelError> {
        if let Some(base64) = session.get::<String>("data") {
            match SessionData::from_base64(&base64) {
                Ok(data) => Ok(data.user),
                Err(err) => Err(PodelError::UserError(err.to_string()))
            }
        } else {
            Err(PodelError::Empty())
        }
    }

    pub fn from_row(row: &PgRow) -> Result<Self, PodelError> {
        if let Some(user_id) = row.try_get::<String, _>("user_id").ok() {
            Ok(User {
                id: user_id,
                email: row.try_get("user_email")?,
                password_hash: row.try_get("user_password_hash")?,
                language: row.try_get("user_language")?,
                name: row.try_get("user_name")?,
                is_admin: row.try_get("user_is_admin")?,
                created_at: row.try_get("user_created_at")?,
            })
        } else {
            Err(PodelError::UserError("Row does not have user id.".into()))
        }
    }
}

pub fn is_valid(session: &Session) -> bool {
    if let Some(expiration) = session.get::<chrono::DateTime<chrono::Utc>>("expiration") {
        if expiration > chrono::Utc::now() {
            return true
        }
    }
    false
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct SessionData {
    pub id: Ulid,
    /// logged-in user
    pub user: Arc<User>,
    /// user's ip address on creation
    pub ip: Option<String>,
    /// shall we invalidate the session on ip change?
    pub enforce_ip: bool,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>
}

impl SessionData {
    pub fn new(user: Arc<User>, ip: Option<impl Into<String>>) -> Self {
        let now = chrono::Utc::now();
        let expiration = now.checked_add_days(Days::new(4));
        Self {
            id: ulid::Ulid::new().into(),
            user,
            ip: ip.map(Into::into),
            enforce_ip: false, //todo
            expires_at: expiration.unwrap_or(now),
            created_at: now
        }
    }

    pub fn from_session(session: &Session) -> Option<Self> {
        session.get::<String>("data").map(|base64| Self::from_base64(&base64).ok()).flatten()
    }

    pub fn to_base64(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Convert expiration to i64 (UTC timestamp in seconds)
        let expiration_ts = self.expires_at.timestamp();
        // Serialize that i64 to 8 bytes (big-endian)
        let mut bytes = expiration_ts.to_be_bytes().to_vec();
        // Then serialize the full SessionData (including expiration, if you like)
        let session_bytes = bincode::serialize(self)?;

        // Prepend those 8 bytes
        bytes.extend_from_slice(&session_bytes);

        // Finally, Base64-encode
        let encoded = base64::encode(&bytes);
        Ok(encoded)
    }

    pub fn from_base64(encoded: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = base64::decode(encoded)?;
        if bytes.len() < 8 {
            println!("Not enough bytes {:?}", bytes);
            return Err("Not enough bytes to decode expiration".into());
        }

        // First 8 bytes -> expiration
        let (expiration_part, session_bytes) = bytes.split_at(8);
        let expiration_ts = i64::from_be_bytes(expiration_part.try_into()?);

        if expiration_ts < chrono::Utc::now().timestamp() {
            return Err(PodelError::UserError(format!("Session expired at: {expiration_ts}.")).into())
        }

        let session: SessionData = bincode::deserialize(session_bytes)?;
        Ok(session)
    }

    pub async fn authenticate(&self, pool: &Pool<Postgres>, session: &Session) -> Result<(), PodelError> {
        if let Err(err) = self.validate(pool).await {
            session.clear();
            Err(err)
        } else {
            Ok(())
        }
    }

    pub async fn validate(&self, pool: &Pool<Postgres>) -> Result<(), PodelError> {
        let row = sqlx::query(r#"
            SELECT id, user_id, ip, expires_at
            FROM auth.session
            WHERE id = $1 AND user_id = $2 AND ip = $3 AND expires_at > NOW()
        "#)
            .bind(&self.id)
            .bind(&self.user.id)
            .bind(&self.ip)
            .bind(&self.expires_at)
            .fetch_one(pool)
            .await?;

        if row.is_empty() {
            Err(PodelError::Empty())
        } else {
            Ok(())
        }
    }

    pub async fn save(&self, pool: &Pool<Postgres>) -> Result<(), PodelError> {
        let result = sqlx::query(r#"
                INSERT INTO auth.session (
                    id,
                    user_id,
                    ip,
                    enforce_ip,
                    expires_at,
                    created_at
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6
                )
            "#)
            .bind(&self.id)
            .bind(&self.user.id)
            .bind(&self.ip)
            .bind(&self.enforce_ip)
            .bind(&self.expires_at)
            .bind(&self.created_at)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            Err(PodelError::DatabaseError(
                "Failed to insert session".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

impl sqlx::FromRow<'_, PgRow> for SessionData {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let user = User::from_row(row).map_err(|_| sqlx::Error::RowNotFound)?;
        Ok(SessionData {
            id: row.try_get::<String, _>("id")?,
            user: Arc::new(user),
            ip: row.try_get::<Option<String>, _>("ip")?,
            enforce_ip: row.try_get::<bool, _>("enforce_ip")?,
            expires_at: row.try_get::<chrono::DateTime<chrono::Utc>, _>("expires_at")?,
            created_at: row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?,
        })
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

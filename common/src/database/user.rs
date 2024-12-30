use crate::PodelError;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use sqlx::{Executor, Pool, Postgres};

#[derive(sqlx::FromRow)]
pub struct User {
    /// ULID
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    #[sqlx(rename = "is_admin")]
    pub admin: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn find(id: impl Into<String>, pool: &Pool<Postgres>) -> Result<User, PodelError> {
        let user: User = sqlx::query_as(
            r#"
            SELECT id, name, email, is_admin, created_at
            FROM auth.user
            WHERE name = $1
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
        admin: bool,
    ) -> Result<User, PodelError> {
        let user = User {
            id: ulid::Ulid::new().to_string(),
            name: username.into(),
            email: email.map(|e| e.into()),
            admin,
            created_at: chrono::Utc::now(),
        };

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.into().as_bytes(), &salt)
            .map_err(|e| PodelError::UserError(format!("Password hashing failed: {}", e)))?
            .to_string();

        let query = r#"
            INSERT INTO auth.user (
                id,
                email,
                password_hash,
                name,
                is_admin,
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
            .bind(user.admin)
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

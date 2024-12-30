use crate::PodelError;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use sqlx::{Executor, Pool, Postgres};

pub struct User {
    /// ULID
    pub id: String,
    admin: bool,
    pub(crate) auth: UserAuth,
}

pub struct UserAuth {
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

pub struct UserView {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub admin: bool,
}

impl User {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        email: impl Into<String>,
        admin: bool,
    ) -> Result<User, PodelError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.into().as_bytes(), &salt)
            .map_err(|e| PodelError::UserError(format!("Password hashing failed: {}", e)))?
            .to_string();

        Ok(User {
            id: ulid::Ulid::new().to_string(),
            admin,
            auth: UserAuth {
                email: email.into(),
                password_hash,
                name: username.into(),
            },
        })
    }

    pub async fn register(&self, pool: &Pool<Postgres>) -> Result<(), PodelError> {
        let query = r#"
            INSERT INTO auth.users (
                id,
                email,
                password_hash,
                username,
                is_active,
                is_verified,
                created_at,
                updated_at
            )
            VALUES (
                $1, $2, $3, $4, true, false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )
        "#;

        let result = sqlx::query(query)
            .bind(&self.id)
            .bind(&self.auth.email)
            .bind(&self.auth.password_hash)
            .bind(&self.auth.name)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(PodelError::DatabaseError(
                "Failed to insert user".to_string(),
            ));
        }

        Ok(())
    }
}

impl Into<UserView> for User {
    fn into(self) -> UserView {
        UserView {
            id: self.id,
            name: self.auth.name,
            email: Some(self.auth.email),
            admin: self.admin,
        }
    }
}

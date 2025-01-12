use std::cmp::min;
use std::ops::Deref;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use crate::database::reviewable::website::Website;
use crate::database::Ulid;
use crate::database::user::User;
use crate::PodelError;

#[derive(sqlx::FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct Comment<T>
where
    T: Send + Sync + Unpin + sqlx::Type<Postgres> + for<'r> sqlx::Decode<'r, Postgres> + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
{
    pub id: Ulid,
    pub parent_id: Ulid,
    #[serde(skip_serializing, skip_deserializing)]
    pub parent: Option<T>,
    pub user: User,
    pub text: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl <T>Comment<T>
where
    T: Commentable + Send + Unpin + Sync + sqlx::Type<Postgres> + for<'r> sqlx::Decode<'r, Postgres> + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
{
    pub fn parent_table_name() -> String {
        std::any::type_name::<T>()
            .split("::")
            .last()
            .map(|s| s.to_lowercase())
            .unwrap_or_default()
    }

    pub fn new(parent: T, text: impl Into<String>, user: Arc<User>) -> Self {
        Self {
            id: ulid::Ulid::new().into(),
            parent_id: parent.id().into(),
            parent: Some(parent),
            user: user.deref().clone(),
            text: text.into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub async fn save(&self, pool: &Pool<Postgres>) -> Result<(), PodelError> {
        sqlx::query(&format!(r#"
                INSERT INTO comment.{} (
                    id,
                    parent_id,
                    user_id,
                    text,
                    created_at,
                    updated_at
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6
                )
            "#, Comment::<T>::parent_table_name()))
            .bind(&self.id)
            .bind(&self.parent_id)
            .bind(&self.user.id)
            .bind(&self.text)
            .bind(&self.created_at)
            .bind(&self.updated_at)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn find_multiple(
        pool: &Pool<Postgres>,
        limit: i64,
        offset: i64
    ) -> Result<Vec<Self>, PodelError> {
        Ok(
            sqlx::query_as::<Postgres, Self>(
            &format!(r#"
                SELECT
                    c.id,
                    c.parent_id as parent_id,
                    u.id as user_id,
                    u.email as user_email,
                    u.password_hash as user_password_hash,
                    u.language as user_language,
                    u.name as user_name,
                    u.is_admin as user_is_admin,
                    u.is_active as user_is_active,
                    u.is_verified as user_is_verified,
                    u.last_login as user_last_login,
                    u.created_at as user_created_at,
                    u.updated_at as user_updated_at,
                    c.text,
                    c.created_at,
                    c.updated_at
                FROM comment.{table_name} c
                JOIN auth.user u ON c.user_id = u.id
                ORDER BY c.created_at DESC
                LIMIT $1
                OFFSET $2
            "#, table_name = Self::parent_table_name()
            )
        )
        .bind(min(limit, 20))
        .bind(offset)
        .fetch_all(pool)
        .await?
        )

    }
}

pub trait Commentable {
    fn id(&self) -> &Ulid;
}
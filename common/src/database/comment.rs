use std::cmp::min;
use std::ops::Deref;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use crate::database::reviewable::website::Website;
use crate::database::Ulid;
use crate::database::user::User;
use crate::PodelError;

pub trait CommentParent: Commentable + Send + Sync + Unpin + sqlx::Type<Postgres> + for<'r> sqlx::Decode<'r, Postgres> + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> {}
impl<T> CommentParent for T where T: Commentable + Send + Sync + Unpin + sqlx::Type<Postgres> + for<'r> sqlx::Decode<'r, Postgres> + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> {}

#[derive(sqlx::FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: Ulid,
    pub parent_type: String,
    pub parent_id: Ulid,
    pub user: User,
    pub text: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Comment {
    pub fn new(parent_type: impl Into<String>, parent_id: impl Into<Ulid>, text: impl Into<String>, user: Arc<User>) -> Self {
        Self {
            id: ulid::Ulid::new().into(),
            parent_type: parent_type.into(),
            parent_id: parent_id.into(),
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
            "#, self.parent_type))
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
        parent_type: impl Into<String>,
        parent_id: impl Into<String> + sqlx::Type<sqlx::Postgres>,
        limit: i64,
        offset: i64
    ) -> Result<Vec<Self>, PodelError> {
        let parent_type = parent_type.into();
        println!("Fetching for {parent_type}");
        Ok(
            sqlx::query_as::<Postgres, Comment>(
            &format!(r#"
                SELECT
                    c.id,
                    c.parent_id as parent_id,
                    $1::text as parent_type,
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
                FROM comment.{parent_type} c
                LEFT JOIN auth.user u ON c.user_id = u.id
                WHERE parent_id = $2
                ORDER BY c.created_at DESC
                LIMIT $3
                OFFSET $4
            "#)
        )
        .bind(parent_type)
        .bind(parent_id.into())
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
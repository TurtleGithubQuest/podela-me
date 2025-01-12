use std::cmp::min;
use crate::database::reviewable::{Karma, Organization};
use crate::database::user::User;
use crate::database::Ulid;
use crate::PodelError;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{Executor, Pool, Postgres, Row};
use crate::database::comment::Commentable;

#[derive(sqlx::Type, Clone, Debug, Serialize, Deserialize)]
pub struct Website {
    pub id: Ulid,
    pub karma: Karma,
    pub organization: Option<Organization>,
    pub name: String,
    pub domain_name: String,
    pub description: Option<String>,
}

impl sqlx::FromRow<'_, PgRow> for Website {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let organization = if let Some(organization_id) = row.try_get::<String, _>("org_id").ok() {
            let user = User::from_row(row).ok();

            Some(Organization {
                id: organization_id,
                name: row.try_get("org_name")?,
                form: row.try_get("org_form")?,
                user,
                created_at: row.try_get("org_created_at")?,
                updated_at: row.try_get("org_updated_at")?
            })
        } else {
            None
        };

        Ok(Website {
            id: row.try_get("id")?,
            karma: row.try_get("karma")?,
            organization,
            name: row.try_get("name")?,
            domain_name: row.try_get("domain_name")?,
            description: row.try_get("description").ok(),
        })
    }
}

impl Website {
    pub fn new(
        name: impl Into<String>,
        domain_name: impl Into<String>,
        description: Option<impl Into<String>>,
        organization: Option<Organization>
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            organization,
            karma: Karma::new(),
            name: name.into(),
            domain_name: domain_name.into(),
            description: description.map(|o| o.into()),
        }
    }

    pub fn with_organization(self, organization: Organization) -> Self {
        Self {
            organization: Some(organization),
            ..self
        }
    }

    fn get_query() -> &'static str {
        r#"SELECT
            w.id,
            w.karma,
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
            o.id as org_id,
            o.name as org_name,
            o.form as org_form,
            o.created_at as org_created_at,
            o.updated_at as org_updated_at,
            w.name,
            w.domain_name,
            w.description
        FROM subject.website w
        LEFT JOIN subject.organization o ON w.organization_id = o.id
        LEFT JOIN auth.user u ON o.user_id = u.id
        "#
    }

    pub async fn find(pool: &Pool<Postgres>, id: impl Into<String>) -> Result<Website, PodelError> {
        let website = sqlx::query_as::<Postgres, Website>(&format!("{} WHERE w.id = $1 OR w.name = $1", Self::get_query()))
        .bind(id.into())
        .fetch_one(pool)
        .await?;

        Ok(website)
    }

    pub async fn find_multiple(
        pool: &Pool<Postgres>,
        limit: i64,
        offset: i64
    ) -> Result<Vec<Website>, PodelError> {
        let websites = sqlx::query_as::<Postgres, Website>(
            &format!(r#"
                {}
                ORDER BY w.created_at DESC
                LIMIT $1
                OFFSET $2"#,
                Self::get_query()
            ),
        )
        .bind(min(limit, 20))
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(websites)
    }

    pub async fn save(&self, pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
        let mut transaction = pool.begin().await?;

        let organization_id = if let Some(org) = &self.organization {
            transaction.execute(sqlx::query(r#"
                    INSERT INTO subject.organization (id, name, form, user_id)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (id) DO UPDATE
                    SET form = EXCLUDED.form, user_id = EXCLUDED.user_id, updated_at = CURRENT_TIMESTAMP
                "#)
                .bind(&org.id)
                .bind(&org.name)
                .bind(&org.form)
                .bind(&org.user.as_ref().map(|user| &user.id))
            ).await?;

            Some(&org.id)
        } else {
            None
        };

        transaction.execute(sqlx::query(r#"
                INSERT INTO subject.website (id, organization_id, name, domain_name, description)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) DO UPDATE
                SET organization_id = EXCLUDED.organization_id, name = EXCLUDED.name, domain_name = EXCLUDED.domain_name, description = EXCLUDED.description, updated_at = CURRENT_TIMESTAMP
            "#)
            .bind(&self.id)
            .bind(&organization_id)
            .bind(&self.name)
            .bind(&self.domain_name)
            .bind(&self.description)
        ).await?;

        transaction.commit().await?;
        Ok(())
    }
}

impl Commentable for Website {
    fn id(&self) -> &Ulid {
        &self.id
    }
}
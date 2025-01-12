use crate::database::user::User;
use crate::database::Ulid;
use core::fmt;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Row};

pub mod website;

#[derive(sqlx::FromRow, sqlx::Type, Clone, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "RECORD")]
pub struct Organization {
    pub id: Ulid,
    pub name: String,
    pub form: LegalForm,
    pub user: Option<User>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Organization {
    pub fn new(name: impl Into<String>, form: LegalForm, user: Option<User>) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            name: name.into(),
            form,
            user,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

/// Represents the type of legal entity.
#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[sqlx(type_name = "subject.legal_form")]
pub enum LegalForm {
    /// Společnost s ručením omezením (Limited Liability Company)
    Sro,
    /// Akciová společnost (Joint Stock Company)
    As,
    /// Veřejná obchodní společnost (General Partnership)
    Vos,
    /// Zapsaný spolek (Registered Association)
    Spolek,
    /// Nadace (Foundation)
    Nadace,
    /// Družstvo (Cooperative)
    Druzstvo,
}

impl fmt::Display for LegalForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LegalForm::Sro => write!(f, "Společnost s ručením omezením"),
            LegalForm::As => write!(f, "Akciová společnost"),
            LegalForm::Vos => write!(f, "Veřejná obchodní společnost"),
            LegalForm::Spolek => write!(f, "Zapsaný spolek"),
            LegalForm::Nadace => write!(f, "Nadace"),
            LegalForm::Druzstvo => write!(f, "Družstvo"),
        }
    }
}

#[derive(sqlx::Type, sqlx::FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct Karma {
    pub amount: i16,
    pub reviews: i16,
    pub age: i16,
    pub popularity: i16
}

impl Karma {
    pub fn new() -> Karma {
        Self {
            amount: 0,
            reviews: 0,
            age: 0,
            popularity: 0
        }
    }
}
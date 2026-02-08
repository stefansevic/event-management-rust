// Modeli za user servis - profili korisnika

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

/// Profil korisnika u bazi
#[derive(Debug, FromRow, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub full_name: String,
    pub phone: Option<String>,
    pub bio: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Zahtev za kreiranje/azuriranje profila
#[derive(Debug, Deserialize)]
pub struct ProfileRequest {
    pub full_name: String,
    pub phone: Option<String>,
    pub bio: Option<String>,
}

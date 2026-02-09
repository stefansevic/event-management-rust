// Modeli za usere

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

/// user u bazi
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

/// Req za create/update profile
#[derive(Debug, Deserialize)]
pub struct ProfileRequest {
    pub full_name: String,
    pub phone: Option<String>,
    pub bio: Option<String>,
}

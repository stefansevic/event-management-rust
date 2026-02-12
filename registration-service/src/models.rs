
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

/// register to event
#[derive(Debug, FromRow, Serialize)]
pub struct Registration {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub ticket_code: String,
    pub status: String,        // confirmed ili cancelled
    pub created_at: NaiveDateTime,
}

/// req for registration
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub event_id: Uuid,
}

/// Podaci o eventu koje dobijamo od event servisa
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct EventData {
    pub id: Uuid,
    pub title: String,
    pub capacity: i32,
}

/// Wrapper za odgovor 
#[derive(Debug, Deserialize)]
pub struct EventServiceResponse {
    pub success: bool,
    pub data: Option<EventData>,
}

/// Broj registracija 
#[derive(Debug, FromRow)]
pub struct CountResult {
    pub count: Option<i64>,
}


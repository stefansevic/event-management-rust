

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

/// event in db
#[derive(Debug, FromRow, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub organizer_id: Uuid,
    pub title: String,
    pub description: String,
    pub location: String,
    pub date_time: NaiveDateTime,
    pub capacity: i32,
    pub category: String,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// create req
#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: String,
    pub location: String,
    pub date_time: NaiveDateTime,
    pub capacity: i32,
    pub category: String,
    pub image_url: Option<String>,
}

/// Query params za search
#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub category: Option<String>,
    pub search: Option<String>,
}

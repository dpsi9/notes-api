use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow; // converts the entire row from database into a Rust struct

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

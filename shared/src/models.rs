// Zajednicki modeli koje koriste svi servisi

use serde::{Deserialize, Serialize};

/// Standardni format odgovora za sve API pozive.
/// T moze biti bilo koji tip - npr. User, Event, itd.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Vraca uspesan odgovor sa podacima
    pub fn success(message: &str, data: T) -> Self {
        ApiResponse {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    /// Vraca gresku bez podataka
    pub fn error(message: &str) -> Self {
        ApiResponse {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

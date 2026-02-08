// Definicije gresaka - koristimo ih kad nesto podje naopako

use serde::{Deserialize, Serialize};

/// Struktura greske koju saljemo klijentu
#[derive(Debug, Serialize, Deserialize)]
pub struct AppError {
    pub code: u16,
    pub message: String,
}

impl AppError {
    pub fn bad_request(msg: &str) -> Self {
        AppError { code: 400, message: msg.to_string() }
    }

    pub fn unauthorized(msg: &str) -> Self {
        AppError { code: 401, message: msg.to_string() }
    }

    pub fn not_found(msg: &str) -> Self {
        AppError { code: 404, message: msg.to_string() }
    }

    pub fn internal(msg: &str) -> Self {
        AppError { code: 500, message: msg.to_string() }
    }
}

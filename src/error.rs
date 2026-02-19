use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub message: String,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl AppError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::new(format!("database error: {e}"))
    }
}

#[cfg(feature = "ssr")]
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::new(format!("HTTP error: {e}"))
    }
}

#[cfg(feature = "ssr")]
impl From<qdrant_client::QdrantError> for AppError {
    fn from(e: qdrant_client::QdrantError) -> Self {
        AppError::new(format!("qdrant error: {e}"))
    }
}

#[cfg(feature = "ssr")]
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::new(format!("IO error: {e}"))
    }
}

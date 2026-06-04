use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Migration: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Scraper: {0}")]
    Scrape(String),

    #[error("Env: {0}")]
    Env(String),

    #[error("Cloudflare blocked — provide a valid cf_clearance token")]
    CfBlocked,
}

// Tauri commands require serializable errors
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

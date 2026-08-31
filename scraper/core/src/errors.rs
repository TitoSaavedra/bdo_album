use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database: {0}")]
    Db(#[from] sqlx::Error),

    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Scraper: {0}")]
    Scrape(String),

    #[error("Env: {0}")]
    Env(String),

}

// Tauri commands require serializable errors
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

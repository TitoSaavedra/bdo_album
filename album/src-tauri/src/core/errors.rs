use sqlx::Error as SqlxError;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Database(SqlxError),
    NotFound(String),
    Internal(String),
    Io(std::io::Error),
    Xml(String),
    Http(String),
    Env(String),
    R2NotConfigured,
}

impl From<SqlxError> for AppError {
    fn from(e: SqlxError) -> Self {
        AppError::Database(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Http(e.to_string())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(e)   => write!(f, "Database error: {}", e),
            AppError::NotFound(s)   => write!(f, "Not found: {}", s),
            AppError::Internal(s)   => write!(f, "Internal error: {}", s),
            AppError::Io(e)         => write!(f, "IO error: {}", e),
            AppError::Xml(s)        => write!(f, "XML error: {}", s),
            AppError::Http(s)       => write!(f, "HTTP error: {}", s),
            AppError::Env(s)        => write!(f, "Env error: {}", s),
            AppError::R2NotConfigured => write!(f, "R2 not configured"),
        }
    }
}

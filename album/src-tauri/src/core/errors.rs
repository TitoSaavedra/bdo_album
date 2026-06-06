use sqlx::Error as SqlxError;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Database(SqlxError),
    NotFound(String),
    Internal(String),
}

impl From<SqlxError> for AppError {
    fn from(e: SqlxError) -> Self {
        AppError::Database(e)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::NotFound(s) => write!(f, "Not found: {}", s),
            AppError::Internal(s) => write!(f, "Internal error: {}", s),
        }
    }
}

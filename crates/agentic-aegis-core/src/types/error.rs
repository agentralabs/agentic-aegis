use thiserror::Error;

#[derive(Error, Debug)]
pub enum AegisError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("session error: {0}")]
    Session(String),

    #[error("shadow execution error: {0}")]
    ShadowExecution(String),

    #[error("protection error: {0}")]
    Protection(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("security threat detected: {0}")]
    SecurityThreat(String),

    #[error("rollback error: {0}")]
    Rollback(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type AegisResult<T> = Result<T, AegisError>;

impl From<std::io::Error> for AegisError {
    fn from(e: std::io::Error) -> Self {
        AegisError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AegisError {
    fn from(e: serde_json::Error) -> Self {
        AegisError::Serialization(e.to_string())
    }
}

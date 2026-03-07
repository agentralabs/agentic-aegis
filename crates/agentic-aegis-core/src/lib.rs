pub mod bridges;
pub mod cache;
pub mod metrics;
pub mod protection;
pub mod query;
pub mod session;
pub mod shadow;
pub mod types;
pub mod validators;

pub use session::SessionManager;
pub use types::{
    AegisError, AegisId, AegisResult, Language, SecurityScan, SessionConfig, SessionId,
    SessionState, StreamingValidation, ThreatLevel, ValidationContext, ValidationError,
    ValidationResult, ValidationSession, ValidationSeverity,
};

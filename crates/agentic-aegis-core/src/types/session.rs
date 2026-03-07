use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::SessionId;
use super::validation::{Language, ValidationContext, ValidationResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SessionState {
    #[default]
    Created,
    Active,
    Paused,
    Completed,
    Failed,
    RolledBack,
}

impl SessionState {
    pub fn is_active(&self) -> bool {
        matches!(self, SessionState::Active | SessionState::Paused)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SessionState::Completed | SessionState::Failed | SessionState::RolledBack
        )
    }

    pub fn can_transition_to(&self, target: &SessionState) -> bool {
        matches!(
            (self, target),
            (SessionState::Created, SessionState::Active)
                | (SessionState::Active, SessionState::Paused)
                | (SessionState::Active, SessionState::Completed)
                | (SessionState::Active, SessionState::Failed)
                | (SessionState::Active, SessionState::RolledBack)
                | (SessionState::Paused, SessionState::Active)
                | (SessionState::Paused, SessionState::Completed)
                | (SessionState::Paused, SessionState::Failed)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub language: Language,
    pub file_path: Option<String>,
    pub max_errors: usize,
    pub stop_on_first_error: bool,
    pub enable_type_checking: bool,
    pub enable_security_scan: bool,
    pub timeout_ms: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            language: Language::Unknown,
            file_path: None,
            max_errors: 50,
            stop_on_first_error: false,
            enable_type_checking: true,
            enable_security_scan: true,
            timeout_ms: 30_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub code: String,
    pub chunk_index: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSession {
    pub id: SessionId,
    pub state: SessionState,
    pub config: SessionConfig,
    pub context: ValidationContext,
    pub results: Vec<ValidationResult>,
    pub snapshots: Vec<SessionSnapshot>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub total_chunks_processed: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
}

impl ValidationSession {
    pub fn new(config: SessionConfig) -> Self {
        let id = SessionId::new();
        let now = Utc::now();
        let language = config.language;
        let file_path = config.file_path.clone().unwrap_or_default();
        let context = ValidationContext::new(id.clone(), language, file_path);

        Self {
            id,
            state: SessionState::Created,
            config,
            context,
            results: Vec::new(),
            snapshots: Vec::new(),
            created_at: now,
            updated_at: now,
            total_chunks_processed: 0,
            total_errors: 0,
            total_warnings: 0,
        }
    }

    pub fn activate(&mut self) -> Result<(), String> {
        if self.state.can_transition_to(&SessionState::Active) {
            self.state = SessionState::Active;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("cannot transition from {:?} to Active", self.state))
        }
    }

    pub fn pause(&mut self) -> Result<(), String> {
        if self.state.can_transition_to(&SessionState::Paused) {
            self.state = SessionState::Paused;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("cannot transition from {:?} to Paused", self.state))
        }
    }

    pub fn complete(&mut self) -> Result<(), String> {
        if self.state.can_transition_to(&SessionState::Completed) {
            self.state = SessionState::Completed;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!(
                "cannot transition from {:?} to Completed",
                self.state
            ))
        }
    }

    pub fn fail(&mut self) -> Result<(), String> {
        if self.state.can_transition_to(&SessionState::Failed) {
            self.state = SessionState::Failed;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("cannot transition from {:?} to Failed", self.state))
        }
    }

    pub fn take_snapshot(&mut self) {
        let snapshot = SessionSnapshot {
            code: self.context.accumulated_code.clone(),
            chunk_index: self.context.chunk_index,
            timestamp: Utc::now(),
        };
        self.snapshots.push(snapshot);
    }

    pub fn is_over_error_limit(&self) -> bool {
        self.total_errors >= self.config.max_errors
    }
}

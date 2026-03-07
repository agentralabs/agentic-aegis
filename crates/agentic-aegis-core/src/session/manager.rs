use std::collections::HashMap;

use crate::types::{
    AegisError, AegisResult, SessionConfig, SessionId, SessionState, StreamingValidation,
    ValidationSession,
};
use crate::validators::{
    SemanticValidator, StreamingValidator, SyntaxValidator, TokenValidator, TypeValidator,
};

pub struct SessionManager {
    sessions: HashMap<String, ValidationSession>,
    validators: Vec<Box<dyn StreamingValidator>>,
}

impl SessionManager {
    pub fn new() -> Self {
        let validators: Vec<Box<dyn StreamingValidator>> = vec![
            Box::new(TokenValidator::new()),
            Box::new(SyntaxValidator::new()),
            Box::new(TypeValidator::new()),
            Box::new(SemanticValidator::new()),
        ];

        Self {
            sessions: HashMap::new(),
            validators,
        }
    }

    pub fn create_session(&mut self, config: SessionConfig) -> AegisResult<SessionId> {
        let mut session = ValidationSession::new(config);
        let id = session.id.clone();
        session.activate().map_err(AegisError::Session)?;
        self.sessions.insert(id.to_string(), session);
        Ok(id)
    }

    pub fn get_session(&self, id: &str) -> AegisResult<&ValidationSession> {
        self.sessions
            .get(id)
            .ok_or_else(|| AegisError::NotFound(format!("session not found: {}", id)))
    }

    pub fn get_session_mut(&mut self, id: &str) -> AegisResult<&mut ValidationSession> {
        self.sessions
            .get_mut(id)
            .ok_or_else(|| AegisError::NotFound(format!("session not found: {}", id)))
    }

    pub async fn validate_chunk(
        &mut self,
        session_id: &str,
        chunk: &str,
    ) -> AegisResult<StreamingValidation> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| AegisError::NotFound(format!("session not found: {}", session_id)))?;

        if !session.state.is_active() {
            return Err(AegisError::Session(format!(
                "session is not active: {:?}",
                session.state
            )));
        }

        // Take snapshot before validation
        session.take_snapshot();

        let context = &session.context;
        let mut combined = StreamingValidation::ok(context.chunk_index);

        for validator in &self.validators {
            let result = validator.validate_chunk(context, chunk).await?;

            if !result.valid {
                combined.valid = false;
            }
            if result.should_stop {
                combined.should_stop = true;
            }
            combined.errors.extend(result.errors);
            combined.warnings.extend(result.warnings);
            if combined.correction_hint.is_none() {
                combined.correction_hint = result.correction_hint;
            }
            combined.confidence = combined.confidence.min(result.confidence);
        }

        // Update session state
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| AegisError::NotFound(format!("session not found: {}", session_id)))?;

        session.context.append_chunk(chunk);
        session.total_chunks_processed += 1;
        session.total_errors += combined.errors.len();
        session.total_warnings += combined.warnings.len();

        if session.is_over_error_limit() {
            let _ = session.fail();
            combined.should_stop = true;
        }

        Ok(combined)
    }

    pub fn end_session(&mut self, session_id: &str) -> AegisResult<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| AegisError::NotFound(format!("session not found: {}", session_id)))?;

        session.complete().map_err(AegisError::Session)?;
        Ok(())
    }

    pub fn list_sessions(&self) -> Vec<&ValidationSession> {
        self.sessions.values().collect()
    }

    pub fn active_sessions(&self) -> Vec<&ValidationSession> {
        self.sessions
            .values()
            .filter(|s| s.state == SessionState::Active)
            .collect()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn remove_session(&mut self, session_id: &str) -> AegisResult<ValidationSession> {
        self.sessions
            .remove(session_id)
            .ok_or_else(|| AegisError::NotFound(format!("session not found: {}", session_id)))
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

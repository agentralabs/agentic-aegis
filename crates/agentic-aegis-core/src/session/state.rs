use crate::types::{AegisError, AegisResult, SessionState};

pub struct SessionStateMachine {
    current: SessionState,
    history: Vec<(SessionState, chrono::DateTime<chrono::Utc>)>,
}

impl SessionStateMachine {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            current: SessionState::Created,
            history: vec![(SessionState::Created, now)],
        }
    }

    pub fn current(&self) -> &SessionState {
        &self.current
    }

    pub fn transition(&mut self, target: SessionState) -> AegisResult<()> {
        if self.current.can_transition_to(&target) {
            self.current = target;
            self.history.push((target, chrono::Utc::now()));
            Ok(())
        } else {
            Err(AegisError::Session(format!(
                "invalid transition from {:?} to {:?}",
                self.current, target
            )))
        }
    }

    pub fn history(&self) -> &[(SessionState, chrono::DateTime<chrono::Utc>)] {
        &self.history
    }

    pub fn is_active(&self) -> bool {
        self.current.is_active()
    }

    pub fn is_terminal(&self) -> bool {
        self.current.is_terminal()
    }

    pub fn duration_in_current_state(&self) -> chrono::Duration {
        if let Some((_, timestamp)) = self.history.last() {
            chrono::Utc::now() - *timestamp
        } else {
            chrono::Duration::zero()
        }
    }
}

impl Default for SessionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

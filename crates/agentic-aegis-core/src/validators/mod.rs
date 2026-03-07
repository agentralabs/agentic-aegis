pub mod semantic_validator;
pub mod syntax_validator;
pub mod token_validator;
pub mod type_validator;

use async_trait::async_trait;

use crate::types::{AegisResult, StreamingValidation, ValidationContext};

#[async_trait]
pub trait StreamingValidator: Send + Sync {
    async fn validate_chunk(
        &self,
        context: &ValidationContext,
        chunk: &str,
    ) -> AegisResult<StreamingValidation>;

    fn can_continue(&self, validation: &StreamingValidation) -> bool {
        !validation.should_stop
    }

    fn name(&self) -> &'static str;
}

pub use semantic_validator::SemanticValidator;
pub use syntax_validator::SyntaxValidator;
pub use token_validator::TokenValidator;
pub use type_validator::TypeValidator;

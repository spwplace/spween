//! Error types for spween parsing and runtime.

use thiserror::Error;

use crate::Span;

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("invalid frontmatter YAML: {message}")]
    InvalidFrontmatter { message: String, span: Span },

    #[error("invalid condition: {message}")]
    InvalidCondition { message: String, span: Span },

    #[error("invalid effect: {message}")]
    InvalidEffect { message: String, span: Span },

    #[error("unexpected end of file")]
    UnexpectedEof { span: Span },
}

impl ParseError {
    /// Returns the source span where this error occurred.
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => span.clone(),
            ParseError::InvalidFrontmatter { span, .. } => span.clone(),
            ParseError::InvalidCondition { span, .. } => span.clone(),
            ParseError::InvalidEffect { span, .. } => span.clone(),
            ParseError::UnexpectedEof { span } => span.clone(),
        }
    }
}

/// Errors that can occur during runtime execution.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("unknown passage: {0}")]
    UnknownPassage(String),

    #[error("no current passage")]
    NoCurrentPassage,

    #[error("invalid choice index: {index} (only {available} choices available)")]
    InvalidChoiceIndex { index: usize, available: usize },

    #[error("choice condition not met")]
    ConditionNotMet,

    #[error("effect handler error: {0}")]
    EffectError(String),

    #[error("scene has ended")]
    SceneEnded,
}

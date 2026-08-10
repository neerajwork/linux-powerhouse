//! Validated execution boundary.
//!
//! This crate is intentionally small at the bootstrap stage. Actual Linux
//! backends will be introduced only after the Tool Registry and policy
//! contracts are covered by tests.

use policy_engine::{Decision, PolicyContext, evaluate};
use powerhouse_core::{ExecutionId, OperationStatus};
use thiserror::Error;
use tool_registry::ToolDefinition;

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("tool execution denied by policy")]
    Denied,
    #[error("user confirmation is required")]
    ConfirmationRequired,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub execution_id: ExecutionId,
    pub status: OperationStatus,
}

/// Validate an operation before any operating-system side effect is allowed.
///
/// This function deliberately does not execute commands yet. It establishes
/// the security boundary that future Linux backends must pass through.
pub fn authorize(
    tool: &ToolDefinition,
    context: &PolicyContext,
) -> Result<ExecutionResult, ExecutionError> {
    match evaluate(tool, context) {
        Decision::Allow => Ok(ExecutionResult {
            execution_id: ExecutionId::new(),
            status: OperationStatus::Success,
        }),
        Decision::RequireConfirmation => Err(ExecutionError::ConfirmationRequired),
        Decision::Deny => Err(ExecutionError::Denied),
    }
}

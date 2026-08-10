//! Validated execution boundary.
//!
//! All operating-system side effects must pass through policy evaluation
//! before a backend is invoked. Linux Powerhouse never gives the AI a raw
//! shell as an execution primitive.

use policy_engine::{Decision, PolicyContext, evaluate};
use powerhouse_core::{ExecutionId, OperationStatus};
use system_status::SystemStatus;
use thiserror::Error;
use tool_registry::{ToolDefinition, system_status_tool};

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("tool execution denied by policy")]
    Denied,
    #[error("user confirmation is required")]
    ConfirmationRequired,
    #[error("system status backend failed: {0}")]
    SystemStatus(#[from] system_status::SystemStatusError),
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub execution_id: ExecutionId,
    pub status: OperationStatus,
}

#[derive(Debug, Clone)]
pub struct SystemStatusExecution {
    pub execution: ExecutionResult,
    pub status: SystemStatus,
}

/// Validate an operation before any operating-system side effect is allowed.
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

/// Execute the first built-in Linux backend through the same policy boundary
/// used by every future Powerhouse capability.
pub fn execute_system_status(
    context: &PolicyContext,
) -> Result<SystemStatusExecution, ExecutionError> {
    let tool = system_status_tool();
    let execution = authorize(&tool, context)?;
    let status = system_status::collect()?;

    Ok(SystemStatusExecution { execution, status })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_status_executes_for_ai_without_confirmation() {
        let context = PolicyContext {
            ai_requested: true,
            user_confirmed: false,
        };

        let result = execute_system_status(&context).expect("system status should be readable");
        assert_eq!(result.execution.status, OperationStatus::Success);
        assert!(!result.status.kernel_version.is_empty());
        assert!(result.status.memory_total_bytes > 0);
    }
}

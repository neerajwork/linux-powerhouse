//! Validated execution boundary.
//!
//! All operating-system side effects must pass through policy evaluation
//! before a backend is invoked. Linux Powerhouse never gives the AI a raw
//! shell as an execution primitive.

use health_status::HealthSnapshot;
use monitoring::{Monitor, MonitorSnapshot};
use policy_engine::{Decision, PolicyContext, evaluate};
use powerhouse_core::{ExecutionId, OperationStatus};
use storage_intelligence::{ScanLimits, StorageAnalysis};
use system_status::SystemStatus;
use thiserror::Error;
use tool_registry::{
    ToolDefinition, health_status_tool, monitoring_status_tool, network_status_tool,
    process_status_tool, storage_intelligence_tool, storage_status_tool, system_status_tool,
};

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("tool execution denied by policy")]
    Denied,
    #[error("user confirmation is required")]
    ConfirmationRequired,
    #[error("system status backend failed: {0}")]
    SystemStatus(#[from] system_status::SystemStatusError),
    #[error("storage status backend failed: {0}")]
    StorageStatus(#[from] storage_status::StorageStatusError),
    #[error("storage intelligence backend failed: {0}")]
    StorageIntelligence(#[from] storage_intelligence::StorageIntelligenceError),
    #[error("process status backend failed: {0}")]
    ProcessStatus(#[from] process_status::ProcessStatusError),
    #[error("network status backend failed: {0}")]
    NetworkStatus(#[from] network_status::NetworkStatusError),
    #[error("monitoring backend failed: {0}")]
    Monitoring(#[from] monitoring::MonitoringError),
    #[error("health evaluation failed: {0}")]
    Health(#[from] health_status::HealthStatusError),
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

pub fn execute_system_status(
    context: &PolicyContext,
) -> Result<SystemStatusExecution, ExecutionError> {
    let execution = authorize(&system_status_tool(), context)?;
    let status = system_status::collect()?;
    Ok(SystemStatusExecution { execution, status })
}

pub fn execute_storage_status(
    context: &PolicyContext,
) -> Result<Vec<storage_status::FilesystemStatus>, ExecutionError> {
    authorize(&storage_status_tool(), context)?;
    Ok(storage_status::collect()?)
}

pub fn execute_storage_analysis(
    context: &PolicyContext,
    path: impl AsRef<std::path::Path>,
    limits: ScanLimits,
) -> Result<StorageAnalysis, ExecutionError> {
    authorize(&storage_intelligence_tool(), context)?;
    Ok(storage_intelligence::analyze_with_limits(path, limits)?)
}

pub fn execute_process_status(
    context: &PolicyContext,
) -> Result<Vec<process_status::ProcessInfo>, ExecutionError> {
    authorize(&process_status_tool(), context)?;
    Ok(process_status::collect(50)?)
}

pub fn execute_network_status(
    context: &PolicyContext,
) -> Result<Vec<network_status::NetworkInterface>, ExecutionError> {
    authorize(&network_status_tool(), context)?;
    Ok(network_status::collect()?)
}

pub fn execute_monitoring_snapshot(
    context: &PolicyContext,
    monitor: &mut Monitor,
) -> Result<MonitorSnapshot, ExecutionError> {
    authorize(&monitoring_status_tool(), context)?;
    Ok(monitor.snapshot()?)
}

pub fn execute_health_status(
    context: &PolicyContext,
    monitoring: Option<&MonitorSnapshot>,
    max_storage_usage: Option<u8>,
) -> Result<HealthSnapshot, ExecutionError> {
    authorize(&health_status_tool(), context)?;
    Ok(health_status::evaluate(monitoring, max_storage_usage)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ai_context() -> PolicyContext {
        PolicyContext {
            ai_requested: true,
            user_confirmed: false,
        }
    }

    fn confirmed_context() -> PolicyContext {
        PolicyContext {
            ai_requested: true,
            user_confirmed: true,
        }
    }

    #[test]
    fn read_only_dashboard_capabilities_execute_for_ai() {
        let context = ai_context();
        assert!(
            !execute_system_status(&context)
                .unwrap()
                .status
                .kernel_version
                .is_empty()
        );
        assert!(!execute_storage_status(&context).unwrap().is_empty());
        assert!(!execute_process_status(&context).unwrap().is_empty());
        assert!(!execute_network_status(&context).unwrap().is_empty());
        let mut monitor = Monitor::new();
        assert!(execute_monitoring_snapshot(&context, &mut monitor).is_ok());
        let snapshot = monitor.snapshot().unwrap();
        assert!(execute_health_status(&context, Some(&snapshot), Some(50)).is_ok());
    }

    #[test]
    fn storage_analysis_requires_user_confirmation() {
        let denied = execute_storage_analysis(
            &ai_context(),
            "/tmp",
            ScanLimits {
                max_depth: 1,
                max_entries: 10,
                top_n: 5,
            },
        );
        assert!(matches!(denied, Err(ExecutionError::ConfirmationRequired)));

        let result = execute_storage_analysis(
            &confirmed_context(),
            "/tmp",
            ScanLimits {
                max_depth: 1,
                max_entries: 10,
                top_n: 5,
            },
        );
        assert!(result.is_ok());
    }
}

//! Capability-oriented Tool Registry.
//!
//! The registry describes what Linux Powerhouse can do. It intentionally
//! separates user-facing capabilities from the commands or libraries used to
//! implement those capabilities.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    ReadOnly,
    Low,
    Reversible,
    Moderate,
    Destructive,
    SystemCritical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub risk: RiskLevel,
    pub permissions: Vec<String>,
    pub ai_autonomous: bool,
}

impl ToolDefinition {
    pub fn new(
        id: impl Into<String>,
        version: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        risk: RiskLevel,
    ) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            name: name.into(),
            description: description.into(),
            category: category.into(),
            risk,
            permissions: Vec::new(),
            ai_autonomous: false,
        }
    }
}

fn read_only_tool(id: &str, name: &str, description: &str, category: &str) -> ToolDefinition {
    let mut tool = ToolDefinition::new(
        id,
        "0.1.0",
        name,
        description,
        category,
        RiskLevel::ReadOnly,
    );
    tool.permissions.push("system.read".into());
    tool.ai_autonomous = true;
    tool
}

pub fn system_status_tool() -> ToolDefinition {
    read_only_tool(
        "system.status",
        "System Status",
        "Read basic operating-system and hardware status information.",
        "system",
    )
}

pub fn unified_system_intelligence_tool() -> ToolDefinition {
    let mut tool = read_only_tool(
        "system.intelligence",
        "Unified System Intelligence",
        "Aggregate bounded storage, process, network, and service signals into one system health snapshot.",
        "system",
    );
    tool.permissions.push("filesystem.read".into());
    tool
}

pub fn storage_status_tool() -> ToolDefinition {
    read_only_tool(
        "storage.status",
        "Storage Status",
        "Read filesystem capacity and mount information.",
        "storage",
    )
}

pub fn storage_intelligence_tool() -> ToolDefinition {
    let mut tool = ToolDefinition::new(
        "storage.analyze",
        "0.1.0",
        "Storage Intelligence",
        "Analyze a user-selected directory with bounded, read-only traversal to identify major space consumers.",
        "storage",
        RiskLevel::Low,
    );
    tool.permissions.push("filesystem.read".into());
    tool.ai_autonomous = false;
    tool
}

pub fn process_status_tool() -> ToolDefinition {
    read_only_tool(
        "process.status",
        "Process Status",
        "Read a bounded snapshot of running processes and memory usage.",
        "processes",
    )
}

pub fn process_intelligence_tool() -> ToolDefinition {
    let mut tool = ToolDefinition::new(
        "process.analyze",
        "0.1.0",
        "Process Intelligence",
        "Analyze a bounded process snapshot for resource concentration, process hierarchy, and deterministic anomalies.",
        "processes",
        RiskLevel::Low,
    );
    tool.permissions.push("system.read".into());
    tool.ai_autonomous = false;
    tool
}

pub fn network_status_tool() -> ToolDefinition {
    read_only_tool(
        "network.status",
        "Network Status",
        "Read network interface state and traffic counters.",
        "network",
    )
}

pub fn network_intelligence_tool() -> ToolDefinition {
    let mut tool = ToolDefinition::new(
        "network.analyze",
        "0.1.0",
        "Network Intelligence",
        "Analyze bounded TCP socket state, listening ports, established connections, and deterministic network signals.",
        "network",
        RiskLevel::Low,
    );
    tool.permissions.push("network.read".into());
    tool.ai_autonomous = false;
    tool
}

pub fn service_intelligence_tool() -> ToolDefinition {
    let mut tool = ToolDefinition::new(
        "service.analyze",
        "0.1.0",
        "Service Intelligence",
        "Analyze bounded systemd service state and deterministic failure signals without mutating services.",
        "services",
        RiskLevel::Low,
    );
    tool.permissions.push("system.read".into());
    tool.ai_autonomous = false;
    tool
}

pub fn monitoring_status_tool() -> ToolDefinition {
    read_only_tool(
        "monitoring.status",
        "Realtime Monitoring",
        "Read live CPU, memory, swap and network throughput metrics.",
        "monitoring",
    )
}

pub fn health_status_tool() -> ToolDefinition {
    read_only_tool(
        "health.status",
        "System Health",
        "Evaluate deterministic health and anomaly signals from trusted system metrics.",
        "health",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_tools_are_read_only_and_ai_safe() {
        for tool in [
            system_status_tool(),
            unified_system_intelligence_tool(),
            storage_status_tool(),
            process_status_tool(),
            network_status_tool(),
            monitoring_status_tool(),
            health_status_tool(),
        ] {
            assert_eq!(tool.risk, RiskLevel::ReadOnly);
            assert!(tool.ai_autonomous);
            assert!(tool.permissions.iter().any(|p| p == "system.read"));
        }
    }

    #[test]
    fn unified_intelligence_requires_read_permissions() {
        let tool = unified_system_intelligence_tool();
        assert_eq!(tool.risk, RiskLevel::ReadOnly);
        assert!(tool.ai_autonomous);
        assert!(tool.permissions.iter().any(|p| p == "system.read"));
        assert!(tool.permissions.iter().any(|p| p == "filesystem.read"));
    }

    #[test]
    fn analysis_tools_require_explicit_user_authorization() {
        let storage = storage_intelligence_tool();
        assert_eq!(storage.risk, RiskLevel::Low);
        assert!(!storage.ai_autonomous);
        assert!(storage.permissions.iter().any(|p| p == "filesystem.read"));

        let process = process_intelligence_tool();
        assert_eq!(process.risk, RiskLevel::Low);
        assert!(!process.ai_autonomous);
        assert!(process.permissions.iter().any(|p| p == "system.read"));

        let network = network_intelligence_tool();
        assert_eq!(network.risk, RiskLevel::Low);
        assert!(!network.ai_autonomous);
        assert!(network.permissions.iter().any(|p| p == "network.read"));

        let service = service_intelligence_tool();
        assert_eq!(service.risk, RiskLevel::Low);
        assert!(!service.ai_autonomous);
        assert!(service.permissions.iter().any(|p| p == "system.read"));
    }
}

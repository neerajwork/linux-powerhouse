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
pub fn storage_status_tool() -> ToolDefinition {
    read_only_tool(
        "storage.status",
        "Storage Status",
        "Read filesystem capacity and mount information.",
        "storage",
    )
}
pub fn process_status_tool() -> ToolDefinition {
    read_only_tool(
        "process.status",
        "Process Status",
        "Read a bounded snapshot of running processes and memory usage.",
        "processes",
    )
}
pub fn network_status_tool() -> ToolDefinition {
    read_only_tool(
        "network.status",
        "Network Status",
        "Read network interface state and traffic counters.",
        "network",
    )
}
pub fn monitoring_status_tool() -> ToolDefinition {
    read_only_tool(
        "monitoring.status",
        "Realtime Monitoring",
        "Read live CPU, memory, swap and network throughput metrics.",
        "monitoring",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_tools_are_read_only_and_ai_safe() {
        for tool in [
            system_status_tool(),
            storage_status_tool(),
            process_status_tool(),
            network_status_tool(),
            monitoring_status_tool(),
        ] {
            assert_eq!(tool.risk, RiskLevel::ReadOnly);
            assert!(tool.ai_autonomous);
            assert!(tool.permissions.iter().any(|p| p == "system.read"));
        }
    }
}

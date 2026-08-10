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

/// The first built-in capability used to validate the registry architecture.
pub fn system_status_tool() -> ToolDefinition {
    let mut tool = ToolDefinition::new(
        "system.status",
        "0.1.0",
        "System Status",
        "Read basic operating-system and hardware status information.",
        "system",
        RiskLevel::ReadOnly,
    );
    tool.permissions.push("system.read".into());
    tool.ai_autonomous = true;
    tool
}

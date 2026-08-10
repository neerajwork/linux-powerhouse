//! Deterministic policy evaluation for AI and user-requested operations.
//!
//! The policy engine is intentionally independent from any AI provider.

use serde::{Deserialize, Serialize};
use tool_registry::{RiskLevel, ToolDefinition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Allow,
    RequireConfirmation,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    pub ai_requested: bool,
    pub user_confirmed: bool,
}

pub fn evaluate(tool: &ToolDefinition, context: &PolicyContext) -> Decision {
    match tool.risk {
        RiskLevel::ReadOnly | RiskLevel::Low if tool.ai_autonomous && context.ai_requested => {
            Decision::Allow
        }
        RiskLevel::ReadOnly => Decision::Allow,
        RiskLevel::Low | RiskLevel::Reversible | RiskLevel::Moderate => {
            if context.user_confirmed {
                Decision::Allow
            } else {
                Decision::RequireConfirmation
            }
        }
        RiskLevel::Destructive | RiskLevel::SystemCritical => {
            if context.user_confirmed {
                Decision::Allow
            } else {
                Decision::RequireConfirmation
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tool_registry::system_status_tool;

    #[test]
    fn read_only_tool_can_be_ai_requested() {
        let tool = system_status_tool();
        let context = PolicyContext {
            ai_requested: true,
            user_confirmed: false,
        };
        assert_eq!(evaluate(&tool, &context), Decision::Allow);
    }

    #[test]
    fn destructive_tool_requires_confirmation() {
        let tool = ToolDefinition::new(
            "files.delete",
            "0.1.0",
            "Delete Files",
            "Delete selected files.",
            "files",
            RiskLevel::Destructive,
        );
        let context = PolicyContext {
            ai_requested: true,
            user_confirmed: false,
        };
        assert_eq!(evaluate(&tool, &context), Decision::RequireConfirmation);
    }
}

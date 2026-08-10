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

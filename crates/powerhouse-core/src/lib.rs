//! Shared domain primitives for Linux Powerhouse.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an individual tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutionId(Uuid);

impl ExecutionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level outcome of an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    Success,
    PartialSuccess,
    Failed,
    Cancelled,
    TimedOut,
    Denied,
    RequiresAuthorization,
}

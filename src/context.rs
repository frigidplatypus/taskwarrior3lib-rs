//! Context management
//!
//! This module handles Taskwarrior contexts for organizing work contexts.

use serde::{Deserialize, Serialize};

/// Named filters for organizing work contexts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    /// Context name (unique identifier)
    pub name: String,
    /// Filter expression defining the context
    pub filter: String,
    /// Human-readable description
    pub description: Option<String>,
}

impl Context {
    pub fn new(name: String, filter: String) -> Self {
        Self {
            name,
            filter,
            description: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

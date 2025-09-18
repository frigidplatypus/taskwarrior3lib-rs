//! Annotation types for tasks
//!
//! This module contains annotation and priority related types.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Notes attached to tasks with timestamps
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    /// When annotation was added
    pub entry: DateTime<Utc>,
    /// Annotation text
    pub description: String,
}

impl Annotation {
    /// Create a new annotation with current timestamp
    pub fn new(description: String) -> Self {
        Self {
            entry: Utc::now(),
            description,
        }
    }
    
    /// Create an annotation with specific timestamp
    pub fn with_timestamp(description: String, entry: DateTime<Utc>) -> Self {
        Self {
            entry,
            description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_annotation() {
        let annotation = Annotation::new("Test note".to_string());
        assert_eq!(annotation.description, "Test note");
        assert!(annotation.entry <= Utc::now());
    }

    #[test]
    fn test_annotation_with_timestamp() {
        let timestamp = Utc::now();
        let annotation = Annotation::with_timestamp("Test note".to_string(), timestamp);
        assert_eq!(annotation.entry, timestamp);
    }
}

//! Recurrence pattern definitions
//!
//! This module contains types for handling recurring tasks.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Recurrence pattern for recurring tasks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecurrencePattern {
    /// The recurrence specification (e.g., "daily", "weekly", "monthly")
    pub pattern: String,
    /// Whether this is a periodic recurrence (true) or fixed recurrence (false)
    pub periodic: bool,
}

impl RecurrencePattern {
    /// Create a new recurrence pattern
    pub fn new(pattern: String) -> Self {
        Self {
            pattern,
            periodic: false,
        }
    }

    /// Create a periodic recurrence pattern
    pub fn periodic(pattern: String) -> Self {
        Self {
            pattern,
            periodic: true,
        }
    }

    /// Parse a recurrence string into a pattern
    pub fn parse(recur_str: &str) -> Result<Self, RecurrenceError> {
        if recur_str.is_empty() {
            return Err(RecurrenceError::Empty);
        }

        // Check for periodic indicator using strip_prefix
        let (pattern, periodic) = if let Some(stripped) = recur_str.strip_prefix('P') {
            (stripped.to_string(), true)
        } else {
            (recur_str.to_string(), false)
        };

        // Validate pattern
        if Self::is_valid_pattern(&pattern) {
            Ok(Self { pattern, periodic })
        } else {
            Err(RecurrenceError::InvalidPattern(pattern))
        }
    }

    /// Check if a pattern string is valid
    fn is_valid_pattern(pattern: &str) -> bool {
        // Common recurrence patterns
        matches!(
            pattern,
            "daily" | "weekly" | "monthly" | "quarterly" | "yearly" | "weekdays" | "weekends"
        ) || pattern.ends_with("d")
            || pattern.ends_with("w")
            || pattern.ends_with("m")
            || pattern.ends_with("q")
            || pattern.ends_with("y")
    }

    /// Get the base unit of recurrence
    pub fn get_unit(&self) -> RecurrenceUnit {
        // Match specific common patterns first
        match self.pattern.as_str() {
            "daily" => RecurrenceUnit::Day,
            "weekly" => RecurrenceUnit::Week,
            "monthly" => RecurrenceUnit::Month,
            "quarterly" => RecurrenceUnit::Quarter,
            "yearly" | "annually" => RecurrenceUnit::Year,
            _ => {
                // Check word-based patterns
                if self.pattern.contains("day") {
                    RecurrenceUnit::Day
                } else if self.pattern.contains("week") {
                    RecurrenceUnit::Week
                } else if self.pattern.contains("month") {
                    RecurrenceUnit::Month
                } else if self.pattern.contains("quarter") {
                    RecurrenceUnit::Quarter
                } else if self.pattern.contains("year") {
                    RecurrenceUnit::Year
                // Then check suffix-based patterns
                } else if self.pattern.ends_with("d") {
                    RecurrenceUnit::Day
                } else if self.pattern.ends_with("w") {
                    RecurrenceUnit::Week
                } else if self.pattern.ends_with("m") {
                    RecurrenceUnit::Month
                } else if self.pattern.ends_with("q") {
                    RecurrenceUnit::Quarter
                } else if self.pattern.ends_with("y") {
                    RecurrenceUnit::Year
                } else {
                    RecurrenceUnit::Day // Default fallback
                }
            }
        }
    }
}

/// Units of recurrence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecurrenceUnit {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

/// Errors that can occur when parsing recurrence patterns
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RecurrenceError {
    #[error("Recurrence pattern cannot be empty")]
    Empty,
    #[error("Invalid recurrence pattern: {0}")]
    InvalidPattern(String),
}

impl fmt::Display for RecurrencePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.periodic {
            write!(f, "P{pattern}", pattern = self.pattern)
        } else {
            write!(f, "{pattern}", pattern = self.pattern)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_recurrence_pattern() {
        let pattern = RecurrencePattern::new("daily".to_string());
        assert_eq!(pattern.pattern, "daily");
        assert!(!pattern.periodic);
    }

    #[test]
    fn test_periodic_recurrence_pattern() {
        let pattern = RecurrencePattern::periodic("weekly".to_string());
        assert_eq!(pattern.pattern, "weekly");
        assert!(pattern.periodic);
    }

    #[test]
    fn test_parse_recurrence_pattern() {
        let pattern = RecurrencePattern::parse("daily").unwrap();
        assert_eq!(pattern.pattern, "daily");
        assert!(!pattern.periodic);

        let periodic_pattern = RecurrencePattern::parse("Pweekly").unwrap();
        assert_eq!(periodic_pattern.pattern, "weekly");
        assert!(periodic_pattern.periodic);
    }

    #[test]
    fn test_invalid_pattern() {
        let result = RecurrencePattern::parse("");
        assert!(result.is_err());
        matches!(result.unwrap_err(), RecurrenceError::Empty);
    }

    #[test]
    fn test_get_unit() {
        let daily = RecurrencePattern::new("daily".to_string());
        assert_eq!(daily.get_unit(), RecurrenceUnit::Day);

        let weekly = RecurrencePattern::new("weekly".to_string());
        assert_eq!(weekly.get_unit(), RecurrenceUnit::Week);

        let monthly = RecurrencePattern::new("3m".to_string());
        assert_eq!(monthly.get_unit(), RecurrenceUnit::Month);
    }

    #[test]
    fn test_display() {
        let normal = RecurrencePattern::new("daily".to_string());
        assert_eq!(format!("{normal}"), "daily");

        let periodic = RecurrencePattern::periodic("weekly".to_string());
        assert_eq!(format!("{periodic}"), "Pweekly");
    }
}

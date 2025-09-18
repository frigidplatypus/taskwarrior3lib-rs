//! Relative date calculation utilities
//!
//! This module provides utilities for calculating relative dates.

use chrono::{DateTime, Utc, Duration};
use crate::error::DateError;

/// Calculate a future date by adding duration
pub fn add_duration(base: DateTime<Utc>, duration: Duration) -> Result<DateTime<Utc>, DateError> {
    Ok(base + duration)
}

/// Calculate a past date by subtracting duration
pub fn subtract_duration(base: DateTime<Utc>, duration: Duration) -> Result<DateTime<Utc>, DateError> {
    Ok(base - duration)
}

/// Parse duration string (e.g., "1week", "3days")
pub fn parse_duration(duration_str: &str) -> Result<Duration, DateError> {
    // This is a simplified implementation
    // Full implementation would be in the date parser
    let duration_str = duration_str.trim();
    
    if duration_str.ends_with("day") || duration_str.ends_with("days") || duration_str.ends_with("d") {
        let num_str = duration_str.trim_end_matches("day").trim_end_matches("days").trim_end_matches("d");
        let num: i64 = num_str.parse().map_err(|_| DateError::InvalidRelative {
            expression: duration_str.to_string(),
        })?;
        Ok(Duration::days(num))
    } else if duration_str.ends_with("week") || duration_str.ends_with("weeks") || duration_str.ends_with("w") {
        let num_str = duration_str.trim_end_matches("week").trim_end_matches("weeks").trim_end_matches("w");
        let num: i64 = num_str.parse().map_err(|_| DateError::InvalidRelative {
            expression: duration_str.to_string(),
        })?;
        Ok(Duration::weeks(num))
    } else {
        Err(DateError::InvalidRelative {
            expression: duration_str.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_duration() {
        let base = Utc::now();
        let future = add_duration(base, Duration::days(1)).unwrap();
        assert!(future > base);
    }

    #[test]
    fn test_parse_duration() {
        let duration = parse_duration("3days").unwrap();
        assert_eq!(duration, Duration::days(3));
        
        let duration = parse_duration("1week").unwrap();
        assert_eq!(duration, Duration::weeks(1));
    }
}

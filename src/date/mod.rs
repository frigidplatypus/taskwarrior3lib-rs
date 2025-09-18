//! Date parsing and handling
//!
//! This module provides comprehensive date parsing functionality including
//! ISO-8601 formats, named synonyms, and relative date calculations.

pub mod parser;
pub mod synonyms;
pub mod relative;

use chrono::{DateTime, Utc};
use crate::error::DateError;

// Re-export main types
pub use parser::DateParser;
pub use synonyms::DateSynonym;

/// Trait for date parsing functionality
pub trait DateParsing {
    /// Parse a date string in various formats
    fn parse_date(&self, input: &str) -> Result<DateTime<Utc>, DateError>;
    
    /// Parse a date synonym (now, today, monday, etc.)
    fn parse_synonym(&self, synonym: &str) -> Result<DateTime<Utc>, DateError>;
    
    /// Parse with custom format
    fn parse_date_with_format(&self, input: &str, format: &str) -> Result<DateTime<Utc>, DateError>;
    
    /// Calculate relative date from base
    fn calculate_relative_date(&self, base: DateTime<Utc>, expression: &str) -> Result<DateTime<Utc>, DateError>;
    
    /// Format date for display
    fn format_date(&self, date: DateTime<Utc>) -> String;
    
    /// Get supported synonyms
    fn get_supported_synonyms(&self) -> Vec<String>;
}

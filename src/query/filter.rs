//! Query filter types
//!
//! This module contains various filter types used in task queries.

use std::collections::HashSet;
use chrono::{DateTime, Utc};

/// Project filter variants
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectFilter {
    /// Exact project match
    Exact(String),
    /// Equals - alias for Exact for API consistency
    Equals(String),
    /// Hierarchical match (includes sub-projects)
    Hierarchy(String),
    /// Multiple project filter
    Multiple(Vec<String>),
    /// No project filter (orphaned tasks)
    None,
}

/// Tag filter configuration
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TagFilter {
    /// Tags that must be present
    pub include: HashSet<String>,
    /// Tags that must not be present
    pub exclude: HashSet<String>,
}

impl TagFilter {
    /// Create a new empty tag filter
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a filter that includes specific tags
    pub fn include_tags(tags: Vec<String>) -> Self {
        Self {
            include: tags.into_iter().collect(),
            exclude: HashSet::new(),
        }
    }
    
    /// Create a filter that excludes specific tags
    pub fn exclude_tags(tags: Vec<String>) -> Self {
        Self {
            include: HashSet::new(),
            exclude: tags.into_iter().collect(),
        }
    }
    
    /// Create a filter that requires a specific tag
    pub fn has_tag(tag: String) -> Self {
        Self {
            include: vec![tag].into_iter().collect(),
            exclude: HashSet::new(),
        }
    }
    
    // Backwards compatibility handled by keeping the single `has_tag` method above.
    
    /// Check if this filter matches a set of task tags
    pub fn matches(&self, task_tags: &HashSet<String>) -> bool {
        // Check if any required tags are missing
        if !self.include.is_empty() && !self.include.iter().any(|tag| task_tags.contains(tag)) {
            return false;
        }
        
        // Check if any excluded tags are present
        if self.exclude.iter().any(|tag| task_tags.contains(tag)) {
            return false;
        }
        
        true
    }
    
    /// Check if the filter has any conditions
    pub fn is_empty(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }
}

/// Date-based filter types
#[derive(Debug, Clone, PartialEq)]
pub enum DateFilter {
    /// Due before specified date
    DueBefore(DateTime<Utc>),
    /// Due after specified date
    DueAfter(DateTime<Utc>),
    /// Due between two dates
    DueBetween(DateTime<Utc>, DateTime<Utc>),
    /// Scheduled before specified date
    ScheduledBefore(DateTime<Utc>),
    /// Scheduled after specified date
    ScheduledAfter(DateTime<Utc>),
    /// Modified before specified date
    ModifiedBefore(DateTime<Utc>),
    /// Modified after specified date
    ModifiedAfter(DateTime<Utc>),
    /// Entry before specified date
    EntryBefore(DateTime<Utc>),
    /// Entry after specified date
    EntryAfter(DateTime<Utc>),
}

impl DateFilter {
    /// Check if this filter matches a task's dates
    pub fn matches(&self, due: Option<DateTime<Utc>>, scheduled: Option<DateTime<Utc>>, 
                   modified: Option<DateTime<Utc>>, entry: DateTime<Utc>) -> bool {
        match self {
            DateFilter::DueBefore(date) => due.is_some_and(|d| d < *date),
            DateFilter::DueAfter(date) => due.is_some_and(|d| d > *date),
            DateFilter::DueBetween(start, end) => {
                due.is_some_and(|d| d >= *start && d <= *end)
            },
            DateFilter::ScheduledBefore(date) => scheduled.is_some_and(|d| d < *date),
            DateFilter::ScheduledAfter(date) => scheduled.is_some_and(|d| d > *date),
            DateFilter::ModifiedBefore(date) => modified.is_some_and(|d| d < *date),
            DateFilter::ModifiedAfter(date) => modified.is_some_and(|d| d > *date),
            DateFilter::EntryBefore(date) => entry < *date,
            DateFilter::EntryAfter(date) => entry > *date,
        }
    }
}

/// Sort criteria for query results
#[derive(Debug, Clone, PartialEq)]
pub struct SortCriteria {
    /// Field to sort by
    pub field: String,
    /// Sort direction (true = ascending, false = descending)
    pub ascending: bool,
}

impl SortCriteria {
    /// Priority sort (descending by default)
    pub fn priority() -> SortCriteria {
        SortCriteria {
            field: "priority".to_string(),
            ascending: false,
        }
    }
    
    /// Create ascending sort criteria
    pub fn ascending(field: &str) -> Self {
        Self {
            field: field.to_string(),
            ascending: true,
        }
    }
    
    /// Create descending sort criteria
    pub fn descending(field: &str) -> Self {
        Self {
            field: field.to_string(),
            ascending: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_filter_include() {
        let filter = TagFilter::include_tags(vec!["work".to_string(), "urgent".to_string()]);
        
        let mut task_tags = HashSet::new();
        task_tags.insert("work".to_string());
        task_tags.insert("important".to_string());
        
        assert!(filter.matches(&task_tags));
    }

    #[test]
    fn test_tag_filter_exclude() {
        let filter = TagFilter::exclude_tags(vec!["someday".to_string()]);
        
        let mut task_tags = HashSet::new();
        task_tags.insert("work".to_string());
        
        assert!(filter.matches(&task_tags));
        
        task_tags.insert("someday".to_string());
        assert!(!filter.matches(&task_tags));
    }

    #[test]
    fn test_date_filter_due_before() {
        let now = Utc::now();
        let past = now - chrono::Duration::days(1);
        let future = now + chrono::Duration::days(1);
        
        let filter = DateFilter::DueBefore(now);
        
        assert!(filter.matches(Some(past), None, None, now));
        assert!(!filter.matches(Some(future), None, None, now));
        assert!(!filter.matches(None, None, None, now));
    }

    #[test]
    fn test_sort_criteria() {
        let asc = SortCriteria::ascending("due");
        assert!(asc.ascending);
        assert_eq!(asc.field, "due");
        
        let desc = SortCriteria::descending("priority");
        assert!(!desc.ascending);
        assert_eq!(desc.field, "priority");
    }
}

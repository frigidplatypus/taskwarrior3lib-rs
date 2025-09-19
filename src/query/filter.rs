//! Query filter types and helpers
//!
//! This module defines the filter types used by TaskQuery and small
//! convenience helpers used by builders and backends.

use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Project filter variants
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectFilter {
    /// Exact match (alias of Equals)
    Exact(String),
    /// Equals a specific project name
    Equals(String),
    /// Matches a project hierarchy prefix (e.g., "Work" matches "Work.Client")
    Hierarchy(String),
    /// Matches if the project's value is one of the provided set
    Multiple(Vec<String>),
    /// Matches when project is None
    None,
}

/// Tag inclusion and exclusion filter
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TagFilter {
    pub include: HashSet<String>,
    pub exclude: HashSet<String>,
//! Query filter types and helpers
//!
//! This module defines the filter types used by TaskQuery and small
//! convenience helpers used by builders and backends.

use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Project filter variants
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectFilter {
    /// Exact match (alias of Equals)
    Exact(String),
    /// Equals a specific project name
    Equals(String),
    /// Matches a project hierarchy prefix (e.g., "Work" matches "Work.Client")
    Hierarchy(String),
    /// Matches if the project's value is one of the provided set
    Multiple(Vec<String>),
    /// Matches when project is None
    None,
}

/// Tag inclusion and exclusion filter
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TagFilter {
    pub include: HashSet<String>,
    pub exclude: HashSet<String>,
}

impl TagFilter {
    /// Convenience to construct a filter that requires a single tag to be present
    pub fn has_tag(tag: String) -> Self {
        let mut filter = Self::default();
        filter.include.insert(tag);
        filter
    }

    /// Convenience to include many tags (match any)
    pub fn include_tags<T: Into<String>>(tags: impl IntoIterator<Item = T>) -> Self {
        let mut filter = Self::default();
        for t in tags {
            filter.include.insert(t.into());
        }
        filter
    }

    /// Convenience to exclude many tags (match none)
    pub fn exclude_tags<T: Into<String>>(tags: impl IntoIterator<Item = T>) -> Self {
        let mut filter = Self::default();
        for t in tags {
            filter.exclude.insert(t.into());
        }
        filter
    }

    /// Returns true if the provided task tags satisfy this filter
    pub fn matches(&self, task_tags: &HashSet<String>) -> bool {
        if !self.include.is_empty() && !self.include.iter().any(|t| task_tags.contains(t)) {
            return false;
        }
        if self.exclude.iter().any(|t| task_tags.contains(t)) {
            return false;
        }
        true
    }
}

/// Date-based filter (range/point comparisons)
#[derive(Debug, Clone, PartialEq)]
pub enum DateFilter {
    DueBefore(DateTime<Utc>),
    DueAfter(DateTime<Utc>),
    DueBetween(DateTime<Utc>, DateTime<Utc>),
    ScheduledBefore(DateTime<Utc>),
    ScheduledAfter(DateTime<Utc>),
    ModifiedBefore(DateTime<Utc>),
    ModifiedAfter(DateTime<Utc>),
    EntryBefore(DateTime<Utc>),
    EntryAfter(DateTime<Utc>),
}

/// Sorting criteria for results
#[derive(Debug, Clone, PartialEq)]
pub struct SortCriteria {
    pub field: String,
    pub ascending: bool,
}

impl SortCriteria {
    /// Convenience for sorting by priority descending (highest first)
    pub fn priority() -> Self {
        Self {
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

/// Extract a simple project token from a Taskwarrior filter expression.
/// Supported syntaxes: `project:Name`, `project==Name`, `project="Name"`.
pub fn parse_project_from_filter(filter: &str) -> Option<String> {
    for token in filter.split_whitespace() {
        // token like project:Name
        if let Some(rest) = token.strip_prefix("project:") {
            return Some(rest.trim_matches('"').trim_matches('\'').to_string());
        }
        // token like project==Name or project="Name"
        if token.starts_with("project==") || token.starts_with("project=") {
            let mut val = token;
            if let Some(pos) = token.find('=') {
                val = &token[pos + 1..];
            }
            let v = val.trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_filter_basic() {
        let filter = TagFilter::has_tag("work".to_string());
        let mut tags = HashSet::new();
        assert!(!filter.matches(&tags));
        tags.insert("work".to_string());
        assert!(filter.matches(&tags));
    }

    #[test]
    fn parse_project_variants() {
        assert_eq!(parse_project_from_filter("project:Work").as_deref(), Some("Work"));
        assert_eq!(parse_project_from_filter("project==Inbox").as_deref(), Some("Inbox"));
        assert_eq!(parse_project_from_filter("project=\"Foo Bar\"").as_deref(), Some("Foo Bar"));
        assert!(parse_project_from_filter("priority:H or due.before:tomorrow").is_none());
    }
}


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

        #[test]
        fn test_parse_project_simple() {
            let f = "project:Work";
            assert_eq!(parse_project_from_filter(f).as_deref(), Some("Work"));
        }

        #[test]
        fn test_parse_project_quoted() {
            let f = "project:\"My Project\"";
            assert_eq!(parse_project_from_filter(f).as_deref(), Some("My Project"));
        }

        #[test]
        fn test_parse_project_equals() {
            let f = "project==Inbox";
            assert_eq!(parse_project_from_filter(f).as_deref(), Some("Inbox"));
        }

        #[test]
        fn test_parse_project_none() {
            let f = "priority:H or due.before:tomorrow";
            assert!(parse_project_from_filter(f).is_none());
        }
    }
        scheduled: Option<DateTime<Utc>>,
        modified: Option<DateTime<Utc>>,
        entry: DateTime<Utc>,
    ) -> bool {
        match self {
            DateFilter::DueBefore(date) => due.is_some_and(|d| d < *date),
            DateFilter::DueAfter(date) => due.is_some_and(|d| d > *date),
            DateFilter::DueBetween(start, end) => due.is_some_and(|d| d >= *start && d <= *end),
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

/// Extract a simple project:<name> token from a Taskwarrior filter expression.
/// This helper is used to interpret active context read filters and map them
/// into structured query ProjectFilter values.
pub fn parse_project_from_filter(filter: &str) -> Option<String> {
    for token in filter.split_whitespace() {
        // token like project:Name
        if let Some(rest) = token.strip_prefix("project:") {
            return Some(rest.trim_matches('"').trim_matches('\'').to_string());
        }
        // token like project=="Name" or project==Name (support several syntaxes)
        if token.starts_with("project==") || token.starts_with("project=") {
            let mut val = token;
            if let Some(pos) = token.find('=') {
                val = &token[pos + 1..];
            }
            let v = val.trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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

    #[test]
    fn test_parse_project_simple() {
        let f = "project:Work";
        assert_eq!(parse_project_from_filter(f).as_deref(), Some("Work"));
    }

    #[test]
    fn test_parse_project_quoted() {
        let f = "project:\"My Project\"";
        assert_eq!(parse_project_from_filter(f).as_deref(), Some("My Project"));
    }

    #[test]
    fn test_parse_project_equals() {
        let f = "project==Inbox";
        assert_eq!(parse_project_from_filter(f).as_deref(), Some("Inbox"));
    }

    #[test]
    fn test_parse_project_none() {
        let f = "priority:H or due.before:tomorrow";
        assert!(parse_project_from_filter(f).is_none());
    }
}
        let desc = SortCriteria::descending("priority");
        assert!(!desc.ascending);
        assert_eq!(desc.field, "priority");
    }
}

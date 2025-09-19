//! Query filter types and helpers (clean module)

use chrono::{DateTime, Utc};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectFilter {
    Exact(String),
    Equals(String),
    Hierarchy(String),
    Multiple(Vec<String>),
    None,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TagFilter {
    pub include: HashSet<String>,
    pub exclude: HashSet<String>,
}

impl TagFilter {
    pub fn has_tag(tag: String) -> Self {
        let mut filter = Self::default();
        filter.include.insert(tag);
        filter
    }
    pub fn include_tags<T: Into<String>>(tags: impl IntoIterator<Item = T>) -> Self {
        let mut filter = Self::default();
        for t in tags { filter.include.insert(t.into()); }
        filter
    }
    pub fn exclude_tags<T: Into<String>>(tags: impl IntoIterator<Item = T>) -> Self {
        let mut filter = Self::default();
        for t in tags { filter.exclude.insert(t.into()); }
        filter
    }
    pub fn matches(&self, task_tags: &HashSet<String>) -> bool {
        if !self.include.is_empty() && !self.include.iter().any(|t| task_tags.contains(t)) {
            return false;
        }
        if self.exclude.iter().any(|t| task_tags.contains(t)) { return false; }
        true
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct SortCriteria {
    pub field: String,
    pub ascending: bool,
}

impl SortCriteria {
    pub fn priority() -> Self { Self { field: "priority".into(), ascending: false } }
    pub fn ascending(field: &str) -> Self { Self { field: field.into(), ascending: true } }
    pub fn descending(field: &str) -> Self { Self { field: field.into(), ascending: false } }
}

/// Extract a simple project token from a Taskwarrior filter expression.
pub fn parse_project_from_filter(filter: &str) -> Option<String> {
    for token in filter.split_whitespace() {
        if let Some(rest) = token.strip_prefix("project:") {
            return Some(rest.trim_matches('"').trim_matches('\'').to_string());
        }
        if token.starts_with("project==") || token.starts_with("project=") {
            let mut val = token;
            if let Some(pos) = token.find('=') { val = &token[pos + 1..]; }
            let v = val.trim_matches('"').trim_matches('\'');
            if !v.is_empty() { return Some(v.to_string()); }
        }
    }
    None
}

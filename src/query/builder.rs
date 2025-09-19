//! Query builder implementation
//!
//! This module provides the TaskQueryBuilder implementation.

use crate::error::QueryError;
use crate::query::{DateFilter, ProjectFilter, SortCriteria, TagFilter, TaskQuery};
#[allow(unused_imports)]
use crate::task::{Priority, TaskStatus};
use chrono::{DateTime, Utc};

/// TaskQueryBuilder implementation
#[derive(Debug, Default)]
pub struct TaskQueryBuilderImpl {
    status: Option<TaskStatus>,
    project_filter: Option<ProjectFilter>,
    tag_filter: Option<TagFilter>,
    date_filter: Option<DateFilter>,
    sort: Option<SortCriteria>,
    limit: Option<usize>,
    offset: Option<usize>,
    filter_mode: Option<crate::query::FilterMode>,
}

/// TaskQueryBuilder trait definition
pub trait TaskQueryBuilder {
    fn new() -> Self;
    fn status(self, status: TaskStatus) -> Self;
    fn project(self, project: String) -> Self;
    fn tag(self, tag: String) -> Self;
    fn due_before(self, date: DateTime<Utc>) -> Self;
    fn due_after(self, date: DateTime<Utc>) -> Self;
    fn sort_by_priority(self) -> Self;
    fn filter_mode(self, mode: crate::query::FilterMode) -> Self;
    fn limit(self, limit: usize) -> Self;
    fn offset(self, offset: usize) -> Self;
    fn build(self) -> Result<TaskQuery, QueryError>;
}

impl TaskQueryBuilder for TaskQueryBuilderImpl {
    fn new() -> Self {
        Self::default()
    }

    fn status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    fn project(mut self, project: String) -> Self {
        self.project_filter = Some(ProjectFilter::Equals(project));
        self
    }

    fn tag(mut self, tag: String) -> Self {
        self.tag_filter = Some(TagFilter::has_tag(tag));
        self
    }

    fn due_before(mut self, date: DateTime<Utc>) -> Self {
        self.date_filter = Some(DateFilter::DueBefore(date));
        self
    }

    fn due_after(mut self, date: DateTime<Utc>) -> Self {
        self.date_filter = Some(DateFilter::DueAfter(date));
        self
    }

    fn sort_by_priority(mut self) -> Self {
        self.sort = Some(SortCriteria::priority());
        self
    }

    fn filter_mode(mut self, mode: crate::query::FilterMode) -> Self {
        self.filter_mode = Some(mode);
        self
    }

    fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    fn build(self) -> Result<TaskQuery, QueryError> {
        // Validate the query
        if self.limit == Some(0) {
            return Err(QueryError::InvalidLimit);
        }
        // default filter_mode is None (up to caller to interpret), keep optional
        Ok(TaskQuery {
            status: self.status,
            project_filter: self.project_filter,
            tag_filter: self.tag_filter,
            date_filter: self.date_filter,
            sort: self.sort,
            limit: self.limit,
            offset: self.offset,
            filter_mode: self.filter_mode,
        })
    }
}

// (No duplicate impl - all TaskQueryBuilder methods are implemented above.)

/// Query builder trait for extensibility
pub trait QueryBuilder {
    type Query;
    type Error;

    /// Validate the built query
    fn validate(&self) -> Result<(), Self::Error>;

    /// Build and validate the query
    fn build_validated(self) -> Result<Self::Query, Self::Error>;
}

impl QueryBuilder for TaskQueryBuilderImpl {
    type Query = TaskQuery;
    type Error = QueryError;

    fn validate(&self) -> Result<(), Self::Error> {
        // Basic validation - can be extended later
        if self.limit == Some(0) {
            return Err(QueryError::InvalidLimit);
        }
        Ok(())
    }

    fn build_validated(self) -> Result<Self::Query, Self::Error> {
        self.validate()?;
        self.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_basic() {
        let builder = TaskQueryBuilderImpl::new();
        let query = builder
            .status(TaskStatus::Pending)
            .project("Work".to_string())
            .build()
            .unwrap();

        assert_eq!(query.status, Some(TaskStatus::Pending));
        assert!(matches!(query.project_filter, Some(ProjectFilter::Equals(ref p)) if p == "Work"));
    }

    #[test]
    fn test_query_builder_validation() {
        let builder = TaskQueryBuilderImpl::new();
        let result = builder.limit(0).build();
        assert!(matches!(result, Err(QueryError::InvalidLimit)));
    }
}

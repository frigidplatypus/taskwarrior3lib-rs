//! Query and filtering system
//!
//! This module provides the query builder and filtering functionality
//! for searching and retrieving tasks.

use crate::task::TaskStatus;
use serde::{Deserialize, Serialize};

pub mod builder;
pub mod filters;

// Re-export commonly used filter types from the filters module
pub use filters::{DateFilter, ProjectFilter, SortCriteria, TagFilter};

/// Task query specification
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TaskQuery {
    pub status: Option<TaskStatus>,
    pub project_filter: Option<ProjectFilter>,
    pub tag_filter: Option<TagFilter>,
    pub date_filter: Option<DateFilter>,
    pub sort: Option<SortCriteria>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    /// How this query interacts with an active Taskwarrior context
    pub filter_mode: Option<crate::query::FilterMode>,
}

// Re-export main types
/// How explicit filters combine with the active Taskwarrior context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterMode {
    /// Combine explicit filters with the active context's read filter (default)
    CombineWithContext,
    /// Ignore any active context and apply explicit filters to the whole dataset
    IgnoreContext,
}

// Keep default behavior implicit elsewhere; builders may add a field for this.

pub use builder::{TaskQueryBuilder, TaskQueryBuilderImpl};

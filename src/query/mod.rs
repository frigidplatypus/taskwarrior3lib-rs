//! Query and filtering system
//!
//! This module provides the query builder and filtering functionality
//! for searching and retrieving tasks.

use crate::task::TaskStatus;
use filter::{DateFilter, TagFilter, ProjectFilter, SortCriteria};

pub mod builder;
pub mod filter;

/// Task query specification
#[derive(Debug, Clone, PartialEq)]
#[derive(Default)]
pub struct TaskQuery {
    pub status: Option<TaskStatus>,
    pub project_filter: Option<ProjectFilter>,
    pub tag_filter: Option<TagFilter>,
    pub date_filter: Option<DateFilter>,
    pub sort: Option<SortCriteria>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}



// Re-export main types
pub use builder::{TaskQueryBuilder, TaskQueryBuilderImpl};

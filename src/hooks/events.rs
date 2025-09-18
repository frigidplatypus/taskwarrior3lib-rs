//! # Hook Events
//!
//! This module defines the hook event system that triggers script execution at specific
//! points during task operations. Each event represents a moment in the task lifecycle
//! where external automation can be performed.
//!
//! ## Event Types
//!
//! Hook events are categorized into three main phases:
//!
//! ### Pre-Operation Events
//! - [`HookEvent::PreAdd`]: Before a task is added (can abort)
//! - [`HookEvent::PreModify`]: Before a task is modified (can abort)  
//! - [`HookEvent::PreDelete`]: Before a task is deleted (can abort)
//!
//! ### On-Operation Events  
//! - [`HookEvent::OnAdd`]: When a task is being added
//! - [`HookEvent::OnModify`]: When a task is being modified
//! - [`HookEvent::OnDelete`]: When a task is being deleted
//! - [`HookEvent::OnComplete`]: When a task is marked complete
//!
//! ### Post-Operation Events
//! - [`HookEvent::PostAdd`]: After a task is successfully added
//! - [`HookEvent::PostModify`]: After a task is successfully modified
//! - [`HookEvent::PostDelete`]: After a task is successfully deleted
//! - [`HookEvent::PostComplete`]: After a task is successfully completed
//!
//! ### Error Events
//! - [`HookEvent::OnAddError`]: When task addition fails
//! - [`HookEvent::OnModifyError`]: When task modification fails
//! - [`HookEvent::OnDeleteError`]: When task deletion fails
//!
//! ## Hook Context
//!
//! The [`HookContext`] provides task data and metadata to hook scripts:
//!
//! ```rust
//! use taskwarriorlib::hooks::{HookContext, HookEvent};
//! use taskwarriorlib::task::Task;
//! use std::collections::HashMap;
//!
//! let task = Task::new("Example task".to_string());
//! let context = HookContext::with_task(HookEvent::OnAdd, task)
//!     .with_data("DEBUG", "1");
//! ```
//!
//! ## Usage
//!
//! Events are typically used internally by the hook system, but can be used
//! directly when implementing custom hook systems:
//!
//! ```rust
//! use taskwarriorlib::hooks::{HookEvent, HookContext};
//!
//! // Check if event allows abortion (pre events can abort)
//! let event = HookEvent::PreAdd;
//! if event.is_pre_event() {
//!     println!("This hook can prevent the operation");
//! }
//!
//! // Get event name for script discovery
//! let script_name = event.to_string();
//! assert_eq!(script_name, "pre-add");
//! ```

use crate::task::Task;
use crate::error::TaskError;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Hook event types that trigger script execution
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    /// Triggered before a task is added
    PreAdd,
    /// Triggered after a task is added
    PostAdd,
    /// Triggered before a task is modified
    PreModify,
    /// Triggered after a task is modified
    PostModify,
    /// Triggered before a task is deleted
    PreDelete,
    /// Triggered after a task is deleted
    PostDelete,
    /// Triggered when a task is completed
    OnComplete,
    /// Triggered when a task is started
    OnStart,
    /// Triggered when a task is stopped
    OnStop,
    /// Custom event type
    Custom(String),
    /// Legacy support
    OnAdd,
    OnModify,
    OnDelete,
    PreOperation(String),
    PostOperation(String),
}

impl HookEvent {
    /// Check if this is a pre-operation event
    pub fn is_pre_event(&self) -> bool {
        matches!(self, 
            HookEvent::PreAdd | 
            HookEvent::PreModify | 
            HookEvent::PreDelete | 
            HookEvent::PreOperation(_)
        )
    }
    
    /// Check if this is a post-operation event
    pub fn is_post_event(&self) -> bool {
        matches!(self, 
            HookEvent::PostAdd | 
            HookEvent::PostModify | 
            HookEvent::PostDelete | 
            HookEvent::PostOperation(_)
        )
    }
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookEvent::PreAdd => write!(f, "pre-add"),
            HookEvent::PostAdd => write!(f, "post-add"),
            HookEvent::PreModify => write!(f, "pre-modify"),
            HookEvent::PostModify => write!(f, "post-modify"),
            HookEvent::PreDelete => write!(f, "pre-delete"),
            HookEvent::PostDelete => write!(f, "post-delete"),
            HookEvent::OnComplete => write!(f, "on-complete"),
            HookEvent::OnStart => write!(f, "on-start"),
            HookEvent::OnStop => write!(f, "on-stop"),
            HookEvent::Custom(name) => write!(f, "{name}"),
            // Legacy support
            HookEvent::OnAdd => write!(f, "on-add"),
            HookEvent::OnModify => write!(f, "on-modify"),
            HookEvent::OnDelete => write!(f, "on-delete"),
            HookEvent::PreOperation(op) => write!(f, "pre-{op}"),
            HookEvent::PostOperation(op) => write!(f, "post-{op}"),
        }
    }
}

/// Hook execution context passed to hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// The event that triggered this hook
    pub event: HookEvent,
    /// The task being operated on (if applicable)
    pub task: Option<Task>,
    /// The previous version of the task (for modify operations)
    pub old_task: Option<Task>,
    /// Additional context data
    pub data: HashMap<String, String>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new(event: HookEvent) -> Self {
        Self {
            event,
            task: None,
            old_task: None,
            data: HashMap::new(),
        }
    }
    
    /// Create context with a task
    pub fn with_task(event: HookEvent, task: Task) -> Self {
        Self {
            event,
            task: Some(task),
            old_task: None,
            data: HashMap::new(),
        }
    }
    
    /// Create context for modify operations
    pub fn with_modify(event: HookEvent, old_task: Task, new_task: Task) -> Self {
        Self {
            event,
            task: Some(new_task),
            old_task: Some(old_task),
            data: HashMap::new(),
        }
    }
    
    /// Add additional context data
    pub fn with_data<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
}

/// Legacy hook event data for compatibility
#[derive(Debug, Clone)]
pub struct HookEventData {
    pub event: HookEvent,
    pub task: Option<Task>,
    pub old_task: Option<Task>,
}

impl From<HookContext> for HookEventData {
    fn from(context: HookContext) -> Self {
        Self {
            event: context.event,
            task: context.task,
            old_task: context.old_task,
        }
    }
}

impl From<HookEventData> for HookContext {
    fn from(data: HookEventData) -> Self {
        Self {
            event: data.event,
            task: data.task,
            old_task: data.old_task,
            data: HashMap::new(),
        }
    }
}

/// Process hook events (placeholder for now)
pub fn process_event(_event_data: &HookEventData) -> Result<(), TaskError> {
    // TODO: Implement actual hook event processing with the execution engine
    Ok(())
}

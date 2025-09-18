# Data Model: Rust Library for Taskwarrior Integration

**Date**: September 18, 2025  
**Phase**: 1 - Design & Contracts

## Core Entities

### Task

The central entity representing a Taskwarrior task.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier (UUID)
    pub id: Uuid,

    /// Task description (required)
    pub description: String,

    /// Current status
    pub status: TaskStatus,

    /// Creation timestamp
    pub entry: DateTime<Utc>,

    /// Last modification timestamp
    pub modified: Option<DateTime<Utc>>,

    /// Due date
    pub due: Option<DateTime<Utc>>,

    /// Scheduled date
    pub scheduled: Option<DateTime<Utc>>,

    /// Wait until date
    pub wait: Option<DateTime<Utc>>,

    /// End date (when completed/deleted)
    pub end: Option<DateTime<Utc>>,

    /// Priority level
    pub priority: Option<Priority>,

    /// Project assignment
    pub project: Option<String>,

    /// Tags assigned to task
    pub tags: HashSet<String>,

    /// Task annotations (notes)
    pub annotations: Vec<Annotation>,

    /// Dependencies (UUIDs of tasks this depends on)
    pub depends: HashSet<Uuid>,

    /// Urgency score (calculated)
    pub urgency: f64,

    /// User-defined attributes
    pub udas: HashMap<String, UdaValue>,

    /// Recurrence configuration
    pub recur: Option<RecurrencePattern>,

    /// Parent task for recurring tasks
    pub parent: Option<Uuid>,

    /// Mask for recurring task templates
    pub mask: Option<String>,

    /// Indication if task is active (started)
    pub active: bool,

    /// Start time for time tracking
    pub start: Option<DateTime<Utc>>,
}
```

### TaskStatus

Enumeration of possible task states.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is pending (not completed)
    Pending,
    /// Task has been completed
    Completed,
    /// Task has been deleted
    Deleted,
    /// Task is waiting (hidden until wait date)
    Waiting,
    /// Task is recurring
    Recurring,
}
```

### Priority

Task priority levels matching Taskwarrior's system.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
}
```

### Annotation

Notes attached to tasks with timestamps.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    /// When annotation was added
    pub entry: DateTime<Utc>,
    /// Annotation text
    pub description: String,
}
```

### TaskQuery

Query builder for filtering and retrieving tasks.

```rust
#[derive(Debug, Default, Clone)]
pub struct TaskQuery {
    /// Status filter
    status: Option<TaskStatus>,
    /// Project filter (supports hierarchy)
    project: Option<ProjectFilter>,
    /// Tag filters (include/exclude)
    tags: TagFilter,
    /// Date range filters
    date_filters: Vec<DateFilter>,
    /// Text search in description/annotations
    search: Option<String>,
    /// Priority filter
    priority: Option<Priority>,
    /// Custom filter expressions
    custom_filters: Vec<String>,
    /// Sort criteria
    sort: Vec<SortCriteria>,
    /// Result limit
    limit: Option<usize>,
}
```

### Context

Named filters for organizing work contexts.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    /// Context name (unique identifier)
    pub name: String,
    /// Filter expression defining the context
    pub filter: String,
    /// Human-readable description
    pub description: Option<String>,
}
```

### Project

Hierarchical project organization.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    /// Full project path (e.g., "Work.Programming.Library")
    pub name: String,
    /// Parent project
    pub parent: Option<String>,
    /// Child projects
    pub children: Vec<String>,
    /// Number of tasks in this project
    pub task_count: usize,
}
```

### Report

Customizable report definitions.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// Report name
    pub name: String,
    /// Columns to display
    pub columns: Vec<String>,
    /// Default filter for report
    pub filter: Option<String>,
    /// Sort criteria
    pub sort: Vec<String>,
    /// Labels for columns
    pub labels: HashMap<String, String>,
}
```

### Configuration

Taskwarrior configuration management.

```rust
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Data directory path
    pub data_dir: PathBuf,
    /// Configuration file path
    pub rc_file: Option<PathBuf>,
    /// All configuration settings
    pub settings: HashMap<String, String>,
    /// User-defined attributes
    pub udas: HashMap<String, UdaDefinition>,
    /// Report definitions
    pub reports: HashMap<String, Report>,
    /// Context definitions
    pub contexts: HashMap<String, Context>,
}
```

### SyncReplica

Synchronization endpoint configuration.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReplica {
    /// Replica identifier
    pub id: String,
    /// Sync server URL or local path
    pub endpoint: String,
    /// Authentication credentials
    pub credentials: Option<SyncCredentials>,
    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
}
```

### DateParser

Comprehensive date parsing and formatting system.

```rust
#[derive(Debug, Clone)]
pub struct DateParser {
    /// Current date format setting (rc.dateformat)
    pub date_format: String,
    /// Time zone for local time calculations
    pub time_zone: chrono_tz::Tz,
    /// Configuration for date synonyms
    pub synonym_config: DateSynonymConfig,
}

#[derive(Debug, Clone)]
pub struct DateSynonymConfig {
    /// Week start day (Monday = 1, Sunday = 7)
    pub week_start: u32,
    /// Current date for relative calculations
    pub reference_date: DateTime<Utc>,
}
```

### DateSynonym

Named date shortcuts supported by Taskwarrior.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DateSynonym {
    // Current time references
    Now,
    Today,
    Sod, // Start of day
    Eod, // End of day
    Yesterday,
    Tomorrow,

    // Day names (supports modifiers like 'next', 'previous')
    Weekday { day: Weekday, modifier: Option<DateModifier> },

    // Month names
    Month { month: Month, modifier: Option<DateModifier> },

    // Period boundaries
    Som, // Start of month
    Eom, // End of month
    Soy, // Start of year
    Eoy, // End of year
    Soq, // Start of quarter
    Eoq, // End of quarter
    Sow, // Start of week
    Eow, // End of week
    Soww, // Start of work week
    Eoww, // End of work week

    // Ordinal days
    OrdinalDay(u32), // 1st, 2nd, etc.

    // Special dates
    Later,
    Someday,

    // Holiday calculations
    Easter,
    GoodFriday,
    EasterMonday,
    Ascension,
    Pentecost,
    Midsommar,
    Midsommarafton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateModifier {
    Next,
    Previous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}
```

## Supporting Types

### UdaValue

Values for user-defined attributes with type safety.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UdaValue {
    String(String),
    Numeric(f64),
    Date(DateTime<Utc>),
    Duration(Duration),
}
```

### RecurrencePattern

Recurrence configuration for repeating tasks.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecurrencePattern {
    /// Recurrence frequency (e.g., "weekly", "monthly")
    pub frequency: String,
    /// Until date for recurrence
    pub until: Option<DateTime<Utc>>,
}
```

### Filters

Various filter types for querying tasks.

```rust
#[derive(Debug, Clone)]
pub enum ProjectFilter {
    Exact(String),
    Hierarchy(String), // Includes sub-projects
    None, // Tasks with no project
}

#[derive(Debug, Clone, Default)]
pub struct TagFilter {
    pub include: HashSet<String>,
    pub exclude: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct DateFilter {
    pub field: DateField,
    pub operator: DateOperator,
    pub value: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub enum DateField {
    Entry,
    Due,
    Scheduled,
    Wait,
    End,
    Modified,
}

#[derive(Debug, Clone, Copy)]
pub enum DateOperator {
    Before,
    After,
    On,
}
```

## Entity Relationships

```
Task (1) ←→ (0..*) Annotation
Task (0..*) ←→ (0..*) Task [dependencies]
Task (0..*) → (0..1) Project
Task (0..*) ←→ (0..*) Tag
Task (0..1) → (0..*) Task [recurring parent-child]
Context (0..1) → (0..*) Task [filtered view]
Report (0..1) → (0..*) Task [formatted view]
Configuration (1) → (0..*) UdaDefinition
Configuration (1) → (0..*) Report
Configuration (1) → (0..*) Context
Configuration (1) → (1) DateParser [date handling]
DateParser (1) → (0..*) DateSynonym [supported synonyms]
SyncReplica (0..*) ←→ (0..*) Task [synchronization]
```

## Data Validation Rules

### Task Validation

- `description` must not be empty
- `id` must be valid UUID v4
- `entry` must be in the past or present
- `due` must be in the future (if set)
- `priority` must be one of defined values
- `tags` must contain valid characters (alphanumeric, underscore, hyphen)
- `project` must follow hierarchy naming rules (dots separate levels)

### Configuration Validation

- `data_dir` must be accessible directory
- UDA definitions must specify valid types
- Report column names must reference valid task fields
- Context filter expressions must be syntactically correct

### Query Validation

- Date filters must have valid date values
- Sort criteria must reference valid task fields
- Limit must be positive integer
- Filter expressions must parse correctly

## State Transitions

### Task Status Transitions

```
Pending → Completed (task done)
Pending → Deleted (task delete)
Pending → Waiting (task wait:date)
Waiting → Pending (wait date passed)
Completed → Pending (task undo)
Deleted → Pending (task undo, if within undo scope)
Recurring → [generates new Pending task]
```

### Time Tracking States

```
Task (not active) → start → Task (active)
Task (active) → stop → Task (not active, logged time)
Task (active) → done → Task (completed, logged time)
```

## Persistence Strategy

- Tasks stored in Taskwarrior's native database format
- Configuration loaded from `.taskrc` and command-line overrides
- Contexts and reports stored in configuration
- UDAs defined in configuration with type metadata
- Sync metadata stored separately from task data
- File locking used for concurrent access protection

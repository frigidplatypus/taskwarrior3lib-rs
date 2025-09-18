# Data Model: Sample Project for Taskwarrior Library

## Overview

The sample project implements a CLI interface to the Taskwarrior library, focusing on core task management operations. The data model revolves around the Task entity provided by the library, with additional entities for CLI command handling and configuration.

## Core Entities

### Task Entity

**Source**: Provided by `taskwarriorlib::task::Task`

**Core Fields**:

- `id: Uuid` - Unique task identifier
- `description: String` - Task description (required)
- `status: TaskStatus` - Current task state (Pending, Completed, etc.)
- `entry: DateTime<Utc>` - Creation timestamp
- `modified: DateTime<Utc>` - Last modification timestamp

**Optional Fields**:

- `project: Option<String>` - Project/category name
- `priority: Option<TaskPriority>` - Priority level (Low, Medium, High)
- `due: Option<DateTime<Utc>` - Due date
- `tags: HashSet<String>` - Associated tags
- `annotations: Vec<Annotation>` - Additional notes with timestamps

**Validation Rules**:

- Description cannot be empty
- Status must be valid enum value
- Due date must be in the future (if provided)
- Project name follows identifier rules
- Tags are non-empty strings

**State Transitions**:

- `Pending` → `Completed` (via done command)
- `Pending` → `Pending` (via edit command)
- `Completed` → `Completed` (no further transitions in sample)

### CLI Command Entities

#### AddCommand

**Purpose**: Handle task creation requests

**Fields**:

- `description: String` - Task description (required)
- `project: Option<String>` - Optional project assignment
- `priority: Option<String>` - Optional priority level
- `due: Option<String>` - Optional due date (parsed)
- `tags: Vec<String>` - Optional tags

**Validation**:

- Description required and non-empty
- Project name valid identifier (if provided)
- Priority valid enum value (if provided)
- Due date valid date format (if provided)
- Tags non-empty strings

#### ListCommand

**Purpose**: Handle task listing requests

**Fields**:

- `status: Option<TaskStatus>` - Filter by status
- `project: Option<String>` - Filter by project
- `limit: Option<usize>` - Maximum results to return

**Validation**:

- Status valid enum value (if provided)
- Project valid identifier (if provided)
- Limit reasonable positive number

#### EditCommand

**Purpose**: Handle task modification requests

**Fields**:

- `id: String` - Task ID to modify (parsed to Uuid)
- `description: Option<String>` - New description
- `project: Option<String>` - New project
- `priority: Option<String>` - New priority
- `due: Option<String>` - New due date
- `tags: Option<Vec<String>>` - New tags

**Validation**:

- ID valid UUID format
- At least one field to update provided
- Same validation as AddCommand for individual fields

#### DoneCommand

**Purpose**: Handle task completion requests

**Fields**:

- `id: String` - Task ID to complete (parsed to Uuid)

**Validation**:

- ID valid UUID format

### Configuration Entity

**Source**: Provided by `taskwarriorlib::config::Configuration`

**Fields**:

- `data_dir: PathBuf` - Taskwarrior data directory
- `config_file: Option<PathBuf>` - Configuration file path
- `verbose: bool` - Enable verbose output

**Validation**:

- Data directory exists and writable
- Config file readable (if provided)

## Relationships

### Task Manager Integration

```
CLI Commands → Task Manager → Storage Backend → File System
```

**Task Manager**: `DefaultTaskManager<FileStorageBackend>`

- Handles business logic for task operations
- Coordinates between commands and storage
- Manages task lifecycle and validation

**Storage Backend**: `FileStorageBackend`

- Persists tasks to file system
- Handles serialization/deserialization
- Manages data integrity

### Command Flow

```
User Input → CLI Parser → Command Entity → Task Manager → Task Entity → Storage
```

## Data Flow Patterns

### Create Task Flow

1. Parse CLI arguments into `AddCommand`
2. Validate command fields
3. Create `Task` entity with validated data
4. Call `task_manager.add_task(task)`
5. Return success with task ID

### Read Tasks Flow

1. Parse CLI arguments into `ListCommand`
2. Build query from command filters
3. Call `task_manager.query_tasks(query)`
4. Format results for display
5. Return formatted task list

### Update Task Flow

1. Parse CLI arguments into `EditCommand`
2. Validate task ID exists
3. Load existing task
4. Apply updates from command
5. Call `task_manager.update_task(task)`
6. Return success confirmation

### Complete Task Flow

1. Parse CLI arguments into `DoneCommand`
2. Validate task ID exists
3. Load existing task
4. Update status to `Completed`
5. Call `task_manager.update_task(task)`
6. Return success confirmation

## Error Handling

### Library Errors

- `TaskError::NotFound` - Task ID doesn't exist
- `TaskError::ValidationError` - Invalid task data
- `TaskError::StorageError` - File system issues
- `TaskError::ConfigurationError` - Config problems

### CLI Errors

- `ParseError` - Invalid command arguments
- `ValidationError` - Business rule violations
- `RuntimeError` - Unexpected failures

### Error Propagation

```
Library Error → Task Manager → CLI Command → User Message
```

## Testing Data Models

### Test Task Factory

**Purpose**: Create consistent test data

**Fields**:

- Base description pattern
- Default project/priority settings
- Timestamp generation strategy

### Test Scenarios

**Happy Path**: Valid task operations
**Edge Cases**: Empty results, single task, many tasks
**Error Cases**: Invalid IDs, missing data, permission issues

## Performance Characteristics

### Memory Usage

- Task entities: ~1KB each
- Command entities: ~500B each
- Manager instance: ~10KB base + tasks

### Operation Latency

- Add task: < 50ms
- List tasks: < 100ms (up to 1000 tasks)
- Edit task: < 75ms
- Complete task: < 75ms

### Scalability Limits

- Reasonable for personal task management
- Supports thousands of tasks
- File-based storage suitable for CLI usage

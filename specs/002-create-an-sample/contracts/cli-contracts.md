# CLI Command Contracts

## Add Task Command Contract

### Request

```bash
taskwarrior-sample add "Task description" [OPTIONS]
```

**Parameters**:

- `description` (required): String, 1-500 characters
- `--project` (optional): String, valid identifier
- `--priority` (optional): Enum [L, M, H]
- `--due` (optional): Date string (YYYY-MM-DD or relative)
- `--tags` (optional): Comma-separated strings

### Response

**Success**:

```
Task added successfully (ID: 550e8400-e29b-41d4-a716-446655440000)
```

**Errors**:

- `Empty description`: "Description cannot be empty"
- `Invalid project`: "Project name contains invalid characters"
- `Invalid date`: "Due date format is invalid"
- `Storage error`: "Failed to save task: {error}"

### Contract Tests

- [ ] Valid task creation succeeds
- [ ] Empty description fails
- [ ] Invalid project name fails
- [ ] Invalid date format fails
- [ ] Storage failure propagates correctly

## List Tasks Command Contract

### Request

```bash
taskwarrior-sample list [OPTIONS]
```

**Parameters**:

- `--status` (optional): Enum [pending, completed, all]
- `--project` (optional): String filter
- `--limit` (optional): Integer 1-1000

### Response

**Success** (table format):

```
ID                                   Description                    Status     Project    Due
550e8400-e29b-41d4-a716-446655440000  Complete project documentation  pending    Work       -
660e8400-e29b-41d4-a716-446655440001  Review code changes           completed   Dev        -
```

**Empty result**:

```
No tasks found
```

**Errors**:

- `Invalid status`: "Status must be: pending, completed, or all"
- `Invalid limit`: "Limit must be between 1 and 1000"
- `Query error`: "Failed to query tasks: {error}"

### Contract Tests

- [ ] Default list shows pending tasks
- [ ] Status filter works correctly
- [ ] Project filter works correctly
- [ ] Limit parameter respected
- [ ] Empty results handled gracefully

## Edit Task Command Contract

### Request

```bash
taskwarrior-sample edit <ID> [OPTIONS]
```

**Parameters**:

- `id` (required): UUID string
- `--description` (optional): String, 1-500 characters
- `--project` (optional): String, valid identifier
- `--priority` (optional): Enum [L, M, H]
- `--due` (optional): Date string
- `--tags` (optional): Comma-separated strings

### Response

**Success**:

```
Task updated successfully
```

**Errors**:

- `Invalid UUID`: "Task ID format is invalid"
- `Task not found`: "Task not found: {id}"
- `No changes`: "No changes specified"
- `Validation error`: "Invalid {field}: {reason}"
- `Storage error`: "Failed to update task: {error}"

### Contract Tests

- [ ] Valid task update succeeds
- [ ] Invalid UUID fails
- [ ] Non-existent task fails
- [ ] No changes specified fails
- [ ] Partial updates work correctly

## Complete Task Command Contract

### Request

```bash
taskwarrior-sample done <ID>
```

**Parameters**:

- `id` (required): UUID string

### Response

**Success**:

```
Task marked as completed
```

**Errors**:

- `Invalid UUID`: "Task ID format is invalid"
- `Task not found`: "Task not found: {id}"
- `Already completed`: "Task is already completed"
- `Storage error`: "Failed to complete task: {error}"

### Contract Tests

- [ ] Valid task completion succeeds
- [ ] Invalid UUID fails
- [ ] Non-existent task fails
- [ ] Already completed task handled appropriately

## Global Options Contract

### Request

All commands support:

```bash
taskwarrior-sample [COMMAND] --help
taskwarrior-sample [COMMAND] --verbose
taskwarrior-sample [COMMAND] --config <PATH>
```

**Parameters**:

- `--help`: Show command help
- `--verbose`: Enable verbose output
- `--config` (optional): Path to config file

### Response

**Help output**: Standard clap help format
**Verbose output**: Additional debug information
**Config**: Custom configuration file path

## Error Response Format Contract

### Standard Error Format

```
Error: {human_readable_message}
```

**Examples**:

```
Error: Description cannot be empty
Error: Task not found: 550e8400-e29b-41d4-a716-446655440000
Error: Failed to save task: Permission denied
```

### Verbose Error Format

```
Error: {human_readable_message}
Details: {technical_details}
```

**Examples**:

```
Error: Failed to save task: Permission denied
Details: I/O error: Permission denied (os error 13)
```

## Library Integration Contract

### Task Manager Contract

**Interface**: `DefaultTaskManager<FileStorageBackend>`

**Methods Used**:

- `add_task(task: Task) -> Result<Task, TaskError>`
- `get_task(id: Uuid) -> Result<Option<Task>, TaskError>`
- `update_task(task: Task) -> Result<(), TaskError>`
- `delete_task(id: Uuid) -> Result<(), TaskError>`
- `query_tasks(query: TaskQuery) -> Result<Vec<Task>, TaskError>`

### Configuration Contract

**Interface**: `ConfigurationBuilder`

**Usage Pattern**:

```rust
let config = ConfigurationBuilder::new()
    .data_dir(custom_path)
    .build()?;
```

### Error Handling Contract

**Error Types**: `TaskError` enum from library

**Handled Errors**:

- `NotFound`: Task doesn't exist
- `ValidationError`: Invalid task data
- `StorageError`: File system issues
- `ConfigurationError`: Config problems

**Propagation**: All library errors converted to CLI-appropriate messages

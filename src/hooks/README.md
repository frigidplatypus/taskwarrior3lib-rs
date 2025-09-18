# Hook System API Documentation

The Taskwarrior library provides a comprehensive hook system that allows users to extend task management functionality with custom scripts and workflows. Hooks are triggered at specific points during task operations, enabling automation, validation, notifications, and data processing.

## Overview

The hook system consists of several key components:

- **Hook Events**: Defined points in task lifecycle (e.g., pre-add, on-modify, post-complete)
- **Hook Execution**: Process management for running hook scripts with timeout and error handling
- **Hook Configuration**: Discovery and management of hook scripts with priority-based execution
- **Hook System Integration**: Seamless integration with TaskManager operations

## Quick Start

```rust
use taskwarriorlib::{
    hooks::{DefaultHookSystem, HookSystem},
    task::manager::{DefaultTaskManager, TaskManager},
    config::Configuration,
    storage::FileStorageBackend
};

// Create hook system and load hooks from directory
let mut hooks = DefaultHookSystem::new();
hooks.load_hooks_from_dir("~/.taskwarrior/hooks")?;

// Create task manager with hook integration
let config = Configuration::default();
let storage = Box::new(FileStorageBackend::new());
let hook_system = Box::new(hooks);
let mut task_manager = DefaultTaskManager::new(config, storage, hook_system)?;

// Task operations will automatically trigger appropriate hooks
let task = task_manager.add_task("Buy groceries".to_string())?;
let completed = task_manager.complete_task(task.id)?;
```

## Hook Events

The library supports the following hook events:

### Pre-Operation Hooks

- `pre-add`: Before adding a new task
- `pre-modify`: Before modifying an existing task
- `pre-delete`: Before deleting a task

### On-Operation Hooks

- `on-add`: When a task is added (after validation)
- `on-modify`: When a task is modified (after validation)
- `on-delete`: When a task is deleted
- `on-complete`: When a task is marked complete

### Post-Operation Hooks

- `post-add`: After a task is successfully added
- `post-modify`: After a task is successfully modified
- `post-delete`: After a task is successfully deleted
- `post-complete`: After a task is successfully completed

### Error Hooks

- `on-add-error`: When task addition fails
- `on-modify-error`: When task modification fails
- `on-delete-error`: When task deletion fails

## Hook Script Structure

Hook scripts should be executable files placed in your hooks directory (typically `~/.taskwarrior/hooks/`):

```bash
#!/bin/bash
# Example: ~/.taskwarrior/hooks/on-add.sh

# Read task data from stdin
read -r task_json

# Process the task (example: log to file)
echo "Task added: $task_json" >> ~/.taskwarrior/task.log

# Exit codes:
# 0 = success, continue processing
# 1 = error, abort operation (for pre-* hooks)
# 2+ = error, continue with warning
exit 0
```

## Hook Configuration

### Directory-Based Configuration

The simplest way to configure hooks is by placing executable scripts in a hooks directory:

```rust
use taskwarriorlib::hooks::DefaultHookSystem;

let mut hooks = DefaultHookSystem::new();
hooks.load_hooks_from_dir("/path/to/hooks")?;

// Automatically discovers scripts matching hook event names:
// - pre-add.sh -> pre-add event
// - on-modify.py -> on-modify event
// - post-complete -> post-complete event
```

### TOML Configuration Files

For more advanced configuration, create `.hookrc` files alongside scripts:

```toml
# ~/.taskwarrior/hooks/notification.hookrc
name = "Desktop Notifications"
description = "Shows desktop notifications for task events"
events = ["on-add", "on-complete", "on-modify"]
priority = 100
timeout = 5
enabled = true

[environment]
DISPLAY = ":0"
USER = "username"
```

### Programmatic Configuration

```rust
use taskwarriorlib::hooks::{DefaultHookSystem, HookConfig, HookEvent};

let mut hooks = DefaultHookSystem::new();

// Create hook configuration
let config = HookConfig {
    name: "Custom Hook".to_string(),
    description: Some("Custom task processing".to_string()),
    path: "/path/to/script.sh".into(),
    events: vec![HookEvent::OnAdd, HookEvent::OnModify],
    priority: 50,
    timeout: Some(10),
    enabled: true,
    environment: [("DEBUG".to_string(), "1".to_string())].into(),
};

hooks.add_hook(config)?;
```

## Hook Context

Hooks receive task data through stdin as JSON and environment variables:

### JSON Input Format

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "description": "Buy groceries",
  "status": "pending",
  "entry": "2024-01-15T10:30:00Z",
  "modified": "2024-01-15T10:30:00Z",
  "project": "personal",
  "tags": ["shopping", "urgent"],
  "priority": "high",
  "due": "2024-01-16T18:00:00Z"
}
```

### Environment Variables

- `TASKWARRIOR_EVENT`: The hook event name (e.g., "on-add")
- `TASKWARRIOR_TASK_ID`: The UUID of the task
- `TASKWARRIOR_VERSION`: Library version
- `TASKWARRIOR_CONFIG_DIR`: Configuration directory path

## Hook System API

### DefaultHookSystem

The main hook system implementation:

```rust
use taskwarriorlib::hooks::DefaultHookSystem;

// Create new hook system
let mut hooks = DefaultHookSystem::new();

// Load hooks from directory
hooks.load_hooks_from_dir("~/.taskwarrior/hooks")?;

// Load hooks from TOML configuration file
hooks.load_hooks_from_config("hooks.toml")?;

// Create with hooks from directory (builder pattern)
let hooks = DefaultHookSystem::with_hooks_from_dir("~/.taskwarrior/hooks")?;

// Check number of loaded hooks
let count = hooks.hook_count();
```

### HookSystem Trait

Implement custom hook systems:

```rust
use taskwarriorlib::hooks::{HookSystem, HookEvent, HookContext, HookResult};
use async_trait::async_trait;

struct CustomHookSystem {
    // implementation details
}

#[async_trait]
impl HookSystem for CustomHookSystem {
    async fn execute_hooks(
        &mut self,
        event: HookEvent,
        context: &HookContext
    ) -> Result<Vec<HookResult>, Box<dyn std::error::Error + Send + Sync>> {
        // Custom hook execution logic
        Ok(vec![])
    }
}
```

### Hook Execution Results

Hooks return execution results indicating success or failure:

```rust
use taskwarriorlib::hooks::HookResult;

// Successful execution
let success = HookResult {
    hook_name: "notification".to_string(),
    event: HookEvent::OnAdd,
    success: true,
    exit_code: Some(0),
    output: Some("Task added successfully".to_string()),
    error: None,
    duration: std::time::Duration::from_millis(150),
};

// Failed execution
let failure = HookResult {
    hook_name: "validator".to_string(),
    event: HookEvent::PreAdd,
    success: false,
    exit_code: Some(1),
    output: None,
    error: Some("Validation failed: missing project".to_string()),
    duration: std::time::Duration::from_millis(50),
};
```

## Integration with TaskManager

The hook system integrates automatically with TaskManager operations:

```rust
use taskwarriorlib::{
    hooks::DefaultHookSystem,
    task::manager::{DefaultTaskManager, TaskManager},
    task::TaskUpdate,
    config::Configuration,
    storage::FileStorageBackend
};

// Set up task manager with hooks
let hooks = Box::new(DefaultHookSystem::with_hooks_from_dir("~/.taskwarrior/hooks")?);
let config = Configuration::default();
let storage = Box::new(FileStorageBackend::new());
let mut manager = DefaultTaskManager::new(config, storage, hooks)?;

// All operations trigger appropriate hooks automatically:

// Triggers: pre-add -> on-add -> post-add
let task = manager.add_task("New task".to_string())?;

// Triggers: pre-modify -> on-modify -> post-modify
let updates = TaskUpdate::new().description("Updated task".to_string());
let updated = manager.update_task(task.id, updates)?;

// Triggers: on-complete -> post-complete
let completed = manager.complete_task(task.id)?;

// Triggers: pre-delete -> on-delete -> post-delete
let deleted = manager.delete_task(task.id)?;
```

## Error Handling

The hook system provides comprehensive error handling:

### Hook Execution Timeouts

```rust
use taskwarriorlib::hooks::{HookConfig, HookEvent};

let config = HookConfig {
    name: "slow-hook".to_string(),
    path: "/path/to/slow-script.sh".into(),
    events: vec![HookEvent::OnAdd],
    timeout: Some(30), // 30-second timeout
    // ... other fields
};
```

### Error Recovery

```rust
// Pre-operation hooks can abort operations by returning non-zero exit codes
// Other hooks log errors but don't stop task operations

// Example error handling in hook scripts:
```

```bash
#!/bin/bash
# pre-add-validator.sh

# Read and validate task
read -r task_json

# Check if task has required project field
if ! echo "$task_json" | jq -e '.project' > /dev/null; then
    echo "Error: Task must have a project assigned" >&2
    exit 1  # Abort task addition
fi

echo "Task validation passed"
exit 0
```

## Best Practices

### Hook Development

1. **Keep hooks fast**: Hooks should complete quickly to avoid blocking operations
2. **Use appropriate timeouts**: Set reasonable timeouts for hook execution
3. **Handle errors gracefully**: Use proper exit codes and error messages
4. **Test thoroughly**: Test hooks with various task scenarios
5. **Log appropriately**: Use structured logging for debugging

### Security Considerations

1. **Validate input**: Always validate JSON input from stdin
2. **Sanitize environment**: Be careful with environment variable usage
3. **Use absolute paths**: Avoid relative paths in hook scripts
4. **Set proper permissions**: Hook scripts should be executable only by owner
5. **Avoid shell injection**: Use proper quoting and validation

### Performance Tips

1. **Minimize hook count**: Only use necessary hooks
2. **Optimize script execution**: Use efficient scripting languages and techniques
3. **Cache when possible**: Cache expensive operations between hook runs
4. **Use priority ordering**: Set priorities to control execution order
5. **Monitor hook performance**: Track execution times and optimize slow hooks

## Examples

### Notification Hook

```bash
#!/bin/bash
# ~/.taskwarrior/hooks/on-add-notification.sh

# Read task data
read -r task_json

# Extract task description
description=$(echo "$task_json" | jq -r '.description')

# Send desktop notification
notify-send "Task Added" "$description"

exit 0
```

### Task Validation Hook

```bash
#!/bin/bash
# ~/.taskwarrior/hooks/pre-add-validator.sh

read -r task_json

# Validate required fields
project=$(echo "$task_json" | jq -r '.project // empty')
if [ -z "$project" ]; then
    echo "Error: Tasks must have a project assigned" >&2
    exit 1
fi

# Check description length
description=$(echo "$task_json" | jq -r '.description')
if [ ${#description} -lt 5 ]; then
    echo "Error: Task description too short (minimum 5 characters)" >&2
    exit 1
fi

exit 0
```

### Time Tracking Hook

```python
#!/usr/bin/env python3
# ~/.taskwarrior/hooks/on-complete-timetracker.py

import json
import sys
import datetime
from pathlib import Path

# Read task data
task_data = json.loads(sys.stdin.read())

# Calculate time spent (example: entry to completion)
entry = datetime.datetime.fromisoformat(task_data['entry'].replace('Z', '+00:00'))
completed = datetime.datetime.now(datetime.timezone.utc)
duration = completed - entry

# Log to time tracking file
log_file = Path.home() / '.taskwarrior' / 'time_log.csv'
with open(log_file, 'a') as f:
    f.write(f"{task_data['id']},{task_data['description']},{duration.total_seconds()}\n")

print(f"Logged {duration.total_seconds()} seconds for task: {task_data['description']}")
sys.exit(0)
```

### Integration with External Tools

```bash
#!/bin/bash
# ~/.taskwarrior/hooks/on-add-github.sh

# Sync new tasks to GitHub Issues
read -r task_json

description=$(echo "$task_json" | jq -r '.description')
project=$(echo "$task_json" | jq -r '.project // "default"')
tags=$(echo "$task_json" | jq -r '.tags[]?' | tr '\n' ',' | sed 's/,$//')

# Create GitHub issue using gh CLI
gh issue create \
    --title "$description" \
    --label "taskwarrior,$tags" \
    --body "Created from Taskwarrior task" \
    --repo "user/project-$project"

exit 0
```

This completes the hook system API documentation. The hook system provides a powerful and flexible way to extend Taskwarrior functionality while maintaining compatibility with existing workflows.

# Quickstart: Using Context-aware APIs

This quickstart shows how to set up contexts, query with and without the active context, and create tasks honoring the context write filter using the public API.

## Setup a configuration with contexts

```rust
use taskwarrior3lib::config::ConfigurationBuilder;

let config = ConfigurationBuilder::new()
   // Put test data under a custom dir; omit for default XDG lookup
   .data_dir("/tmp/taskdata")
   // Define contexts
   .set("context.home", "project:Home")
   .set("context.work", "project:Work")
   // Select the active context
   .set("context", "work")
   .build()?;
```

## Build a TaskManager

```rust
use taskwarrior3lib::hooks::DefaultHookSystem;
use taskwarrior3lib::task::manager::TaskManagerBuilder;
use taskwarrior3lib::storage::FileStorageBackend; // or TaskChampionStorageBackend

let storage = FileStorageBackend::with_path("/tmp/taskdata");
let hooks = DefaultHookSystem::new();
let mut task_manager = TaskManagerBuilder::new()
   .config(config)
   .storage(Box::new(storage))
   .hooks(Box::new(hooks))
   .build()?;
```

## Query within active context (default)

```rust
use taskwarrior3lib::query::TaskQueryBuilderImpl;
use taskwarrior3lib::TaskStatus;

// With active context = work (project:Work), this returns only Work tasks
let q = TaskQueryBuilderImpl::new()
   .status(TaskStatus::Pending)
   .build()?;
let tasks = task_manager.query_tasks(&q)?;
```

## Ignore the active context

```rust
use taskwarrior3lib::query::{TaskQueryBuilderImpl, FilterMode};
use taskwarrior3lib::TaskStatus;

let q = TaskQueryBuilderImpl::new()
   .status(TaskStatus::Pending)
   .filter_mode(FilterMode::IgnoreContext)
   .build()?;
let tasks = task_manager.query_tasks(&q)?; // returns tasks regardless of active context
```

## Add a task honoring context write_filter

If the active context defines a `write_filter` such as `context.work.write = project:WorkInbox`,
adding a task will assign the project by default. To bypass, use `IgnoreContext`.

```rust
use taskwarrior3lib::task::manager::{AddOptions, TaskManager};
use taskwarrior3lib::query::FilterMode;

// Default behavior (applies write_filter if present)
let created = task_manager.add_task("Write report".to_string())?;

// Bypass write_filter
let opts = AddOptions { filter_mode: Some(FilterMode::IgnoreContext) };
let created2 = task_manager.add_task_with_options("Scratch note".to_string(), opts)?;
```

## Manage contexts

```rust
use taskwarrior3lib::config::context;

// List and show
let all = context::list(task_manager.config())?;
let active = context::show(task_manager.config())?;

// Set and clear (persist to taskrc)
let mut cfg_owned = taskwarrior3lib::config::Configuration::from_xdg()?;
context::set(&mut cfg_owned, "work")?;
context::clear(&mut cfg_owned)?;
```

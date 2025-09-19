# Taskwarrior Sample Project

This sample CLI demonstrates usage of the Taskwarrior Rust library (via TaskChampion) for core task management operations. It uses a writable, file-based backend for persistent task storage.

## Features
- **Add tasks** with descriptions, projects, priorities, and due dates
- **List tasks** (pending or completed)
- **Edit tasks** (description, project, priority, due)
- **Mark tasks as done**
- **Import tasks** from system Taskwarrior (if installed)
- **Debug**: Show backend and data directory info

## Storage Backend
- Uses TaskChampion's file-based backend (`.taskdata/` directory)
- All tasks are stored in `.taskdata/tasks.json` (created automatically)
- Backups are kept in `.taskdata/backups/`

## Usage

Build and run the CLI (from the sample directory):

```sh
cargo run -- add "My first task"
cargo run -- list
cargo run -- done <TASK_ID>
cargo run -- edit <TASK_ID> description="Updated description"
cargo run -- import   # Imports up to 10 pending tasks from system Taskwarrior
cargo run -- debug    # Shows backend and data directory info
```

### Command Reference
- `add <description> [--project <name>] [--priority <P>] [--due <YYYY-MM-DD>]`: Add a new task
- `list`: List all pending tasks
- `done <id>`: Mark a task as completed
- `edit <id> [field=value ...]`: Edit a task (description, project, priority, due)
- `import`: Import up to 10 pending tasks from system Taskwarrior (requires `task` CLI)
- `debug`: Show backend and data directory info

### Example Output
```
$ cargo run -- add "Buy groceries" --project "Home" --priority H
Task added successfully
ID: 123e4567-e89b-12d3-a456-426614174000

$ cargo run -- list
ID                                   | Description                   | Status     | Project        | Due         
------------------------------------------------------------------------------------------------------------------------
123e4567-e89b-12d3-a456-426614174000 | Buy groceries                 | pending    | Home           | -           
```

## Environment Setup
- No special setup required; `.taskdata/` is created automatically.
- Requires Rust and Cargo.
- For `import`, the system `task` CLI must be installed.

## Testing
Run all contract, integration, and unit tests:
```sh
cargo test
```
All tests should pass, demonstrating full CRUD functionality and error handling.

## Troubleshooting
- If you see errors about missing/corrupted data, delete `.taskdata/` and re-run.
- For import errors, ensure the `task` CLI is installed and working.

## Notes
- This project is for demonstration and validation of the library.
- See source code for more details and examples.

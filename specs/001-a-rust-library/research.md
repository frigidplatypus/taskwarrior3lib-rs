# Research: Rust Library for Taskwarrior Integration

**Date**: September 18, 2025  
**Phase**: 0 - Outline & Research

## Research Topics

### 1. Taskwarrior Database Format and File Structure

**Decision**: Use direct file system access to Taskwarrior's data directory  
**Rationale**:

- Taskwarrior 3.x stores tasks in a local database format (not the old plain text format)
- Direct access provides better performance than shell command invocation
- Allows for real-time updates and monitoring of task changes
- Maintains compatibility with Taskwarrior's native operations

**Alternatives considered**:

- Shell command invocation via `task` CLI - rejected due to performance overhead
- JSON export/import only - rejected as it doesn't support real-time operations
- TaskChampion library integration - considered but adds complexity for this use case

**Implementation approach**:

- Access `~/.local/share/taskwarrior/` or XDG_DATA_HOME equivalent
- Read/write to the task database files directly
- Implement file locking to prevent corruption during concurrent access

### 2. XDG Base Directory Specification Implementation

**Decision**: Use the `dirs` crate with fallback logic  
**Rationale**:

- `dirs` crate provides cross-platform XDG compliance
- Handles platform differences (Linux XDG, macOS Application Support, Windows AppData)
- Well-maintained and widely used in Rust ecosystem

**Alternatives considered**:

- `xdg` crate - Linux-only, doesn't handle cross-platform needs
- Manual implementation - error-prone and maintenance burden
- `directories` crate - less feature-complete than `dirs`

**Implementation approach**:

```rust
// Priority order:
// 1. Explicit user configuration
// 2. TASKDATA environment variable
// 3. XDG_DATA_HOME/taskwarrior
// 4. ~/.local/share/taskwarrior (Linux)
// 5. ~/Library/Application Support/Taskwarrior (macOS)
// 6. %APPDATA%/Taskwarrior (Windows)
```

### 3. Rust Library Configuration Management Best Practices

**Decision**: Builder pattern with environment variable and file override support  
**Rationale**:

- Builder pattern is idiomatic Rust for complex configuration
- Environment variables provide deployment flexibility
- Configuration file support maintains Taskwarrior compatibility
- Provides compile-time safety with runtime flexibility

**Alternatives considered**:

- Simple struct with defaults - insufficient flexibility
- Configuration traits - overly complex for this use case
- External configuration crates (config-rs) - adds dependency weight

**Implementation approach**:

```rust
TaskManager::builder()
    .data_dir(custom_path)
    .config_file(custom_taskrc)
    .auto_sync(true)
    .build()?
```

### 4. Taskwarrior Hook API Integration Patterns

**Decision**: Event-driven callback system with trait-based hooks  
**Rationale**:

- Hooks are a key Taskwarrior feature for automation
- Trait-based approach provides type safety and flexibility
- Event system allows multiple hooks per event
- Maintains compatibility with existing Taskwarrior hook ecosystem

**Alternatives considered**:

- Shell script execution only - limited functionality
- No hook support - missing key feature
- Function pointer callbacks - less flexible than traits

**Implementation approach**:

```rust
trait TaskHook {
    fn on_add(&mut self, task: &Task) -> Result<(), HookError>;
    fn on_modify(&mut self, old: &Task, new: &Task) -> Result<(), HookError>;
    fn on_delete(&mut self, task: &Task) -> Result<(), HookError>;
}
```

### 5. Taskwarrior Synchronization Protocols

**Decision**: Support TaskChampion sync protocol with adapter pattern  
**Rationale**:

- TaskChampion is the official sync implementation for Taskwarrior 3.x
- Adapter pattern allows integration without tight coupling
- Maintains compatibility with existing sync servers
- Provides foundation for future sync protocol support

**Alternatives considered**:

- No sync support - missing critical feature
- Custom sync implementation - reinventing the wheel
- Legacy sync protocols only - doesn't support Taskwarrior 3.x

**Implementation approach**:

- Optional dependency on TaskChampion
- Sync trait for pluggable sync backends
- Support for both cloud storage and dedicated sync servers

### 6. Error Handling and Type Design

**Decision**: Comprehensive error types with `thiserror` for ergonomics  
**Rationale**:

- `thiserror` provides excellent error ergonomics in Rust
- Structured error types enable better error handling by library users
- Context preservation helps with debugging
- Follows Rust best practices for library error design

**Alternatives considered**:

- Simple string errors - poor user experience
- `anyhow` crate - better for applications than libraries
- Custom error implementation - unnecessary complexity

**Implementation approach**:

```rust
#[derive(thiserror::Error, Debug)]
pub enum TaskError {
    #[error("Task not found: {id}")]
    NotFound { id: String },
    #[error("Invalid task data: {reason}")]
    InvalidData { reason: String },
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}
```

## Technical Decisions Summary

| Component               | Technology Choice     | Key Reason                            |
| ----------------------- | --------------------- | ------------------------------------- |
| Configuration Discovery | `dirs` crate          | Cross-platform XDG compliance         |
| Error Handling          | `thiserror`           | Library-focused ergonomics            |
| Database Access         | Direct file system    | Performance and real-time capability  |
| Configuration API       | Builder pattern       | Rust idiomaticity and flexibility     |
| Hook System             | Trait-based callbacks | Type safety and extensibility         |
| Sync Integration        | TaskChampion adapter  | Official protocol compatibility       |
| Async Support           | `tokio` (optional)    | Future-proofing and I/O efficiency    |
| Serialization           | `serde` ecosystem     | JSON compatibility requirement        |
| Date/Time               | `chrono`              | Taskwarrior date format compatibility |

## Dependencies Analysis

**Core Dependencies**:

- `serde` + `serde_json` - JSON serialization (required by spec)
- `chrono` - Date/time handling compatible with Taskwarrior
- `uuid` - Task ID generation and management
- `thiserror` - Error type definitions
- `dirs` - XDG directory discovery

**Optional Dependencies**:

- `tokio` - Async I/O support (feature-gated)
- `taskchampion` - Sync protocol support (feature-gated)

**Development Dependencies**:

- `tempfile` - Test environment setup
- `assert_matches` - Test ergonomics

## Risk Assessment

**Low Risk**:

- XDG directory discovery - well-established patterns
- JSON serialization - mature ecosystem
- Error handling - standard Rust practices

**Medium Risk**:

- Direct database access - requires careful file format understanding
- Hook integration - needs thorough testing with real hooks

**High Risk**:

- Sync protocol integration - complex distributed systems concerns
- Concurrent access - file locking and consistency challenges

## Next Steps for Phase 1

1. Design detailed data models based on Taskwarrior's task schema
2. Define API contracts for all major operations
3. Create contract tests that fail initially
4. Design configuration and initialization patterns
5. Plan integration test scenarios

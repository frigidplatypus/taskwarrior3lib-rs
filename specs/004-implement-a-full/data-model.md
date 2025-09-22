# Phase 1 — Data Model

Entities:

- ReplicaWrapper
  - path: PathBuf
  - access_mode: enum { ReadOnly, ReadWrite }
  - retry_policy: Option<RetryPolicy>

- OperationBatch
  - operations: Vec<taskchampion::Operation>
  - metadata: Option<HashMap<String,String>>

- Task (library model)
 - Task (library model)
  - uuid: Uuid         # stable canonical identifier stored in TaskChampion
  - display_id: Option<u32>    # transient display id (working_set index) used by CLI for quick edits; may be None when not known
  - description: String
  - status: enum { Pending, Completed, Deleted }
  - entry: DateTime<Utc>
  - due: Option<DateTime<Utc>>
  - annotations: Vec<Annotation>
  - tags: HashSet<String>
  - uda: HashMap<String, String>

Validation rules:
- `uuid` must be a valid RFC 4122 UUID (use the `uuid` crate's parsing/validation). The library's canonical identifier field is `uuid` and MUST be used for all storage and operation construction.
- The numeric CLI `id` (working_set index) is transient and MUST NOT be used as the primary key; it may be present as `id: Option<u32>` only for CLI/display contexts.
- Dates must be ISO8601 and stored as `DateTime<Utc>`.

Notes about identifiers:
- Use the `uuid` (RFC 4122) for persisted references, links between tasks, and for constructing TaskChampion `Operation`s (Create/Update/Delete). The numeric `id` that appears in CLI lists is a volatile working_set index and can change when the working set is rebuilt or when filters change. The library will preserve a transient `display_id: Option<u32>` where helpful for CLI helpers, but all durable references and commits must use the `uuid` field.

Notes:
- `uuid` is the canonical, immutable identifier for a task and must be used when creating operations (Create/Delete/Update) against the Replica. The `id` field is a convenience transient integer derived from the TaskChampion `working_set` mapping for CLI-style interactions (e.g., `task 12 done`) and must never be used as the primary key for storage operations.
- `id` may be None for tasks that were loaded outside of a working_set context (e.g., when reading raw replica data in a library consumer). When present, `id` values are stable only until the working_set is rebuilt.
- Use `uuid` for all persisted references and operation construction to avoid ambiguity across replicas and sync boundaries.

Notes:
- For write operations, model will translate Task changes into OperationBatch using taskchampion Operation builders where possible.

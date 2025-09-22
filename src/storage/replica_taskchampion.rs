use crate::error::{StorageError, TaskError};
use crate::storage::operation_batch::Operation as Op;
use crate::storage::replica_wrapper::ReplicaWrapper;
use std::path::Path;
use uuid::Uuid;
#[cfg(feature = "taskchampion")]
use std::sync::{Arc, Mutex};

// Commands sent to the replica actor thread
#[cfg(feature = "taskchampion")]
enum ReplicaCommand {
    Commit { ops: Vec<Op>, resp: std::sync::mpsc::Sender<Result<(), TaskError>> },
    Open { path: std::path::PathBuf, resp: std::sync::mpsc::Sender<Result<(), TaskError>> },
    ReadTask { id: Uuid, resp: std::sync::mpsc::Sender<Result<Option<crate::task::Task>, TaskError>> },
}

// Legacy helper removed: prefer the replica-aware mapping helper
// The preferred mapping function is `map_ops_to_tc_operations_with_replica` which
// can use Task helper methods by operating on a live `taskchampion::Replica`.

// Variant of the mapper that can prefer Task helper methods by using a live
// `taskchampion::Replica`. This produces more precise `Operation` variants for
// per-item changes (tags, dependencies, annotations) by creating or obtaining
// a `Task` and calling its helpers. When unable to use the Task API the
// function falls back to TaskData updates, behaving like
// `map_ops_to_tc_operations`.
#[cfg(feature = "taskchampion")]
pub fn map_ops_to_tc_operations_with_replica(replica: &mut taskchampion::Replica, ops: &[Op]) -> Result<taskchampion::Operations, TaskError> {
    use taskchampion::{Operations, TaskData, Annotation as TcAnnotation, Tag as TcTag};
    let mut tc_ops = Operations::new();

    for op in ops {
        match op {
            Op::Create { uuid, data } => {
                let mut td = TaskData::create(*uuid, &mut tc_ops);
                if let serde_json::Value::Object(map) = data {
                    for (k, v) in map {
                        let sval = match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Array(arr) => {
                                let parts: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                                parts.join(" ")
                            }
                            _ => serde_json::to_string(v).unwrap_or_default(),
                        };
                        td.update(k, Some(sval), &mut tc_ops);
                    }
                }
            }
            Op::UndoPoint => tc_ops.push(taskchampion::Operation::UndoPoint),
            Op::SetField { uuid, key, value } => {
                let mut td = TaskData::create(*uuid, &mut tc_ops);
                td.update(key, Some(value.clone()), &mut tc_ops);
            }
            Op::UnsetField { uuid, key } => {
                let mut td = TaskData::create(*uuid, &mut tc_ops);
                td.update(key, None, &mut tc_ops);
            }
            Op::AddTag { uuid, tag } => {
                if let Ok(mut t) = replica.create_task(*uuid, &mut tc_ops) {
                    if let Ok(tc_tag) = tag.parse::<TcTag>() {
                        let _ = t.add_tag(&tc_tag, &mut tc_ops);
                    }
                }
            }
            Op::RemoveTag { uuid, tag } => {
                if let Ok(mut t) = replica.create_task(*uuid, &mut tc_ops) {
                    if let Ok(tc_tag) = tag.parse::<TcTag>() {
                        let _ = t.remove_tag(&tc_tag, &mut tc_ops);
                    }
                }
            }
            Op::AddAnnotation { uuid, entry, description } => {
                if let Ok(mut t) = replica.create_task(*uuid, &mut tc_ops) {
                    let ann = TcAnnotation { entry: *entry, description: description.clone() };
                    let _ = t.add_annotation(ann, &mut tc_ops);
                }
            }
            Op::AddDependency { uuid, depends_on } => {
                if let Ok(mut t) = replica.create_task(*uuid, &mut tc_ops) {
                    let _ = t.add_dependency(*depends_on, &mut tc_ops);
                }
            }
            Op::RemoveDependency { uuid, depends_on } => {
                if let Ok(mut t) = replica.create_task(*uuid, &mut tc_ops) {
                    let _ = t.remove_dependency(*depends_on, &mut tc_ops);
                }
            }
            Op::Update { uuid, key, old: _old, new } => {
                let sval = match new {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(arr) => {
                        let parts: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                        parts.join(" ")
                    }
                    other => serde_json::to_string(other).unwrap_or_default(),
                };
                let mut td = TaskData::create(*uuid, &mut tc_ops);
                td.update(key, Some(sval), &mut tc_ops);
            }
            Op::Delete { uuid } => {
                let mut td = TaskData::create(*uuid, &mut tc_ops);
                td.update("status", Some("deleted".to_string()), &mut tc_ops);
            }
        }
    }

    Ok(tc_ops)
}

// Note: The real TaskChampion-backed Replica implementation is feature-gated
// and intentionally omitted here to avoid pulling complex, non-Send/Sync
// runtime types into the library build during tests. The current stub
// provides compile-time safety and allows tests to run. A full wrapper that
// uses the `taskchampion` crate can be implemented behind the feature flag
// later.

/// Factory to open a TaskChampion-backed replica wrapper.
pub fn open_taskchampion_replica(path: &Path) -> Result<Box<dyn ReplicaWrapper>, TaskError> {
    #[cfg(feature = "taskchampion")]
    {
        // Run the non-Send taskchampion::Replica on a dedicated thread and
        // communicate with it via channels. This proxy is Send+Sync and
        // implements ReplicaWrapper without forcing Replica itself to be Send.
        use std::sync::mpsc;
        use std::thread;
    // PathBuf is available via std::path when needed; avoid unused-import warning
        use taskchampion::{Operations, TaskData};
        use taskchampion::storage::{StorageConfig, AccessMode};

        // Command enum for actor requests is declared at module scope below

        // Create channels and spawn the actor thread. The actor will create the
        // Replica from the provided path inside the thread (so we don't need
        // Replica to be Send) and reply to requests over response channels.
    let (cmd_tx, cmd_rx) = mpsc::channel::<ReplicaCommand>();
        let path_buf = path.to_path_buf();

    // The actor will use the replica-aware mapping helper
    // map_ops_to_tc_operations_with_replica to build Operations.

        // startup handshake channel
        let (startup_tx, startup_rx) = mpsc::channel();

    thread::Builder::new()
            .name("replica-taskchampion-actor".to_string())
            .spawn(move || {
                // Try to construct storage and replica inside the thread.
                let storage_res = StorageConfig::OnDisk {
                    taskdb_dir: path_buf.clone(),
                    create_if_missing: true,
                    access_mode: AccessMode::ReadWrite,
                }.into_storage();

                let mut replica = match storage_res {
                    Ok(storage) => match taskchampion::Replica::new(storage) {
                        rep => {
                            // warn: Replica::new returns value directly in this API
                            rep
                        }
                    },
                    Err(e) => {
                        let _ = startup_tx.send(Err(TaskError::Storage { source: StorageError::Database { message: format!("Failed to open TaskChampion storage: {e}") } }));
                        return;
                    }
                };
                        use std::sync::Arc;
                // signal successful startup
                let _ = startup_tx.send(Ok(()));

                // actor loop
                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        ReplicaCommand::Commit { ops, resp } => {
                            // Map our internal ops into taskchampion::Operations using the
                            // helper that prefers Task helper methods when possible.
                            match crate::storage::operation_batch::to_taskchampion_operations(&mut replica, &ops) {
                                Ok(tc_ops) => {
                                    let res = replica.commit_operations(tc_ops);
                                    let _ = match res {
                                        Ok(_) => resp.send(Ok(())),
                                        Err(e) => resp.send(Err(TaskError::Storage { source: StorageError::Database { message: format!("TaskChampion commit failed: {e}") } })),
                                    };
                                }
                                Err(e) => {
                                    let _ = resp.send(Err(TaskError::Storage { source: StorageError::Database { message: format!("TaskChampion mapping failed: {e}") } }));
                                }
                            }
                        }
                        ReplicaCommand::Open { path, resp } => {
                            // Attempt to replace replica by constructing a new one.
                            let storage_res = StorageConfig::OnDisk {
                                taskdb_dir: path.clone(),
                                create_if_missing: true,
                                access_mode: AccessMode::ReadWrite,
                            }.into_storage();
                            match storage_res {
                                Ok(storage) => {
                                    // create a new replica in-place
                                    replica = taskchampion::Replica::new(storage);
                                    let _ = resp.send(Ok(()));
                                }
                                Err(e) => {
                                    let _ = resp.send(Err(TaskError::Storage { source: StorageError::Database { message: format!("Failed to open TaskChampion storage: {e}") } }));
                                }
                            }
                        }
                        ReplicaCommand::ReadTask { id, resp } => {
                            // Query the replica's task data map and convert to our Task type.
                            match replica.all_task_data() {
                                Ok(map) => {
                                    if let Some(td) = map.get(&id) {
                                        // td is a map-like structure: &HashMap<String, String>
                                        // Build a Task from available fields.
                                        // Minimal fields: description, status, entry
                                        let description = td.get("description").map(|s| s.to_string()).unwrap_or_default();
                                        let status_str = td.get("status").map(|s| s.to_string()).unwrap_or_else(|| "pending".to_string());
                                        let status = match status_str.as_str() {
                                            "pending" => crate::task::model::TaskStatus::Pending,
                                            "completed" => crate::task::model::TaskStatus::Completed,
                                            "deleted" => crate::task::model::TaskStatus::Deleted,
                                            "waiting" => crate::task::model::TaskStatus::Waiting,
                                            _ => crate::task::model::TaskStatus::Pending,
                                        };
                                        let entry = td.get("entry").and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&chrono::Utc)).unwrap_or_else(chrono::Utc::now);

                                        // Start with a new Task and overwrite fields
                                        let mut task = crate::task::model::Task::new(description.clone());
                                        task.id = id;
                                        task.description = description;
                                        task.status = status;
                                        task.entry = entry;

                                        // project
                                        if let Some(proj) = td.get("project") {
                                            task.project = Some(proj.to_string());
                                        }

                                        // tags
                                        if let Some(tags_str) = td.get("tags") {
                                            let set: std::collections::HashSet<String> = tags_str.split_whitespace().map(|s| s.to_string()).collect();
                                            task.tags = set;
                                        }

                                        // timestamps: modified, due, scheduled, wait, end, start
                                        if let Some(mod_s) = td.get("modified") {
                                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(mod_s) {
                                                task.modified = Some(dt.with_timezone(&chrono::Utc));
                                            }
                                        }
                                        if let Some(due_s) = td.get("due") {
                                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(due_s) {
                                                task.due = Some(dt.with_timezone(&chrono::Utc));
                                            }
                                        }
                                        if let Some(sched_s) = td.get("scheduled") {
                                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(sched_s) {
                                                task.scheduled = Some(dt.with_timezone(&chrono::Utc));
                                            }
                                        }
                                        if let Some(wait_s) = td.get("wait") {
                                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(wait_s) {
                                                task.wait = Some(dt.with_timezone(&chrono::Utc));
                                            }
                                        }
                                        if let Some(end_s) = td.get("end") {
                                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(end_s) {
                                                task.end = Some(dt.with_timezone(&chrono::Utc));
                                            }
                                        }
                                        if let Some(start_s) = td.get("start") {
                                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(start_s) {
                                                task.start = Some(dt.with_timezone(&chrono::Utc));
                                            }
                                        }

                                        // priority
                                        if let Some(prio) = td.get("priority") {
                                            match &prio[..] {
                                                "H" => task.priority = Some(crate::task::model::Priority::High),
                                                "M" => task.priority = Some(crate::task::model::Priority::Medium),
                                                "L" => task.priority = Some(crate::task::model::Priority::Low),
                                                _ => {}
                                            }
                                        }

                                        // annotations: try keys 'annotations' or lines in a single string
                                        if let Some(anns_str) = td.get("annotations") {
                                            for line in anns_str.lines() {
                                                // Expect "<rfc3339> <description>"
                                                if let Some((ts, desc)) = line.split_once(' ') {
                                                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                                                        let ann = crate::task::annotation::Annotation::with_timestamp(desc.replace("\\n", "\n"), dt.with_timezone(&chrono::Utc));
                                                        task.annotations.push(ann);
                                                    } else {
                                                        // fallback: store whole line as description with current time
                                                        let ann = crate::task::annotation::Annotation::new(line.to_string());
                                                        task.annotations.push(ann);
                                                    }
                                                } else {
                                                    let ann = crate::task::annotation::Annotation::new(line.to_string());
                                                    task.annotations.push(ann);
                                                }
                                            }
                                        }

                                        // dependencies
                                        if let Some(dep_str) = td.get("depends") {
                                            let mut deps = std::collections::HashSet::new();
                                            for token in dep_str.split_whitespace() {
                                                if let Ok(u) = Uuid::parse_str(token) {
                                                    deps.insert(u);
                                                }
                                            }
                                            task.depends = deps;
                                        }

                                        // recurrence
                                        if let Some(recur_s) = td.get("recur") {
                                            if let Ok(rp) = crate::task::recurrence::RecurrencePattern::parse(recur_s) {
                                                task.recur = Some(rp);
                                            }
                                        }

                                        // parent, mask
                                        if let Some(parent_s) = td.get("parent") {
                                            if let Ok(u) = Uuid::parse_str(parent_s) {
                                                task.parent = Some(u);
                                            }
                                        }
                                        if let Some(mask_s) = td.get("mask") {
                                            task.mask = Some(mask_s.to_string());
                                        }

                                        // active flag
                                        if let Some(active_s) = td.get("active") {
                                            let s = &active_s[..];
                                            task.active = matches!(s, "1" | "true" | "True");
                                        }

                                        // UDAs: any key not recognized above and not in a list of standard fields
                                        let standard = ["description","status","entry","project","tags","modified","due","scheduled","wait","end","start","priority","annotations","depends","recur","parent","mask","active","id","uuid"];
                                        for (k, v) in td.iter() {
                                            if standard.contains(&k.as_str()) { continue; }
                                            // Try to parse number
                                            if let Ok(n) = v.parse::<f64>() {
                                                task.udas.insert(k.clone(), crate::task::model::UdaValue::Number(n));
                                                continue;
                                            }
                                            // Try date
                                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                                                task.udas.insert(k.clone(), crate::task::model::UdaValue::Date(dt.with_timezone(&chrono::Utc)));
                                                continue;
                                            }
                                            // Fallback to string
                                            task.udas.insert(k.clone(), crate::task::model::UdaValue::String(v.clone()));
                                        }

                                        let _ = resp.send(Ok(Some(task)));
                                    } else {
                                        let _ = resp.send(Ok(None));
                                    }
                                }
                                Err(e) => {
                                    let _ = resp.send(Err(TaskError::Storage { source: StorageError::Database { message: format!("Failed to read replica task data: {e}") } }));
                                }
                            }
                        }
                    }
                }
            }).map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("Failed to spawn replica actor thread: {e}") } })?;

        // Wait for startup handshake
        use std::time::Duration;
        match startup_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => {
                let proxy = ReplicaTaskChampionActor { sender: Arc::new(Mutex::new(cmd_tx)) };
                return Ok(Box::new(proxy));
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(TaskError::Storage { source: StorageError::Database { message: "Timed out waiting for replica actor startup".to_string() } }),
        }
    }

    // Fallback stub when feature is not enabled
    #[cfg(not(feature = "taskchampion"))]
    {
        // consume path to avoid unused variable warning when feature is disabled
        let _ = path;
        Ok(Box::new(ReplicaTaskChampionStub))
    }
}

// Provide a simple stub type for completeness when feature is disabled.
pub struct ReplicaTaskChampionStub;

impl ReplicaWrapper for ReplicaTaskChampionStub {
    fn commit_operations(&mut self, _ops: &[Op]) -> Result<(), TaskError> {
        Err(TaskError::Storage {
            source: StorageError::Database { message: "TaskChampion replica not available".to_string() },
        })
    }

    fn open(&mut self, _path: &Path) -> Result<(), TaskError> {
        Err(TaskError::Storage {
            source: StorageError::Database { message: "TaskChampion replica not available".to_string() },
        })
    }

    fn read_task(&self, _id: Uuid) -> Result<Option<crate::task::Task>, TaskError> {
        Err(TaskError::Storage {
            source: StorageError::Database { message: "TaskChampion replica not available".to_string() },
        })
    }
}

// The actor-based proxy implementation is below. We intentionally avoid
// creating a direct Replica value in this module to prevent Send/Sync issues.

#[cfg(feature = "taskchampion")]
struct ReplicaTaskChampionActor {
    // Sender is protected by Mutex only to satisfy Send+Sync; mpsc::Sender is
    // already Send, but wrapping keeps the field Sync for the boxed trait object.
    sender: Arc<Mutex<std::sync::mpsc::Sender<ReplicaCommand>>>,
}

#[cfg(feature = "taskchampion")]
impl ReplicaWrapper for ReplicaTaskChampionActor {
    fn commit_operations(&mut self, ops: &[Op]) -> Result<(), TaskError> {
        let (tx, rx) = std::sync::mpsc::channel();
    let cmd = ReplicaCommand::Commit { ops: ops.to_vec(), resp: tx };
        // Acquire lock briefly to send
        let guard = self.sender.lock().map_err(|_| TaskError::Storage { source: StorageError::Database { message: "Replica actor sender mutex poisoned".to_string() } })?;
        guard.send(cmd).map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("Failed to send commit command to replica actor: {e}") } })?;
        rx.recv().map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("No response from replica actor: {e}") } })??;
        Ok(())
    }

    fn open(&mut self, path: &Path) -> Result<(), TaskError> {
        let (tx, rx) = std::sync::mpsc::channel();
    let cmd = ReplicaCommand::Open { path: path.to_path_buf(), resp: tx };
        let guard = self.sender.lock().map_err(|_| TaskError::Storage { source: StorageError::Database { message: "Replica actor sender mutex poisoned".to_string() } })?;
        guard.send(cmd).map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("Failed to send open command to replica actor: {e}") } })?;
        rx.recv().map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("No response from replica actor: {e}") } })??;
        Ok(())
    }

    fn read_task(&self, _id: Uuid) -> Result<Option<crate::task::Task>, TaskError> {
        let (tx, rx) = std::sync::mpsc::channel();
        let cmd = ReplicaCommand::ReadTask { id: _id, resp: tx };
        let guard = self.sender.lock().map_err(|_| TaskError::Storage { source: StorageError::Database { message: "Replica actor sender mutex poisoned".to_string() } })?;
        guard.send(cmd).map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("Failed to send read command to replica actor: {e}") } })?;
        let res = rx.recv().map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("No response from replica actor: {e}") } })?;
        res
    }
}

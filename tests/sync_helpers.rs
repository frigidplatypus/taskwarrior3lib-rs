use taskwarrior3lib::io::{ProcessResult, ProcessRunner};
use taskwarrior3lib::sync::helpers::run_task_sync_and_reload_replica;
use tempfile::tempdir;

#[derive(Default)]
struct FakeRunnerSuccess {
    pub last_cmd: std::sync::Mutex<Option<(String, Vec<String>)>>,
}

impl ProcessRunner for FakeRunnerSuccess {
    fn run(&self, cmd: &str, args: &[&str], _timeout: Option<std::time::Duration>) -> std::io::Result<ProcessResult> {
        let mut guard = self.last_cmd.lock().unwrap();
        *guard = Some((cmd.to_string(), args.iter().map(|s| s.to_string()).collect()));
        Ok(ProcessResult { exit_code: 0, stdout: "ok".into(), stderr: "".into() })
    }
}

#[test]
fn test_run_task_sync_and_reload_replica_success() {
    let runner = FakeRunnerSuccess::default();
    let dir = tempdir().unwrap();
    // Create an empty file to mimic a DB; initialize() will fail if missing, so touch the file
    let db_path = dir.path().join("taskchampion.sqlite3");
    std::fs::File::create(&db_path).unwrap();

    // With the runner succeeding and a present DB file, the helper should return Ok(())
    let res = run_task_sync_and_reload_replica(&runner, &db_path, None);
    assert!(res.is_ok(), "expected Ok from helper, got {:?}", res);
}

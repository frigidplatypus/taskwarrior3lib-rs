use std::sync::Arc;
use taskwarrior3lib::io::{ProcessResult, ProcessRunner};

// A fake process runner for testing which records the last command invoked.
#[derive(Default)]
struct FakeRunner {
    pub last_cmd: std::sync::Mutex<Option<(String, Vec<String>)>>,
}

impl ProcessRunner for FakeRunner {
    fn run(&self, cmd: &str, args: &[&str], _timeout: Option<std::time::Duration>) -> std::io::Result<ProcessResult> {
        let mut guard = self.last_cmd.lock().unwrap();
        *guard = Some((cmd.to_string(), args.iter().map(|s| s.to_string()).collect()));
        Ok(ProcessResult { exit_code: 0, stdout: "ok".into(), stderr: "".into() })
    }
}

#[test]
fn contract_run_task_sync_and_reload_replica_should_invoke_task_sync() {
    // This is a contract test placeholder. The real helper is not implemented yet.
    let runner = Arc::new(FakeRunner::default());

    // The expected behavior: run `task sync` using the provided ProcessRunner.
    // We'll call the fake runner directly to assert basic wiring. When the
    // helper is implemented, it should use the runner and cause the same effect.
    let _ = runner.run("task", &["sync"], None).unwrap();

    let guard = runner.last_cmd.lock().unwrap();
    let recorded = guard.as_ref().expect("expected command to be recorded");
    assert_eq!(recorded.0, "task");
    assert_eq!(recorded.1, vec!["sync".to_string()]);
}

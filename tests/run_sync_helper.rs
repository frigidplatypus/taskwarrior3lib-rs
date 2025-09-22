use std::sync::Arc;
use taskwarrior3lib::io::{ProcessResult, ProcessRunner};
use taskwarrior3lib::io::process_runner::ProcessError;

#[derive(Default)]
struct FakeRunner {
    pub last_cmd: std::sync::Mutex<Option<(String, Vec<String>)>>,
}

impl ProcessRunner for FakeRunner {
    fn run(&self, cmd: &str, args: &[&str], _timeout: Option<std::time::Duration>) -> Result<ProcessResult, ProcessError> {
        let mut guard = self.last_cmd.lock().unwrap();
        *guard = Some((cmd.to_string(), args.iter().map(|s| s.to_string()).collect()));
        Ok(ProcessResult { exit_code: 0, stdout: "ok".into(), stderr: "".into() })
    }
}

#[test]
fn run_sync_helper_contract_smoke() {
    let runner = Arc::new(FakeRunner::default());
    let _ = runner.run("task", &["sync"], None).unwrap();
    let guard = runner.last_cmd.lock().unwrap();
    let recorded = guard.as_ref().expect("expected command to be recorded");
    assert_eq!(recorded.0, "task");
    assert_eq!(recorded.1, vec!["sync".to_string()]);
}

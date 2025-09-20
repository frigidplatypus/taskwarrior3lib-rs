use std::process::Stdio;
use std::time::Duration;

/// Result of running a process via the ProcessRunner.
#[derive(Debug, PartialEq, Eq)]
pub struct ProcessResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Trait used to run external processes. This allows tests to inject a fake runner.
pub trait ProcessRunner: Send + Sync {
    /// Run the provided command with args, returning the ProcessResult or an io::Error.
    fn run(&self, cmd: &str, args: &[&str], timeout: Option<Duration>) -> std::io::Result<ProcessResult>;
}

/// System implementation that shells out using std::process::Command.
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn run(&self, cmd: &str, args: &[&str], _timeout: Option<Duration>) -> std::io::Result<ProcessResult> {
        let mut c = std::process::Command::new(cmd);
        c.args(args);
        c.stdin(Stdio::null());
        c.stdout(Stdio::piped());
        c.stderr(Stdio::piped());

        let output = c.output()?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(ProcessResult {
            exit_code,
            stdout,
            stderr,
        })
    }
}

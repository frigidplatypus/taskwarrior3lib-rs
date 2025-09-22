use std::process::Stdio;
use std::time::Duration;

/// Result of running a process via the ProcessRunner.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProcessResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Error from running a process
#[derive(thiserror::Error, Debug)]
pub enum ProcessError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Timeout")]
    Timeout,
}

/// Trait used to run external processes. This allows tests to inject a fake runner.
pub trait ProcessRunner: Send + Sync {
    /// Run the provided command with args, returning the ProcessResult or ProcessError.
    fn run(&self, cmd: &str, args: &[&str], timeout: Option<Duration>) -> Result<ProcessResult, ProcessError>;
}

/// System implementation that shells out using std::process::Command.
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn run(&self, cmd: &str, args: &[&str], _timeout: Option<Duration>) -> Result<ProcessResult, ProcessError> {
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

/// Convenience function to get the default process runner
pub fn default_runner() -> Box<dyn ProcessRunner> {
    Box::new(SystemProcessRunner)
}

#[cfg(any(test, feature = "taskchampion"))]
/// Mock implementation for testing
pub struct MockProcessRunner<F>
where
    F: Fn(&str, &[&str], Option<Duration>) -> Result<ProcessResult, ProcessError> + Send + Sync,
{
    pub run_fn: F,
}

#[cfg(any(test, feature = "taskchampion"))]
impl<F> ProcessRunner for MockProcessRunner<F>
where
    F: Fn(&str, &[&str], Option<Duration>) -> Result<ProcessResult, ProcessError> + Send + Sync,
{
    fn run(&self, cmd: &str, args: &[&str], timeout: Option<Duration>) -> Result<ProcessResult, ProcessError> {
        (self.run_fn)(cmd, args, timeout)
    }
}

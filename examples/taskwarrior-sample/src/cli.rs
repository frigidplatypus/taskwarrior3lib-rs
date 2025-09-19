use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "taskwarrior-sample")]
#[command(about = "Taskwarrior Library Sample CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new task
    Add {
        /// Task description
        description: String,
        /// Project name
        #[arg(short, long)]
        project: Option<String>,
    },
    /// List tasks
    List,
    /// Mark task as complete
    Done {
        /// Task ID
        id: String,
    },
    /// Import tasks from system Taskwarrior
    Import,
    /// Debug information
    Debug,
}

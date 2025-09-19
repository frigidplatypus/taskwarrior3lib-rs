#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use taskwarrior_sample::cli::Cli;

    #[test]
    fn test_command_parsing_unit() {
        // Ensure CLI parses 'add' command
        let cli = Cli::command().try_get_matches_from(vec!["taskwarrior-sample", "add", "Test parsing"]);
        assert!(cli.is_ok(), "Parsing 'add' command should succeed");

        // Ensure CLI fails on unknown command
        let cli = Cli::command().try_get_matches_from(vec!["taskwarrior-sample", "unknown"]);
        assert!(cli.is_err(), "Parsing unknown command should fail");
    }
}

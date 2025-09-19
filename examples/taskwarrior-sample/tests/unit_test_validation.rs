#[cfg(test)]
mod tests {
    use taskwarrior_sample::models::Task;

    #[test]
    fn test_validation_logic_unit() {
        // Valid task
        let valid_task = Task::new("Valid description".to_string());
        assert!(valid_task.validate().is_ok(), "Valid task should pass validation");

        // Invalid task (empty description)
        let invalid_task = Task::new("".to_string());
        assert!(invalid_task.validate().is_err(), "Empty description should fail validation");
    }
}

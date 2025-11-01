#[cfg(feature = "persistence")]
mod persistence_tests {
    use protest::*;
    use tempfile::TempDir;

    #[test]
    fn test_failure_snapshot_basic() {
        let temp_dir = TempDir::new().unwrap();
        let failure_dir = temp_dir.path();

        let snapshot = FailureSnapshot::new(failure_dir).unwrap();

        // Create and save a failure
        let failure_case =
            FailureCase::new(42, "100".to_string(), "Value too large".to_string(), 5);

        snapshot
            .save_failure("test_example", &failure_case)
            .unwrap();

        // Load and verify
        let failures = snapshot.load_failures("test_example").unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].seed, 42);
        assert_eq!(failures[0].input, "100");
        assert!(failures[0].error_message.contains("Value too large"));
        assert_eq!(failures[0].shrink_steps, 5);
    }

    #[test]
    fn test_corpus_management() {
        let temp_dir = TempDir::new().unwrap();
        let corpus_dir = temp_dir.path().join("corpus");

        let mut corpus = TestCorpus::new(&corpus_dir).unwrap();

        // Add interesting test cases
        corpus
            .add_case("edge_case_1".to_string(), "Boundary value".to_string())
            .unwrap();

        corpus
            .add_case(
                "edge_case_2".to_string(),
                "Another interesting case".to_string(),
            )
            .unwrap();

        // Reload and verify
        let cases = corpus.load_all().unwrap();
        assert_eq!(cases.len(), 2);
        assert!(cases.iter().any(|c| c.input == "edge_case_1"));
        assert!(cases.iter().any(|c| c.input == "edge_case_2"));
    }

    #[test]
    fn test_failure_delete() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = FailureSnapshot::new(temp_dir.path()).unwrap();

        // Save a failure
        let failure = FailureCase::new(123, "test".to_string(), "error".to_string(), 0);
        snapshot.save_failure("test", &failure).unwrap();

        // Verify it exists
        let failures = snapshot.load_failures("test").unwrap();
        assert_eq!(failures.len(), 1);

        // Delete it
        snapshot.delete_failure("test", 123).unwrap();

        // Verify it's gone
        let failures = snapshot.load_failures("test").unwrap();
        assert_eq!(failures.len(), 0);
    }

    #[test]
    fn test_list_tests_with_failures() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = FailureSnapshot::new(temp_dir.path()).unwrap();

        // Save failures for multiple tests
        let failure1 = FailureCase::new(1, "a".to_string(), "e".to_string(), 0);
        let failure2 = FailureCase::new(2, "b".to_string(), "e".to_string(), 0);

        snapshot.save_failure("test_a", &failure1).unwrap();
        snapshot.save_failure("test_b", &failure2).unwrap();

        // List all tests
        let tests = snapshot.list_tests_with_failures().unwrap();
        assert_eq!(tests.len(), 2);
        assert!(tests.contains(&"test_a".to_string()));
        assert!(tests.contains(&"test_b".to_string()));
    }

    #[test]
    fn test_failure_with_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = FailureSnapshot::new(temp_dir.path()).unwrap();

        // Create failure with metadata
        let failure = FailureCase::new(42, "input".to_string(), "error".to_string(), 5)
            .with_metadata("git_commit".to_string(), "abc123".to_string())
            .with_metadata("environment".to_string(), "ci".to_string());

        snapshot.save_failure("test", &failure).unwrap();

        // Load and verify metadata
        let failures = snapshot.load_failures("test").unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].metadata.get("git_commit").unwrap(), "abc123");
        assert_eq!(failures[0].metadata.get("environment").unwrap(), "ci");
    }

    // Simple Property implementation for testing
    struct FailsOver50;
    impl protest::Property<u32> for FailsOver50 {
        type Output = ();
        fn test(&self, x: u32) -> Result<(), protest::PropertyError> {
            if x > 50 {
                Err(protest::PropertyError::property_failed("Value too large"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_automatic_replay_with_fixed_failure() {
        let temp_dir = TempDir::new().unwrap();
        let failure_dir = temp_dir.path().join("failures");

        // Create persistence config
        let persistence_cfg = PersistenceConfig::enabled().with_failure_dir(failure_dir.clone());

        // First run: property will fail for values > 50
        use protest::Arbitrary;
        let result = PropertyTestBuilder::new()
            .persistence_config(persistence_cfg.clone())
            .test_name("replay_test")
            .iterations(10)
            .seed(12345)
            .run(u32::arbitrary(), FailsOver50);

        // Should fail
        assert!(result.is_err());

        // Second run: Same property, will replay saved failure first
        let result2 = PropertyTestBuilder::new()
            .persistence_config(persistence_cfg.clone())
            .test_name("replay_test")
            .iterations(10)
            .seed(67890) // Different seed, but replay will use the saved seed
            .run(u32::arbitrary(), FailsOver50);

        // Should fail again (replay will catch it)
        assert!(result2.is_err());

        // The replay should have been executed before the main test
        // Check that the failure was saved
        let snapshot = FailureSnapshot::new(&failure_dir).unwrap();
        let failures = snapshot.load_failures("replay_test").unwrap();
        assert!(!failures.is_empty(), "Failure should have been saved");
    }

    // Always passing property for cleanup test
    struct AlwaysPass;
    impl protest::Property<u32> for AlwaysPass {
        type Output = ();
        fn test(&self, _x: u32) -> Result<(), protest::PropertyError> {
            Ok(())
        }
    }

    #[test]
    fn test_automatic_replay_cleans_up_fixed_failures() {
        let temp_dir = TempDir::new().unwrap();
        let failure_dir = temp_dir.path().join("failures");

        let persistence_cfg = PersistenceConfig::enabled().with_failure_dir(failure_dir.clone());

        // First run: property will fail
        use protest::Arbitrary;
        let result = PropertyTestBuilder::new()
            .persistence_config(persistence_cfg.clone())
            .test_name("cleanup_test")
            .iterations(10)
            .seed(12345)
            .run(u32::arbitrary(), FailsOver50);

        // Should fail
        assert!(result.is_err());

        // Verify failure was saved
        let snapshot = FailureSnapshot::new(&failure_dir).unwrap();
        let failures = snapshot.load_failures("cleanup_test").unwrap();
        assert!(!failures.is_empty(), "Failure should have been saved");
        let saved_seed = failures[0].seed;

        // Second run: property now passes, replay should clean up
        let result2 = PropertyTestBuilder::new()
            .persistence_config(persistence_cfg.clone())
            .test_name("cleanup_test")
            .iterations(10)
            .seed(99999)
            .run(u32::arbitrary(), AlwaysPass);

        // Should pass
        assert!(result2.is_ok());

        // Verify failure was cleaned up
        let failures_after = snapshot.load_failures("cleanup_test").unwrap();
        assert!(
            !failures_after.iter().any(|f| f.seed == saved_seed),
            "Fixed failure should have been cleaned up"
        );
    }
}

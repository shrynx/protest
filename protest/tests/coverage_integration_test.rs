#[cfg(feature = "persistence")]
mod coverage_tests {
    use protest::*;
    use tempfile::TempDir;

    #[test]
    fn test_coverage_tracker_records_paths() {
        let tracker = CoverageTracker::new();

        // Record different paths for different inputs
        tracker.record_path("input1", 100);
        tracker.record_path("input1", 200);
        tracker.record_path("input2", 100); // Same path, different input
        tracker.record_path("input3", 300);

        // Should have 3 unique paths
        assert_eq!(tracker.unique_paths(), 3);
    }

    #[test]
    fn test_coverage_corpus_adds_interesting_inputs() {
        let temp_dir = TempDir::new().unwrap();
        let config = CoverageCorpusConfig::new(temp_dir.path())
            .with_min_coverage(1.0)
            .with_max_size(100);

        let mut corpus = CoverageCorpus::new(config).unwrap();

        // Add inputs with different path hashes
        let added1 = corpus.try_add(&42, 100).unwrap();
        let added2 = corpus.try_add(&43, 200).unwrap();
        let added3 = corpus.try_add(&44, 300).unwrap();

        // All should be added as they provide new paths
        assert!(added1);
        assert!(added2);
        assert!(added3);

        // Check corpus size
        let size = corpus.corpus_size().unwrap();
        assert_eq!(size, 3);
    }

    #[test]
    fn test_coverage_corpus_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = CoverageCorpusConfig::new(temp_dir.path());

        let mut corpus = CoverageCorpus::new(config).unwrap();

        corpus.try_add(&100, path_hash(&[1, 2, 3])).unwrap();
        corpus.try_add(&200, path_hash(&[4, 5, 6])).unwrap();

        let stats = corpus.stats();
        assert!(stats.total_paths > 0);
        assert_eq!(stats.corpus_size, 2);
    }

    #[test]
    fn test_path_hash_consistency() {
        // Same values should produce same hash
        let hash1 = path_hash(&[1, 2, 3, 4, 5]);
        let hash2 = path_hash(&[1, 2, 3, 4, 5]);
        assert_eq!(hash1, hash2);

        // Different values should produce different hash
        let hash3 = path_hash(&[5, 4, 3, 2, 1]);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_coverage_corpus_config() {
        let config = CoverageCorpusConfig::new("/tmp/corpus")
            .with_min_coverage(5.0)
            .with_max_size(500)
            .auto_optimize(false);

        assert_eq!(config.min_coverage_increase, 5.0);
        assert_eq!(config.max_corpus_size, 500);
        assert!(!config.auto_optimize);
    }

    #[test]
    fn test_coverage_tracker_input_coverage() {
        let tracker = CoverageTracker::new();

        tracker.record_path("input1", 100);
        tracker.record_path("input1", 200);
        tracker.record_path("input1", 300);

        tracker.record_path("input2", 100);

        // input1 should have higher coverage
        let coverage1 = tracker.input_coverage_percent("input1");
        let coverage2 = tracker.input_coverage_percent("input2");

        assert!(coverage1 > coverage2);
    }

    #[test]
    fn test_coverage_inputs_by_coverage() {
        let tracker = CoverageTracker::new();

        tracker.record_path("input_a", 1);
        tracker.record_path("input_a", 2);
        tracker.record_path("input_a", 3);

        tracker.record_path("input_b", 4);
        tracker.record_path("input_b", 5);

        tracker.record_path("input_c", 6);

        let by_coverage = tracker.inputs_by_coverage();

        // Should be sorted by coverage (descending)
        assert_eq!(by_coverage.len(), 3);
        assert_eq!(by_coverage[0].1, 3); // input_a has 3 paths
        assert_eq!(by_coverage[1].1, 2); // input_b has 2 paths
        assert_eq!(by_coverage[2].1, 1); // input_c has 1 path
    }
}

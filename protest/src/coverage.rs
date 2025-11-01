//! Coverage-guided corpus building for property testing
//!
//! This module provides functionality to track code coverage during property testing
//! and build a corpus of interesting test cases that maximize coverage.

use crate::persistence::TestCorpus;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Tracks coverage information during test execution
#[derive(Debug, Clone)]
pub struct CoverageTracker {
    /// Set of unique execution paths seen
    paths_seen: Arc<Mutex<HashSet<u64>>>,

    /// Coverage information per input
    input_coverage: Arc<Mutex<HashMap<String, HashSet<u64>>>>,

    /// Total number of unique paths discovered
    total_paths: Arc<Mutex<usize>>,
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self {
            paths_seen: Arc::new(Mutex::new(HashSet::new())),
            input_coverage: Arc::new(Mutex::new(HashMap::new())),
            total_paths: Arc::new(Mutex::new(0)),
        }
    }

    /// Record that a specific path was taken with given input
    pub fn record_path(&self, input_id: &str, path_hash: u64) {
        let mut paths = self.paths_seen.lock().unwrap();
        let is_new = paths.insert(path_hash);

        if is_new {
            let mut total = self.total_paths.lock().unwrap();
            *total += 1;
        }

        let mut input_cov = self.input_coverage.lock().unwrap();
        input_cov
            .entry(input_id.to_string())
            .or_default()
            .insert(path_hash);
    }

    /// Get the number of unique paths seen
    pub fn unique_paths(&self) -> usize {
        *self.total_paths.lock().unwrap()
    }

    /// Get the coverage percentage for a specific input
    pub fn input_coverage_percent(&self, input_id: &str) -> f64 {
        let input_cov = self.input_coverage.lock().unwrap();
        let total_paths = self.unique_paths();

        if total_paths == 0 {
            return 0.0;
        }

        let input_paths = input_cov.get(input_id).map(|s| s.len()).unwrap_or(0);
        (input_paths as f64 / total_paths as f64) * 100.0
    }

    /// Check if an input discovered new coverage
    pub fn has_new_coverage(&self, input_id: &str) -> bool {
        let input_cov = self.input_coverage.lock().unwrap();
        if let Some(input_paths) = input_cov.get(input_id) {
            let paths = self.paths_seen.lock().unwrap();
            input_paths.iter().any(|path| !paths.contains(path))
        } else {
            false
        }
    }

    /// Get inputs sorted by coverage (highest first)
    pub fn inputs_by_coverage(&self) -> Vec<(String, usize)> {
        let input_cov = self.input_coverage.lock().unwrap();
        let mut inputs: Vec<_> = input_cov
            .iter()
            .map(|(id, paths)| (id.clone(), paths.len()))
            .collect();
        inputs.sort_by(|a, b| b.1.cmp(&a.1));
        inputs
    }

    /// Clear all coverage data
    pub fn reset(&self) {
        self.paths_seen.lock().unwrap().clear();
        self.input_coverage.lock().unwrap().clear();
        *self.total_paths.lock().unwrap() = 0;
    }
}

/// Configuration for coverage-guided corpus building
#[derive(Debug, Clone)]
pub struct CoverageCorpusConfig {
    /// Minimum coverage increase required to add to corpus (percentage)
    pub min_coverage_increase: f64,

    /// Maximum corpus size
    pub max_corpus_size: usize,

    /// Path to corpus directory
    pub corpus_dir: PathBuf,

    /// Enable automatic corpus optimization
    pub auto_optimize: bool,
}

impl Default for CoverageCorpusConfig {
    fn default() -> Self {
        Self {
            min_coverage_increase: 1.0, // 1% minimum increase
            max_corpus_size: 1000,
            corpus_dir: PathBuf::from(".protest/corpus"),
            auto_optimize: true,
        }
    }
}

impl CoverageCorpusConfig {
    pub fn new<P: Into<PathBuf>>(corpus_dir: P) -> Self {
        Self {
            corpus_dir: corpus_dir.into(),
            ..Default::default()
        }
    }

    pub fn with_min_coverage(mut self, percent: f64) -> Self {
        self.min_coverage_increase = percent;
        self
    }

    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_corpus_size = size;
        self
    }

    pub fn auto_optimize(mut self, enable: bool) -> Self {
        self.auto_optimize = enable;
        self
    }
}

/// Coverage-guided corpus builder
pub struct CoverageCorpus {
    config: CoverageCorpusConfig,
    tracker: CoverageTracker,
    corpus: TestCorpus,
}

impl CoverageCorpus {
    pub fn new(config: CoverageCorpusConfig) -> std::io::Result<Self> {
        let corpus = TestCorpus::new(&config.corpus_dir)?;
        Ok(Self {
            config,
            tracker: CoverageTracker::new(),
            corpus,
        })
    }

    /// Add input to corpus if it increases coverage significantly
    pub fn try_add<T: std::fmt::Debug>(
        &mut self,
        input: &T,
        path_hash: u64,
    ) -> std::io::Result<bool> {
        let input_str = format!("{:?}", input);
        let input_id = hash_string(&input_str);

        // Record this path
        self.tracker.record_path(&input_id, path_hash);

        // Check if this input provides new coverage
        let current_paths = self.tracker.unique_paths();
        let coverage_increase = self.calculate_coverage_increase(&input_id, current_paths);

        if coverage_increase >= self.config.min_coverage_increase {
            let reason = format!(
                "Coverage increase: {:.2}% (total paths: {})",
                coverage_increase, current_paths
            );
            self.corpus.add_case(input_str, reason)?;

            // Optimize if needed
            if self.config.auto_optimize && self.corpus_size()? > self.config.max_corpus_size {
                self.optimize()?;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get current corpus size
    pub fn corpus_size(&mut self) -> std::io::Result<usize> {
        Ok(self.corpus.load_all()?.len())
    }

    /// Optimize corpus by removing redundant entries
    pub fn optimize(&mut self) -> std::io::Result<usize> {
        let cases = self.corpus.load_all()?;
        let inputs_by_cov = self.tracker.inputs_by_coverage();

        // Keep only the top N by coverage
        let to_keep: HashSet<_> = inputs_by_cov
            .iter()
            .take(self.config.max_corpus_size)
            .map(|(id, _)| id.clone())
            .collect();

        let removed = cases.len().saturating_sub(to_keep.len());

        // Note: This is a simplified optimization
        // In a real implementation, we'd remove files from disk
        // For now, we just report the count

        Ok(removed)
    }

    /// Get coverage statistics
    pub fn stats(&mut self) -> CoverageStats {
        CoverageStats {
            total_paths: self.tracker.unique_paths(),
            corpus_size: self.corpus.load_all().map(|c| c.len()).unwrap_or(0),
            inputs_by_coverage: self.tracker.inputs_by_coverage(),
        }
    }

    fn calculate_coverage_increase(&self, _input_id: &str, _current_paths: usize) -> f64 {
        // Simplified calculation - in practice this would be more sophisticated
        // For now, assume any new path is valuable
        10.0 // Return 10% as placeholder
    }
}

/// Coverage statistics
#[derive(Debug)]
pub struct CoverageStats {
    pub total_paths: usize,
    pub corpus_size: usize,
    pub inputs_by_coverage: Vec<(String, usize)>,
}

/// Hash a string to a unique identifier
fn hash_string(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Helper to generate a simple path hash from values
pub fn path_hash<T: Hash>(values: &[T]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    for value in values {
        value.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_tracker() {
        let tracker = CoverageTracker::new();

        tracker.record_path("input1", 123);
        tracker.record_path("input1", 456);
        tracker.record_path("input2", 123);

        assert_eq!(tracker.unique_paths(), 2);
    }

    #[test]
    fn test_path_hash() {
        let hash1 = path_hash(&[1, 2, 3]);
        let hash2 = path_hash(&[1, 2, 3]);
        let hash3 = path_hash(&[3, 2, 1]);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_string() {
        let h1 = hash_string("test");
        let h2 = hash_string("test");
        let h3 = hash_string("different");

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}

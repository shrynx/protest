//! Test case persistence and replay functionality
//!
//! This module provides tools for saving failing test cases, managing test corpuses,
//! and replaying tests deterministically for debugging and regression testing.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A saved test failure case with all necessary information for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureCase {
    /// The RNG seed that produced this failure
    pub seed: u64,

    /// Serialized representation of the failing input
    pub input: String,

    /// The error message from the failure
    pub error_message: String,

    /// When this failure was recorded
    pub timestamp: SystemTime,

    /// Number of shrink steps that were performed
    pub shrink_steps: usize,

    /// Optional metadata about the test
    pub metadata: HashMap<String, String>,
}

impl FailureCase {
    /// Create a new failure case
    pub fn new(seed: u64, input: String, error_message: String, shrink_steps: usize) -> Self {
        Self {
            seed,
            input,
            error_message,
            timestamp: SystemTime::now(),
            shrink_steps,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to this failure case
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Deserialize the stored input back to its original type
    /// Returns None if deserialization fails
    pub fn deserialize_input<T>(&self) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_str(&self.input).ok()
    }
}

/// Snapshot storage for test failures - saves and manages failing test cases
pub struct FailureSnapshot {
    /// Root directory for storing failures
    root_dir: PathBuf,
}

impl FailureSnapshot {
    /// Create a new failure snapshot storage at the given directory
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let root_dir = path.as_ref().to_path_buf();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    /// Save a failure case for a specific test
    pub fn save_failure(&self, test_name: &str, failure: &FailureCase) -> io::Result<PathBuf> {
        let test_dir = self.root_dir.join(test_name);
        fs::create_dir_all(&test_dir)?;

        // Generate filename based on seed
        let filename = format!("failure_seed_{}.json", failure.seed);
        let path = test_dir.join(filename);

        // Serialize and write
        let json = serde_json::to_string_pretty(failure)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut file = File::create(&path)?;
        file.write_all(json.as_bytes())?;

        Ok(path)
    }

    /// Load all failures for a specific test
    pub fn load_failures(&self, test_name: &str) -> io::Result<Vec<FailureCase>> {
        let test_dir = self.root_dir.join(test_name);

        if !test_dir.exists() {
            return Ok(Vec::new());
        }

        let mut failures = Vec::new();

        for entry in fs::read_dir(&test_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                if let Ok(failure) = serde_json::from_str::<FailureCase>(&contents) {
                    failures.push(failure);
                }
            }
        }

        Ok(failures)
    }

    /// Delete a specific failure case
    pub fn delete_failure(&self, test_name: &str, seed: u64) -> io::Result<()> {
        let filename = format!("failure_seed_{}.json", seed);
        let path = self.root_dir.join(test_name).join(filename);

        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// List all test names that have saved failures
    pub fn list_tests_with_failures(&self) -> io::Result<Vec<String>> {
        let mut tests = Vec::new();

        for entry in fs::read_dir(&self.root_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir()
                && let Some(name) = path.file_name().and_then(|s| s.to_str())
            {
                tests.push(name.to_string());
            }
        }

        Ok(tests)
    }

    /// Clear all failures for a specific test
    pub fn clear_test_failures(&self, test_name: &str) -> io::Result<()> {
        let test_dir = self.root_dir.join(test_name);

        if test_dir.exists() {
            fs::remove_dir_all(test_dir)?;
        }

        Ok(())
    }
}

/// Manager for test corpus - interesting test cases that should be reused
pub struct TestCorpus {
    /// Directory where corpus files are stored
    corpus_dir: PathBuf,

    /// Cached interesting cases
    cached_cases: Vec<CorpusCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusCase {
    /// The input that was interesting
    pub input: String,

    /// Why this case is interesting
    pub reason: String,

    /// When it was added
    pub timestamp: SystemTime,

    /// Optional tags for categorization
    pub tags: Vec<String>,
}

impl TestCorpus {
    /// Create a new test corpus at the given directory
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let corpus_dir = path.as_ref().to_path_buf();
        fs::create_dir_all(&corpus_dir)?;

        Ok(Self {
            corpus_dir,
            cached_cases: Vec::new(),
        })
    }

    /// Add an interesting case to the corpus
    pub fn add_case(&mut self, input: String, reason: String) -> io::Result<()> {
        let case = CorpusCase {
            input,
            reason,
            timestamp: SystemTime::now(),
            tags: Vec::new(),
        };

        self.add_corpus_case(case)
    }

    /// Add a corpus case with tags
    pub fn add_corpus_case(&mut self, case: CorpusCase) -> io::Result<()> {
        // Generate filename based on timestamp
        let nanos = case
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let filename = format!("corpus_{}.json", nanos);
        let path = self.corpus_dir.join(filename);

        // Serialize and write
        let json = serde_json::to_string_pretty(&case)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut file = File::create(&path)?;
        file.write_all(json.as_bytes())?;

        self.cached_cases.push(case);
        Ok(())
    }

    /// Load all corpus cases
    pub fn load_all(&mut self) -> io::Result<Vec<CorpusCase>> {
        self.cached_cases.clear();

        for entry in fs::read_dir(&self.corpus_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                if let Ok(case) = serde_json::from_str::<CorpusCase>(&contents) {
                    self.cached_cases.push(case);
                }
            }
        }

        Ok(self.cached_cases.clone())
    }

    /// Get all cached corpus cases
    pub fn cases(&self) -> &[CorpusCase] {
        &self.cached_cases
    }
}

/// Configuration for test persistence
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Whether to persist failures
    pub persist_failures: bool,

    /// Directory for failure database
    pub failure_dir: PathBuf,

    /// Whether to use corpus for test generation
    pub use_corpus: bool,

    /// Directory for test corpus
    pub corpus_dir: Option<PathBuf>,

    /// Whether to replay saved failures first
    pub replay_failures: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            persist_failures: false,
            failure_dir: PathBuf::from(".protest/failures"),
            use_corpus: false,
            corpus_dir: None,
            replay_failures: true,
        }
    }
}

impl PersistenceConfig {
    /// Create a new persistence configuration with all features enabled
    pub fn enabled() -> Self {
        Self {
            persist_failures: true,
            failure_dir: PathBuf::from(".protest/failures"),
            use_corpus: true,
            corpus_dir: Some(PathBuf::from(".protest/corpus")),
            replay_failures: true,
        }
    }

    /// Set the failure directory
    pub fn with_failure_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.failure_dir = dir.into();
        self
    }

    /// Set the corpus directory
    pub fn with_corpus_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.corpus_dir = Some(dir.into());
        self
    }

    /// Enable failure persistence
    pub fn enable_persistence(mut self) -> Self {
        self.persist_failures = true;
        self
    }

    /// Enable corpus usage
    pub fn enable_corpus(mut self) -> Self {
        self.use_corpus = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_failure_snapshot_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = FailureSnapshot::new(temp_dir.path()).unwrap();

        let failure =
            FailureCase::new(12345, "test input".to_string(), "test error".to_string(), 5);

        snapshot.save_failure("test_function", &failure).unwrap();

        let loaded = snapshot.load_failures("test_function").unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].seed, 12345);
        assert_eq!(loaded[0].input, "test input");
    }

    #[test]
    fn test_failure_snapshot_delete() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = FailureSnapshot::new(temp_dir.path()).unwrap();

        let failure =
            FailureCase::new(12345, "test input".to_string(), "test error".to_string(), 5);

        snapshot.save_failure("test_function", &failure).unwrap();
        snapshot.delete_failure("test_function", 12345).unwrap();

        let loaded = snapshot.load_failures("test_function").unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_corpus_add_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut corpus = TestCorpus::new(temp_dir.path()).unwrap();

        corpus
            .add_case(
                "interesting input".to_string(),
                "found edge case".to_string(),
            )
            .unwrap();

        let cases = corpus.load_all().unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].input, "interesting input");
    }

    #[test]
    fn test_list_tests_with_failures() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = FailureSnapshot::new(temp_dir.path()).unwrap();

        let failure =
            FailureCase::new(12345, "test input".to_string(), "test error".to_string(), 5);

        snapshot.save_failure("test1", &failure).unwrap();
        snapshot.save_failure("test2", &failure).unwrap();

        let tests = snapshot.list_tests_with_failures().unwrap();
        assert_eq!(tests.len(), 2);
        assert!(tests.contains(&"test1".to_string()));
        assert!(tests.contains(&"test2".to_string()));
    }
}

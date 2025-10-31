//! Configuration types for controlling test behavior and generation parameters.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// Invalid number of iterations (must be > 0)
    InvalidIterations(usize),
    /// Invalid number of shrink iterations (must be > 0)
    InvalidShrinkIterations(usize),
    /// Invalid timeout (must be > 0)
    InvalidTimeout,
    /// Invalid max depth (must be > 0)
    InvalidMaxDepth(usize),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidIterations(n) => {
                write!(f, "Invalid iterations count: {} (must be > 0)", n)
            }
            ConfigError::InvalidShrinkIterations(n) => {
                write!(f, "Invalid shrink iterations count: {} (must be > 0)", n)
            }
            ConfigError::InvalidTimeout => {
                write!(f, "Invalid timeout (must be > 0)")
            }
            ConfigError::InvalidMaxDepth(n) => {
                write!(f, "Invalid max depth: {} (must be > 0)", n)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration for generators
#[derive(Debug)]
pub struct GeneratorConfig {
    /// Hint for the size of generated collections
    pub size_hint: usize,
    /// Maximum depth for nested structures
    pub max_depth: usize,
    /// Custom ranges and constraints for specific types
    pub custom_ranges: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Clone for GeneratorConfig {
    fn clone(&self) -> Self {
        Self {
            size_hint: self.size_hint,
            max_depth: self.max_depth,
            // We can't clone the custom_ranges, so we create a new empty HashMap
            // This is acceptable for now as custom ranges will be handled differently
            custom_ranges: HashMap::new(),
        }
    }
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            size_hint: 10,
            max_depth: 5,
            custom_ranges: HashMap::new(),
        }
    }
}

impl GeneratorConfig {
    /// Create a new generator configuration with validation
    pub fn new(
        size_hint: usize,
        max_depth: usize,
        custom_ranges: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    ) -> Result<Self, ConfigError> {
        if max_depth == 0 {
            return Err(ConfigError::InvalidMaxDepth(max_depth));
        }

        Ok(Self {
            size_hint,
            max_depth,
            custom_ranges,
        })
    }

    /// Validate the generator configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_depth == 0 {
            return Err(ConfigError::InvalidMaxDepth(self.max_depth));
        }
        Ok(())
    }

    /// Merge this configuration with another, with this config taking precedence for non-default values
    pub fn merge_with(self, _other: &GeneratorConfig) -> Self {
        // For now, we can't clone custom_ranges due to trait object limitations
        // In a real implementation, we'd need a different approach for custom ranges
        Self {
            size_hint: if self.size_hint != 10 {
                self.size_hint
            } else {
                _other.size_hint
            },
            max_depth: if self.max_depth != 5 {
                self.max_depth
            } else {
                _other.max_depth
            },
            custom_ranges: self.custom_ranges, // Always use self's custom ranges
        }
    }
}

/// Configuration for individual property tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Number of test iterations to run
    pub iterations: usize,
    /// Maximum number of shrinking iterations
    pub max_shrink_iterations: usize,
    /// Timeout for shrinking process
    pub shrink_timeout: Duration,
    /// Optional seed for reproducible tests
    pub seed: Option<u64>,
    /// Generator configuration overrides
    pub generator_config: GeneratorConfig,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            max_shrink_iterations: 1000,
            shrink_timeout: Duration::from_secs(10),
            seed: None,
            generator_config: GeneratorConfig::default(),
        }
    }
}

impl TestConfig {
    /// Create a new test configuration with validation
    pub fn new(
        iterations: usize,
        max_shrink_iterations: usize,
        shrink_timeout: Duration,
        seed: Option<u64>,
        generator_config: GeneratorConfig,
    ) -> Result<Self, ConfigError> {
        if iterations == 0 {
            return Err(ConfigError::InvalidIterations(iterations));
        }
        if max_shrink_iterations == 0 {
            return Err(ConfigError::InvalidShrinkIterations(max_shrink_iterations));
        }
        if shrink_timeout.is_zero() {
            return Err(ConfigError::InvalidTimeout);
        }

        Ok(Self {
            iterations,
            max_shrink_iterations,
            shrink_timeout,
            seed,
            generator_config,
        })
    }

    /// Validate the test configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.iterations == 0 {
            return Err(ConfigError::InvalidIterations(self.iterations));
        }
        if self.max_shrink_iterations == 0 {
            return Err(ConfigError::InvalidShrinkIterations(
                self.max_shrink_iterations,
            ));
        }
        if self.shrink_timeout.is_zero() {
            return Err(ConfigError::InvalidTimeout);
        }
        self.generator_config.validate()?;
        Ok(())
    }

    /// Merge this configuration with a global configuration, with this config taking precedence
    pub fn merge_with_global(self, global: &GlobalConfig) -> Self {
        Self {
            iterations: self.iterations,
            max_shrink_iterations: self.max_shrink_iterations,
            shrink_timeout: self.shrink_timeout,
            seed: self.seed.or(global.default_seed),
            generator_config: self.generator_config.merge_with(&global.generator_config),
        }
    }

    /// Create a test configuration from global defaults with optional overrides
    pub fn from_global_with_overrides(
        global: &GlobalConfig,
        iterations: Option<usize>,
        seed: Option<u64>,
        generator_overrides: Option<GeneratorConfig>,
    ) -> Result<Self, ConfigError> {
        let config = Self {
            iterations: iterations.unwrap_or(global.default_iterations),
            max_shrink_iterations: 1000,             // Default value
            shrink_timeout: Duration::from_secs(10), // Default value
            seed: seed.or(global.default_seed),
            generator_config: generator_overrides
                .unwrap_or_else(|| global.generator_config.clone()),
        };
        config.validate()?;
        Ok(config)
    }
}

/// Global configuration for default test behavior
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    /// Default number of iterations for tests
    pub default_iterations: usize,
    /// Default seed for reproducible tests
    pub default_seed: Option<u64>,
    /// Default generator configuration
    pub generator_config: GeneratorConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_iterations: 100,
            default_seed: None,
            generator_config: GeneratorConfig::default(),
        }
    }
}

impl GlobalConfig {
    /// Create a new global configuration with validation
    pub fn new(
        default_iterations: usize,
        default_seed: Option<u64>,
        generator_config: GeneratorConfig,
    ) -> Result<Self, ConfigError> {
        if default_iterations == 0 {
            return Err(ConfigError::InvalidIterations(default_iterations));
        }

        Ok(Self {
            default_iterations,
            default_seed,
            generator_config,
        })
    }

    /// Validate the global configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.default_iterations == 0 {
            return Err(ConfigError::InvalidIterations(self.default_iterations));
        }
        self.generator_config.validate()?;
        Ok(())
    }
}

/// Statistics about value generation
#[derive(Debug, Default)]
pub struct GenerationStats {
    /// Total values generated
    pub total_generated: usize,
    /// Distribution information (type-specific)
    pub distribution_info: HashMap<String, Box<dyn Any + Send + Sync>>,
    /// Coverage information for generated value ranges
    pub coverage_info: CoverageInfo,
    /// Performance metrics for generation
    pub performance_metrics: GenerationPerformanceMetrics,
}

/// Coverage information for tracking generated value ranges and distributions
#[derive(Debug, Default)]
pub struct CoverageInfo {
    /// Coverage for numeric types (min, max, count per range)
    pub numeric_coverage: HashMap<String, NumericCoverage>,
    /// Coverage for string types (length distribution, character sets)
    pub string_coverage: HashMap<String, StringCoverage>,
    /// Coverage for collection types (size distribution)
    pub collection_coverage: HashMap<String, CollectionCoverage>,
    /// Coverage for boolean values
    pub boolean_coverage: HashMap<String, BooleanCoverage>,
    /// Coverage for enum variants
    pub enum_coverage: HashMap<String, EnumCoverage>,
    /// Custom coverage for user-defined types
    pub custom_coverage: HashMap<String, Box<dyn CustomCoverage + Send + Sync>>,
}

/// Coverage tracking for numeric types
#[derive(Debug, Clone)]
pub struct NumericCoverage {
    /// Minimum value generated
    pub min_value: f64,
    /// Maximum value generated
    pub max_value: f64,
    /// Total count of values generated
    pub total_count: usize,
    /// Distribution across ranges (range_start -> count)
    pub range_distribution: HashMap<String, usize>,
    /// Statistical moments (mean, variance, etc.)
    pub statistics: NumericStatistics,
}

/// Statistical information for numeric values
#[derive(Debug, Clone)]
pub struct NumericStatistics {
    /// Mean of generated values
    pub mean: f64,
    /// Variance of generated values
    pub variance: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Skewness (measure of asymmetry)
    pub skewness: f64,
    /// Kurtosis (measure of tail heaviness)
    pub kurtosis: f64,
}

/// Coverage tracking for string types
#[derive(Debug, Clone)]
pub struct StringCoverage {
    /// Length distribution (length -> count)
    pub length_distribution: HashMap<usize, usize>,
    /// Character set coverage (character -> count)
    pub character_distribution: HashMap<char, usize>,
    /// Pattern coverage (regex patterns matched)
    pub pattern_coverage: HashMap<String, usize>,
    /// Total strings generated
    pub total_count: usize,
    /// Average string length
    pub average_length: f64,
}

/// Coverage tracking for collection types
#[derive(Debug, Clone)]
pub struct CollectionCoverage {
    /// Size distribution (size -> count)
    pub size_distribution: HashMap<usize, usize>,
    /// Total collections generated
    pub total_count: usize,
    /// Average collection size
    pub average_size: f64,
    /// Maximum size generated
    pub max_size: usize,
    /// Minimum size generated
    pub min_size: usize,
}

/// Coverage tracking for boolean values
#[derive(Debug, Clone)]
pub struct BooleanCoverage {
    /// Count of true values
    pub true_count: usize,
    /// Count of false values
    pub false_count: usize,
    /// Total boolean values generated
    pub total_count: usize,
    /// Ratio of true to total
    pub true_ratio: f64,
}

/// Coverage tracking for enum variants
#[derive(Debug, Clone)]
pub struct EnumCoverage {
    /// Variant distribution (variant_name -> count)
    pub variant_distribution: HashMap<String, usize>,
    /// Total enum values generated
    pub total_count: usize,
    /// Coverage percentage (variants_covered / total_variants)
    pub coverage_percentage: f64,
}

/// Trait for custom coverage tracking
pub trait CustomCoverage: std::fmt::Debug {
    /// Record a generated value
    fn record_value(&mut self, value: &dyn std::any::Any);
    /// Get coverage summary
    fn get_summary(&self) -> String;
    /// Get coverage percentage (0.0 to 1.0)
    fn get_coverage_percentage(&self) -> f64;
}

/// Performance metrics for generation process
#[derive(Debug, Clone)]
pub struct GenerationPerformanceMetrics {
    /// Total time spent generating values
    pub total_generation_time: std::time::Duration,
    /// Average time per generation
    pub average_generation_time: std::time::Duration,
    /// Fastest generation time
    pub fastest_generation: std::time::Duration,
    /// Slowest generation time
    pub slowest_generation: std::time::Duration,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
}

/// Memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Peak memory usage during generation
    pub peak_memory_usage: usize,
    /// Average memory usage
    pub average_memory_usage: usize,
    /// Total allocations
    pub total_allocations: usize,
}

impl Clone for CoverageInfo {
    fn clone(&self) -> Self {
        Self {
            numeric_coverage: self.numeric_coverage.clone(),
            string_coverage: self.string_coverage.clone(),
            collection_coverage: self.collection_coverage.clone(),
            boolean_coverage: self.boolean_coverage.clone(),
            enum_coverage: self.enum_coverage.clone(),
            // We can't clone custom coverage, so we create a new empty HashMap
            custom_coverage: HashMap::new(),
        }
    }
}

impl Default for GenerationPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_generation_time: std::time::Duration::from_secs(0),
            average_generation_time: std::time::Duration::from_secs(0),
            fastest_generation: std::time::Duration::from_secs(u64::MAX),
            slowest_generation: std::time::Duration::from_secs(0),
            memory_stats: MemoryStats::default(),
        }
    }
}

impl Default for NumericCoverage {
    fn default() -> Self {
        Self::new()
    }
}

impl NumericCoverage {
    /// Create a new numeric coverage tracker
    pub fn new() -> Self {
        Self {
            min_value: f64::INFINITY,
            max_value: f64::NEG_INFINITY,
            total_count: 0,
            range_distribution: HashMap::new(),
            statistics: NumericStatistics::new(),
        }
    }

    /// Record a numeric value
    pub fn record_value(&mut self, value: f64) {
        self.min_value = self.min_value.min(value);
        self.max_value = self.max_value.max(value);
        self.total_count += 1;

        // Update range distribution (using simple bucketing)
        let bucket = self.get_bucket_for_value(value);
        *self.range_distribution.entry(bucket).or_insert(0) += 1;

        // Update statistics
        self.statistics.update(value, self.total_count);
    }

    /// Get bucket name for a value (simple implementation)
    fn get_bucket_for_value(&self, value: f64) -> String {
        let bucket_size = 10.0;
        let bucket_index = (value / bucket_size).floor() as i64;
        format!(
            "[{}, {})",
            bucket_index as f64 * bucket_size,
            (bucket_index + 1) as f64 * bucket_size
        )
    }

    /// Get coverage percentage for a specific range
    pub fn get_range_coverage(&self, min: f64, max: f64) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let values_in_range = self
            .range_distribution
            .iter()
            .filter(|(range, _)| {
                // Simple range parsing - in practice, this would be more robust
                range.contains(&min.to_string()) || range.contains(&max.to_string())
            })
            .map(|(_, count)| *count)
            .sum::<usize>();

        values_in_range as f64 / self.total_count as f64
    }
}

impl Default for NumericStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl NumericStatistics {
    /// Create new numeric statistics
    pub fn new() -> Self {
        Self {
            mean: 0.0,
            variance: 0.0,
            std_dev: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
        }
    }

    /// Update statistics with a new value (online algorithm)
    pub fn update(&mut self, value: f64, count: usize) {
        if count == 1 {
            self.mean = value;
            self.variance = 0.0;
            self.std_dev = 0.0;
            return;
        }

        // Online mean and variance calculation (Welford's algorithm)
        let delta = value - self.mean;
        self.mean += delta / count as f64;
        let delta2 = value - self.mean;
        self.variance += delta * delta2;

        if count > 1 {
            self.variance /= (count - 1) as f64;
            self.std_dev = self.variance.sqrt();
        }

        // Simplified skewness and kurtosis calculation
        // In practice, these would use more sophisticated online algorithms
        self.skewness = 0.0; // Placeholder
        self.kurtosis = 0.0; // Placeholder
    }
}

impl Default for StringCoverage {
    fn default() -> Self {
        Self::new()
    }
}

impl StringCoverage {
    /// Create a new string coverage tracker
    pub fn new() -> Self {
        Self {
            length_distribution: HashMap::new(),
            character_distribution: HashMap::new(),
            pattern_coverage: HashMap::new(),
            total_count: 0,
            average_length: 0.0,
        }
    }

    /// Record a string value
    pub fn record_value(&mut self, value: &str) {
        self.total_count += 1;

        // Update length distribution
        *self.length_distribution.entry(value.len()).or_insert(0) += 1;

        // Update character distribution
        for ch in value.chars() {
            *self.character_distribution.entry(ch).or_insert(0) += 1;
        }

        // Update average length
        self.average_length = (self.average_length * (self.total_count - 1) as f64
            + value.len() as f64)
            / self.total_count as f64;

        // Check for common patterns
        self.check_patterns(value);
    }

    /// Check for common string patterns
    fn check_patterns(&mut self, value: &str) {
        if value.chars().all(|c| c.is_ascii_alphabetic()) {
            *self
                .pattern_coverage
                .entry("alphabetic".to_string())
                .or_insert(0) += 1;
        }
        if value.chars().all(|c| c.is_ascii_digit()) {
            *self
                .pattern_coverage
                .entry("numeric".to_string())
                .or_insert(0) += 1;
        }
        if value.chars().all(|c| c.is_ascii_alphanumeric()) {
            *self
                .pattern_coverage
                .entry("alphanumeric".to_string())
                .or_insert(0) += 1;
        }
        if value.contains(' ') {
            *self
                .pattern_coverage
                .entry("contains_space".to_string())
                .or_insert(0) += 1;
        }
    }

    /// Get character set coverage percentage
    pub fn get_character_set_coverage(&self, expected_chars: &[char]) -> f64 {
        if expected_chars.is_empty() {
            return 1.0;
        }

        let covered_chars = expected_chars
            .iter()
            .filter(|ch| self.character_distribution.contains_key(ch))
            .count();

        covered_chars as f64 / expected_chars.len() as f64
    }
}

impl Default for CollectionCoverage {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectionCoverage {
    /// Create a new collection coverage tracker
    pub fn new() -> Self {
        Self {
            size_distribution: HashMap::new(),
            total_count: 0,
            average_size: 0.0,
            max_size: 0,
            min_size: usize::MAX,
        }
    }

    /// Record a collection size
    pub fn record_size(&mut self, size: usize) {
        self.total_count += 1;

        // Update size distribution
        *self.size_distribution.entry(size).or_insert(0) += 1;

        // Update min/max
        self.min_size = self.min_size.min(size);
        self.max_size = self.max_size.max(size);

        // Update average size
        self.average_size = (self.average_size * (self.total_count - 1) as f64 + size as f64)
            / self.total_count as f64;
    }

    /// Get size range coverage
    pub fn get_size_range_coverage(&self, min_size: usize, max_size: usize) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let values_in_range = self
            .size_distribution
            .iter()
            .filter(|(size, _)| **size >= min_size && **size <= max_size)
            .map(|(_, count)| *count)
            .sum::<usize>();

        values_in_range as f64 / self.total_count as f64
    }
}

impl Default for BooleanCoverage {
    fn default() -> Self {
        Self::new()
    }
}

impl BooleanCoverage {
    /// Create a new boolean coverage tracker
    pub fn new() -> Self {
        Self {
            true_count: 0,
            false_count: 0,
            total_count: 0,
            true_ratio: 0.0,
        }
    }

    /// Record a boolean value
    pub fn record_value(&mut self, value: bool) {
        self.total_count += 1;

        if value {
            self.true_count += 1;
        } else {
            self.false_count += 1;
        }

        self.true_ratio = self.true_count as f64 / self.total_count as f64;
    }

    /// Check if both true and false values have been generated
    pub fn has_full_coverage(&self) -> bool {
        self.true_count > 0 && self.false_count > 0
    }
}

impl EnumCoverage {
    /// Create a new enum coverage tracker
    pub fn new(_total_variants: usize) -> Self {
        Self {
            variant_distribution: HashMap::new(),
            total_count: 0,
            coverage_percentage: 0.0,
        }
    }

    /// Record an enum variant
    pub fn record_variant(&mut self, variant_name: &str, total_variants: usize) {
        self.total_count += 1;
        *self
            .variant_distribution
            .entry(variant_name.to_string())
            .or_insert(0) += 1;

        // Update coverage percentage
        let covered_variants = self.variant_distribution.len();
        self.coverage_percentage = covered_variants as f64 / total_variants as f64;
    }

    /// Get the least covered variant
    pub fn get_least_covered_variant(&self) -> Option<(&String, &usize)> {
        self.variant_distribution
            .iter()
            .min_by_key(|(_, count)| *count)
    }

    /// Get the most covered variant
    pub fn get_most_covered_variant(&self) -> Option<(&String, &usize)> {
        self.variant_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
    }
}

impl GenerationStats {
    /// Create a comprehensive report of generation statistics
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("                    GENERATION STATISTICS                     \n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");

        // Basic statistics
        report.push_str("📊 BASIC STATISTICS:\n");
        report.push_str(&format!(
            "   Total values generated: {}\n",
            self.total_generated
        ));
        report.push_str(&format!(
            "   Generation time: {:?}\n",
            self.performance_metrics.total_generation_time
        ));
        report.push_str(&format!(
            "   Average time per value: {:?}\n",
            self.performance_metrics.average_generation_time
        ));
        report.push('\n');

        // Coverage information
        report.push_str("📈 COVERAGE INFORMATION:\n");

        // Numeric coverage
        if !self.coverage_info.numeric_coverage.is_empty() {
            report.push_str("   Numeric Types:\n");
            for (type_name, coverage) in &self.coverage_info.numeric_coverage {
                report.push_str(&format!(
                    "     {}: {} values, range [{:.2}, {:.2}], mean: {:.2}\n",
                    type_name,
                    coverage.total_count,
                    coverage.min_value,
                    coverage.max_value,
                    coverage.statistics.mean
                ));
            }
        }

        // String coverage
        if !self.coverage_info.string_coverage.is_empty() {
            report.push_str("   String Types:\n");
            for (type_name, coverage) in &self.coverage_info.string_coverage {
                report.push_str(&format!(
                    "     {}: {} strings, avg length: {:.1}, {} unique chars\n",
                    type_name,
                    coverage.total_count,
                    coverage.average_length,
                    coverage.character_distribution.len()
                ));
            }
        }

        // Collection coverage
        if !self.coverage_info.collection_coverage.is_empty() {
            report.push_str("   Collection Types:\n");
            for (type_name, coverage) in &self.coverage_info.collection_coverage {
                report.push_str(&format!(
                    "     {}: {} collections, avg size: {:.1}, range [{}, {}]\n",
                    type_name,
                    coverage.total_count,
                    coverage.average_size,
                    coverage.min_size,
                    coverage.max_size
                ));
            }
        }

        // Boolean coverage
        if !self.coverage_info.boolean_coverage.is_empty() {
            report.push_str("   Boolean Types:\n");
            for (type_name, coverage) in &self.coverage_info.boolean_coverage {
                let coverage_status = if coverage.has_full_coverage() {
                    "✓"
                } else {
                    "✗"
                };
                report.push_str(&format!(
                    "     {}: {} values, true ratio: {:.2} {}\n",
                    type_name, coverage.total_count, coverage.true_ratio, coverage_status
                ));
            }
        }

        // Enum coverage
        if !self.coverage_info.enum_coverage.is_empty() {
            report.push_str("   Enum Types:\n");
            for (type_name, coverage) in &self.coverage_info.enum_coverage {
                report.push_str(&format!(
                    "     {}: {} values, {:.1}% variant coverage\n",
                    type_name,
                    coverage.total_count,
                    coverage.coverage_percentage * 100.0
                ));
            }
        }

        report.push('\n');

        // Performance metrics
        report.push_str("⚡ PERFORMANCE METRICS:\n");
        report.push_str(&format!(
            "   Fastest generation: {:?}\n",
            self.performance_metrics.fastest_generation
        ));
        report.push_str(&format!(
            "   Slowest generation: {:?}\n",
            self.performance_metrics.slowest_generation
        ));
        report.push_str(&format!(
            "   Peak memory usage: {} bytes\n",
            self.performance_metrics.memory_stats.peak_memory_usage
        ));
        report.push('\n');

        report.push_str("═══════════════════════════════════════════════════════════════\n");

        report
    }

    /// Get a concise summary of the statistics
    pub fn get_summary(&self) -> String {
        format!(
            "Generated {} values in {:?} (avg: {:?}/value)",
            self.total_generated,
            self.performance_metrics.total_generation_time,
            self.performance_metrics.average_generation_time
        )
    }

    /// Check if coverage meets specified thresholds
    pub fn check_coverage_thresholds(&self, thresholds: &CoverageThresholds) -> CoverageReport {
        let mut report = CoverageReport::new();

        // Check numeric coverage
        for (type_name, coverage) in &self.coverage_info.numeric_coverage {
            if let Some(threshold) = thresholds.numeric_thresholds.get(type_name) {
                let range_coverage =
                    coverage.get_range_coverage(threshold.min_value, threshold.max_value);
                report.add_numeric_result(
                    type_name.clone(),
                    range_coverage >= threshold.min_coverage,
                );
            }
        }

        // Check boolean coverage
        for (type_name, coverage) in &self.coverage_info.boolean_coverage {
            if thresholds.require_full_boolean_coverage {
                report.add_boolean_result(type_name.clone(), coverage.has_full_coverage());
            }
        }

        // Check enum coverage
        for (type_name, coverage) in &self.coverage_info.enum_coverage {
            if let Some(threshold) = thresholds.enum_thresholds.get(type_name) {
                report.add_enum_result(
                    type_name.clone(),
                    coverage.coverage_percentage >= threshold.min_coverage,
                );
            }
        }

        report
    }
}

/// Thresholds for coverage checking
#[derive(Debug, Clone)]
pub struct CoverageThresholds {
    /// Thresholds for numeric types
    pub numeric_thresholds: HashMap<String, NumericThreshold>,
    /// Whether to require full boolean coverage (both true and false)
    pub require_full_boolean_coverage: bool,
    /// Thresholds for enum types
    pub enum_thresholds: HashMap<String, EnumThreshold>,
}

/// Threshold for numeric type coverage
#[derive(Debug, Clone)]
pub struct NumericThreshold {
    /// Minimum value that should be covered
    pub min_value: f64,
    /// Maximum value that should be covered
    pub max_value: f64,
    /// Minimum coverage percentage required
    pub min_coverage: f64,
}

/// Threshold for enum type coverage
#[derive(Debug, Clone)]
pub struct EnumThreshold {
    /// Minimum coverage percentage required
    pub min_coverage: f64,
}

/// Report of coverage threshold checking
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Results for numeric types
    pub numeric_results: HashMap<String, bool>,
    /// Results for boolean types
    pub boolean_results: HashMap<String, bool>,
    /// Results for enum types
    pub enum_results: HashMap<String, bool>,
    /// Overall pass/fail status
    pub overall_pass: bool,
}

impl Default for CoverageReport {
    fn default() -> Self {
        Self::new()
    }
}

impl CoverageReport {
    /// Create a new coverage report
    pub fn new() -> Self {
        Self {
            numeric_results: HashMap::new(),
            boolean_results: HashMap::new(),
            enum_results: HashMap::new(),
            overall_pass: true,
        }
    }

    /// Add a numeric coverage result
    pub fn add_numeric_result(&mut self, type_name: String, passed: bool) {
        self.numeric_results.insert(type_name, passed);
        if !passed {
            self.overall_pass = false;
        }
    }

    /// Add a boolean coverage result
    pub fn add_boolean_result(&mut self, type_name: String, passed: bool) {
        self.boolean_results.insert(type_name, passed);
        if !passed {
            self.overall_pass = false;
        }
    }

    /// Add an enum coverage result
    pub fn add_enum_result(&mut self, type_name: String, passed: bool) {
        self.enum_results.insert(type_name, passed);
        if !passed {
            self.overall_pass = false;
        }
    }

    /// Generate a report string
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("Coverage Threshold Report:\n");
        report.push_str(&format!(
            "Overall Status: {}\n",
            if self.overall_pass { "PASS" } else { "FAIL" }
        ));

        if !self.numeric_results.is_empty() {
            report.push_str("\nNumeric Coverage:\n");
            for (type_name, passed) in &self.numeric_results {
                let status = if *passed { "✓" } else { "✗" };
                report.push_str(&format!("  {} {}\n", status, type_name));
            }
        }

        if !self.boolean_results.is_empty() {
            report.push_str("\nBoolean Coverage:\n");
            for (type_name, passed) in &self.boolean_results {
                let status = if *passed { "✓" } else { "✗" };
                report.push_str(&format!("  {} {}\n", status, type_name));
            }
        }

        if !self.enum_results.is_empty() {
            report.push_str("\nEnum Coverage:\n");
            for (type_name, passed) in &self.enum_results {
                let status = if *passed { "✓" } else { "✗" };
                report.push_str(&format!("  {} {}\n", status, type_name));
            }
        }

        report
    }
}

/// Global configuration manager for hierarchical configuration
pub struct ConfigManager {
    global_config: GlobalConfig,
}

impl ConfigManager {
    /// Create a new configuration manager with default global configuration
    pub fn new() -> Self {
        Self {
            global_config: GlobalConfig::default(),
        }
    }

    /// Create a new configuration manager with custom global configuration
    pub fn with_global_config(global_config: GlobalConfig) -> Result<Self, ConfigError> {
        global_config.validate()?;
        Ok(Self { global_config })
    }

    /// Get the current global configuration
    pub fn global_config(&self) -> &GlobalConfig {
        &self.global_config
    }

    /// Update the global configuration
    pub fn set_global_config(&mut self, global_config: GlobalConfig) -> Result<(), ConfigError> {
        global_config.validate()?;
        self.global_config = global_config;
        Ok(())
    }

    /// Create a test configuration that inherits from global defaults
    pub fn create_test_config(&self) -> TestConfig {
        TestConfig::from_global_with_overrides(&self.global_config, None, None, None)
            .unwrap_or_else(|_| TestConfig::default())
    }

    /// Create a test configuration with specific overrides
    pub fn create_test_config_with_overrides(
        &self,
        iterations: Option<usize>,
        seed: Option<u64>,
        generator_overrides: Option<GeneratorConfig>,
    ) -> Result<TestConfig, ConfigError> {
        TestConfig::from_global_with_overrides(
            &self.global_config,
            iterations,
            seed,
            generator_overrides,
        )
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local global configuration manager (doc comment not allowed on thread_local!)
thread_local! {
    static CONFIG_MANAGER: std::cell::RefCell<ConfigManager> = std::cell::RefCell::new(ConfigManager::new());
}

/// Get the current global configuration
pub fn get_global_config() -> GlobalConfig {
    CONFIG_MANAGER.with(|manager| manager.borrow().global_config().clone())
}

/// Set the global configuration
pub fn set_global_config(config: GlobalConfig) -> Result<(), ConfigError> {
    config.validate()?;
    CONFIG_MANAGER.with(|manager| manager.borrow_mut().set_global_config(config))
}

/// Create a test configuration that inherits from global defaults
pub fn create_test_config() -> TestConfig {
    CONFIG_MANAGER.with(|manager| manager.borrow().create_test_config())
}

/// Create a test configuration with specific overrides
pub fn create_test_config_with_overrides(
    iterations: Option<usize>,
    seed: Option<u64>,
    generator_overrides: Option<GeneratorConfig>,
) -> Result<TestConfig, ConfigError> {
    CONFIG_MANAGER.with(|manager| {
        manager
            .borrow()
            .create_test_config_with_overrides(iterations, seed, generator_overrides)
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_config_validation() {
        // Valid configuration
        let config = GeneratorConfig::new(10, 5, HashMap::new());
        assert!(config.is_ok());

        // Invalid max_depth
        let config = GeneratorConfig::new(10, 0, HashMap::new());
        assert!(matches!(config, Err(ConfigError::InvalidMaxDepth(0))));

        // Test validate method
        let mut config = GeneratorConfig::default();
        assert!(config.validate().is_ok());

        config.max_depth = 0;
        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidMaxDepth(0))
        ));
    }

    #[test]
    fn test_test_config_validation() {
        // Valid configuration
        let config = TestConfig::new(
            100,
            1000,
            Duration::from_secs(10),
            None,
            GeneratorConfig::default(),
        );
        assert!(config.is_ok());

        // Invalid iterations
        let config = TestConfig::new(
            0,
            1000,
            Duration::from_secs(10),
            None,
            GeneratorConfig::default(),
        );
        assert!(matches!(config, Err(ConfigError::InvalidIterations(0))));

        // Invalid shrink iterations
        let config = TestConfig::new(
            100,
            0,
            Duration::from_secs(10),
            None,
            GeneratorConfig::default(),
        );
        assert!(matches!(
            config,
            Err(ConfigError::InvalidShrinkIterations(0))
        ));

        // Invalid timeout
        let config = TestConfig::new(
            100,
            1000,
            Duration::from_secs(0),
            None,
            GeneratorConfig::default(),
        );
        assert!(matches!(config, Err(ConfigError::InvalidTimeout)));
    }

    #[test]
    fn test_global_config_validation() {
        // Valid configuration
        let config = GlobalConfig::new(100, None, GeneratorConfig::default());
        assert!(config.is_ok());

        // Invalid iterations
        let config = GlobalConfig::new(0, None, GeneratorConfig::default());
        assert!(matches!(config, Err(ConfigError::InvalidIterations(0))));
    }

    #[test]
    fn test_config_merge_precedence() {
        let global = GlobalConfig {
            default_iterations: 50,
            default_seed: Some(123),
            generator_config: GeneratorConfig {
                size_hint: 20,
                max_depth: 3,
                custom_ranges: HashMap::new(),
            },
        };

        let test_config = TestConfig {
            iterations: 200,
            max_shrink_iterations: 500,
            shrink_timeout: Duration::from_secs(5),
            seed: Some(456),
            generator_config: GeneratorConfig {
                size_hint: 15,
                max_depth: 7,
                custom_ranges: HashMap::new(),
            },
        };

        let merged = test_config.merge_with_global(&global);

        // Test config values should take precedence
        assert_eq!(merged.iterations, 200);
        assert_eq!(merged.seed, Some(456));
        assert_eq!(merged.generator_config.size_hint, 15);
        assert_eq!(merged.generator_config.max_depth, 7);
    }

    #[test]
    fn test_config_merge_with_defaults() {
        let global = GlobalConfig {
            default_iterations: 50,
            default_seed: Some(123),
            generator_config: GeneratorConfig {
                size_hint: 20,
                max_depth: 3,
                custom_ranges: HashMap::new(),
            },
        };

        let test_config = TestConfig {
            iterations: 200,
            max_shrink_iterations: 500,
            shrink_timeout: Duration::from_secs(5),
            seed: None,                                   // Should inherit from global
            generator_config: GeneratorConfig::default(), // Should merge with global
        };

        let merged = test_config.merge_with_global(&global);

        // Should inherit seed from global
        assert_eq!(merged.seed, Some(123));
        // Should inherit generator config from global since test config uses defaults
        assert_eq!(merged.generator_config.size_hint, 20);
        assert_eq!(merged.generator_config.max_depth, 3);
    }

    #[test]
    fn test_from_global_with_overrides() {
        let global = GlobalConfig {
            default_iterations: 50,
            default_seed: Some(123),
            generator_config: GeneratorConfig {
                size_hint: 20,
                max_depth: 3,
                custom_ranges: HashMap::new(),
            },
        };

        // Test with no overrides
        let config = TestConfig::from_global_with_overrides(&global, None, None, None).unwrap();
        assert_eq!(config.iterations, 50);
        assert_eq!(config.seed, Some(123));

        // Test with iterations override
        let config =
            TestConfig::from_global_with_overrides(&global, Some(100), None, None).unwrap();
        assert_eq!(config.iterations, 100);
        assert_eq!(config.seed, Some(123));

        // Test with seed override
        let config =
            TestConfig::from_global_with_overrides(&global, None, Some(456), None).unwrap();
        assert_eq!(config.iterations, 50);
        assert_eq!(config.seed, Some(456));
    }

    #[test]
    fn test_config_manager() {
        let mut manager = ConfigManager::new();

        // Test default global config
        let global = manager.global_config();
        assert_eq!(global.default_iterations, 100);

        // Test creating test config from global
        let test_config = manager.create_test_config();
        assert_eq!(test_config.iterations, 100);

        // Test updating global config
        let new_global = GlobalConfig {
            default_iterations: 200,
            default_seed: Some(789),
            generator_config: GeneratorConfig::default(),
        };
        manager.set_global_config(new_global).unwrap();

        let updated_test_config = manager.create_test_config();
        assert_eq!(updated_test_config.iterations, 200);
        assert_eq!(updated_test_config.seed, Some(789));
    }

    #[test]
    fn test_config_manager_with_overrides() {
        let manager = ConfigManager::new();

        // Test creating test config with overrides
        let config = manager
            .create_test_config_with_overrides(Some(300), Some(999), None)
            .unwrap();
        assert_eq!(config.iterations, 300);
        assert_eq!(config.seed, Some(999));
    }

    #[test]
    fn test_thread_local_config_functions() {
        // Test getting default global config
        let global = get_global_config();
        assert_eq!(global.default_iterations, 100);

        // Test setting global config
        let new_global = GlobalConfig {
            default_iterations: 150,
            default_seed: Some(555),
            generator_config: GeneratorConfig::default(),
        };
        set_global_config(new_global).unwrap();

        let updated_global = get_global_config();
        assert_eq!(updated_global.default_iterations, 150);
        assert_eq!(updated_global.default_seed, Some(555));

        // Test creating test config
        let test_config = create_test_config();
        assert_eq!(test_config.iterations, 150);
        assert_eq!(test_config.seed, Some(555));

        // Test creating test config with overrides
        let config = create_test_config_with_overrides(Some(250), None, None).unwrap();
        assert_eq!(config.iterations, 250);
        assert_eq!(config.seed, Some(555)); // Should inherit from global
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::InvalidIterations(0);
        assert_eq!(
            format!("{}", error),
            "Invalid iterations count: 0 (must be > 0)"
        );

        let error = ConfigError::InvalidShrinkIterations(0);
        assert_eq!(
            format!("{}", error),
            "Invalid shrink iterations count: 0 (must be > 0)"
        );

        let error = ConfigError::InvalidTimeout;
        assert_eq!(format!("{}", error), "Invalid timeout (must be > 0)");

        let error = ConfigError::InvalidMaxDepth(0);
        assert_eq!(format!("{}", error), "Invalid max depth: 0 (must be > 0)");
    }

    #[test]
    fn test_generator_config_merge() {
        let base = GeneratorConfig {
            size_hint: 20,
            max_depth: 3,
            custom_ranges: HashMap::new(),
        };

        // Test merging with default values (should use base values)
        let default_config = GeneratorConfig::default();
        let merged = default_config.merge_with(&base);
        assert_eq!(merged.size_hint, 20); // From base
        assert_eq!(merged.max_depth, 3); // From base

        // Test merging with non-default values (should use override values)
        let override_config = GeneratorConfig {
            size_hint: 15,
            max_depth: 7,
            custom_ranges: HashMap::new(),
        };
        let merged = override_config.merge_with(&base);
        assert_eq!(merged.size_hint, 15); // From override
        assert_eq!(merged.max_depth, 7); // From override
    }
}

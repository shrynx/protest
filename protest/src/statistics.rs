//! Statistics collection and analysis for property-based testing.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::config::{
    CoverageReport, CoverageThresholds, CustomCoverage, EnumCoverage, GenerationStats, MemoryStats,
};

/// Statistics collector that tracks generation patterns and performance
pub struct StatisticsCollector {
    /// Current statistics being collected
    stats: GenerationStats,
    /// Whether statistics collection is enabled
    enabled: bool,
    /// Start time for performance tracking
    start_time: Option<Instant>,
    /// Individual generation times for analysis
    generation_times: Vec<Duration>,
    /// Memory tracking (simplified)
    memory_tracker: MemoryTracker,
}

/// Memory tracking helper (simplified implementation)
struct MemoryTracker {
    peak_usage: usize,
    current_usage: usize,
    allocations: usize,
}

impl StatisticsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        Self {
            stats: GenerationStats::default(),
            enabled: true,
            start_time: None,
            generation_times: Vec::new(),
            memory_tracker: MemoryTracker::new(),
        }
    }

    /// Create a disabled statistics collector (for performance)
    pub fn disabled() -> Self {
        Self {
            stats: GenerationStats::default(),
            enabled: false,
            start_time: None,
            generation_times: Vec::new(),
            memory_tracker: MemoryTracker::new(),
        }
    }

    /// Enable statistics collection
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable statistics collection
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if statistics collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start timing a generation operation
    pub fn start_generation_timing(&mut self) {
        if !self.enabled {
            return;
        }
        self.start_time = Some(Instant::now());
    }

    /// End timing a generation operation and record the duration
    pub fn end_generation_timing(&mut self) {
        if !self.enabled {
            return;
        }

        if let Some(start) = self.start_time.take() {
            let duration = start.elapsed();
            self.generation_times.push(duration);
            self.update_performance_metrics(duration);
        }
    }

    /// Record a generated value for statistics
    pub fn record_generated_value<T: Any + std::fmt::Debug>(&mut self, value: &T, type_name: &str) {
        if !self.enabled {
            return;
        }

        self.stats.total_generated += 1;

        // Record type-specific statistics
        self.record_type_specific_stats(value, type_name);

        // Update memory tracking - record at least the size of the value
        let size = std::mem::size_of_val(value).max(1); // Ensure at least 1 byte is recorded
        self.memory_tracker.record_allocation(size);

        // Update performance metrics with current memory stats
        self.stats.performance_metrics.memory_stats = self.memory_tracker.get_stats();
    }

    /// Record type-specific statistics based on the value type
    fn record_type_specific_stats<T: Any + std::fmt::Debug>(&mut self, value: &T, type_name: &str) {
        let _type_id = TypeId::of::<T>();
        let value_any: &dyn Any = value;

        // Handle numeric types
        if let Some(num_val) = self.try_extract_numeric(value_any) {
            let coverage = self
                .stats
                .coverage_info
                .numeric_coverage
                .entry(type_name.to_string())
                .or_default();
            coverage.record_value(num_val);
        }
        // Handle string types
        else if let Some(str_val) = value_any.downcast_ref::<String>() {
            let coverage = self
                .stats
                .coverage_info
                .string_coverage
                .entry(type_name.to_string())
                .or_default();
            coverage.record_value(str_val);
        } else if let Some(str_val) = value_any.downcast_ref::<&str>() {
            let coverage = self
                .stats
                .coverage_info
                .string_coverage
                .entry(type_name.to_string())
                .or_default();
            coverage.record_value(str_val);
        }
        // Handle boolean types
        else if let Some(bool_val) = value_any.downcast_ref::<bool>() {
            let coverage = self
                .stats
                .coverage_info
                .boolean_coverage
                .entry(type_name.to_string())
                .or_default();
            coverage.record_value(*bool_val);
        }
        // Handle Vec types (simplified)
        else if type_name.contains("Vec") {
            // For collections, we need to use a different approach since we can't easily get the size
            // This is a simplified implementation
            let coverage = self
                .stats
                .coverage_info
                .collection_coverage
                .entry(type_name.to_string())
                .or_default();
            // We'll estimate size based on debug representation length as a proxy
            let debug_str = format!("{:?}", value);
            let estimated_size = if debug_str == "[]" {
                0
            } else {
                debug_str.matches(',').count() + 1
            };
            coverage.record_size(estimated_size);
        }
    }

    /// Try to extract a numeric value from Any
    fn try_extract_numeric(&self, value: &dyn Any) -> Option<f64> {
        // Try different numeric types
        if let Some(val) = value.downcast_ref::<i8>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<i16>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<i32>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<i64>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<i128>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<isize>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<u8>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<u16>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<u32>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<u64>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<u128>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<usize>() {
            Some(*val as f64)
        } else if let Some(val) = value.downcast_ref::<f32>() {
            Some(*val as f64)
        } else {
            value.downcast_ref::<f64>().copied()
        }
    }

    /// Record an enum variant
    pub fn record_enum_variant(
        &mut self,
        type_name: &str,
        variant_name: &str,
        total_variants: usize,
    ) {
        if !self.enabled {
            return;
        }

        let coverage = self
            .stats
            .coverage_info
            .enum_coverage
            .entry(type_name.to_string())
            .or_insert_with(|| EnumCoverage::new(total_variants));
        coverage.record_variant(variant_name, total_variants);
    }

    /// Record a collection size directly
    pub fn record_collection_size(&mut self, type_name: &str, size: usize) {
        if !self.enabled {
            return;
        }

        let coverage = self
            .stats
            .coverage_info
            .collection_coverage
            .entry(type_name.to_string())
            .or_default();
        coverage.record_size(size);
    }

    /// Add custom coverage tracking
    pub fn add_custom_coverage(
        &mut self,
        type_name: String,
        coverage: Box<dyn CustomCoverage + Send + Sync>,
    ) {
        if !self.enabled {
            return;
        }

        self.stats
            .coverage_info
            .custom_coverage
            .insert(type_name, coverage);
    }

    /// Update performance metrics with a new generation time
    fn update_performance_metrics(&mut self, duration: Duration) {
        let metrics = &mut self.stats.performance_metrics;

        metrics.total_generation_time += duration;

        if !self.generation_times.is_empty() {
            metrics.average_generation_time =
                metrics.total_generation_time / self.generation_times.len() as u32;
        }

        if duration < metrics.fastest_generation {
            metrics.fastest_generation = duration;
        }

        if duration > metrics.slowest_generation {
            metrics.slowest_generation = duration;
        }

        // Update memory stats
        metrics.memory_stats = self.memory_tracker.get_stats();
    }

    /// Get the current statistics
    pub fn get_stats(&self) -> &GenerationStats {
        &self.stats
    }

    /// Get a mutable reference to the current statistics
    pub fn get_stats_mut(&mut self) -> &mut GenerationStats {
        &mut self.stats
    }

    /// Take ownership of the statistics (consuming the collector)
    pub fn into_stats(self) -> GenerationStats {
        self.stats
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.stats = GenerationStats::default();
        self.generation_times.clear();
        self.memory_tracker.reset();
    }

    /// Generate a detailed analysis report
    pub fn generate_analysis_report(&self) -> String {
        if !self.enabled {
            return "Statistics collection is disabled".to_string();
        }

        let mut report = String::new();

        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("                    GENERATION ANALYSIS                       \n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");

        // Basic statistics
        report.push_str(&self.stats.generate_report());

        // Additional analysis
        report.push_str("🔍 DETAILED ANALYSIS:\n");

        // Generation time distribution
        if !self.generation_times.is_empty() {
            report.push_str(&self.analyze_generation_times());
        }

        // Coverage analysis
        report.push_str(&self.analyze_coverage_patterns());

        // Performance insights
        report.push_str(&self.analyze_performance());

        report.push_str("═══════════════════════════════════════════════════════════════\n");

        report
    }

    /// Analyze generation time patterns
    fn analyze_generation_times(&self) -> String {
        let mut analysis = String::new();

        analysis.push_str("   Generation Time Distribution:\n");

        if self.generation_times.len() < 2 {
            analysis.push_str("     Insufficient data for analysis\n");
            return analysis;
        }

        // Calculate percentiles
        let mut sorted_times = self.generation_times.clone();
        sorted_times.sort();

        let p50 = sorted_times[sorted_times.len() / 2];
        let p90 = sorted_times[(sorted_times.len() * 9) / 10];
        let p99 = sorted_times[(sorted_times.len() * 99) / 100];

        analysis.push_str(&format!("     50th percentile: {:?}\n", p50));
        analysis.push_str(&format!("     90th percentile: {:?}\n", p90));
        analysis.push_str(&format!("     99th percentile: {:?}\n", p99));

        // Identify outliers
        let mean_time = self.stats.performance_metrics.average_generation_time;
        let outliers = self
            .generation_times
            .iter()
            .filter(|&&time| time > mean_time * 3)
            .count();

        if outliers > 0 {
            analysis.push_str(&format!(
                "     Outliers (>3x mean): {} ({:.1}%)\n",
                outliers,
                outliers as f64 / self.generation_times.len() as f64 * 100.0
            ));
        }

        analysis.push('\n');
        analysis
    }

    /// Analyze coverage patterns
    fn analyze_coverage_patterns(&self) -> String {
        let mut analysis = String::new();

        analysis.push_str("   Coverage Pattern Analysis:\n");

        // Analyze numeric coverage patterns
        for (type_name, coverage) in &self.stats.coverage_info.numeric_coverage {
            let range_size = coverage.max_value - coverage.min_value;
            let density = coverage.total_count as f64 / range_size.max(1.0);

            analysis.push_str(&format!(
                "     {}: range density {:.2} values/unit\n",
                type_name, density
            ));

            // Check for potential bias
            if coverage.statistics.std_dev > 0.0 {
                let cv = coverage.statistics.std_dev / coverage.statistics.mean.abs();
                if cv > 1.0 {
                    analysis.push_str(&format!("       ⚠️  High variability (CV: {:.2})\n", cv));
                }
            }
        }

        // Analyze boolean coverage
        for (type_name, coverage) in &self.stats.coverage_info.boolean_coverage {
            if !coverage.has_full_coverage() {
                analysis.push_str(&format!("     ⚠️  {} missing coverage: ", type_name));
                if coverage.true_count == 0 {
                    analysis.push_str("no true values\n");
                } else {
                    analysis.push_str("no false values\n");
                }
            } else {
                let bias = (coverage.true_ratio - 0.5).abs();
                if bias > 0.2 {
                    analysis.push_str(&format!(
                        "     ⚠️  {} shows bias: {:.1}% true\n",
                        type_name,
                        coverage.true_ratio * 100.0
                    ));
                }
            }
        }

        // Analyze enum coverage
        for (type_name, coverage) in &self.stats.coverage_info.enum_coverage {
            if coverage.coverage_percentage < 1.0 {
                let missing_variants = (1.0 - coverage.coverage_percentage)
                    * coverage.variant_distribution.len() as f64;
                analysis.push_str(&format!(
                    "     ⚠️  {} missing ~{:.0} variants\n",
                    type_name, missing_variants
                ));
            }

            // Check for variant bias
            if let (Some((least, least_count)), Some((most, most_count))) = (
                coverage.get_least_covered_variant(),
                coverage.get_most_covered_variant(),
            ) {
                let ratio = *most_count as f64 / (*least_count as f64).max(1.0);
                if ratio > 5.0 {
                    analysis.push_str(&format!(
                        "       ⚠️  Variant bias: {} ({}) vs {} ({})\n",
                        most, most_count, least, least_count
                    ));
                }
            }
        }

        analysis.push('\n');
        analysis
    }

    /// Analyze performance characteristics
    fn analyze_performance(&self) -> String {
        let mut analysis = String::new();

        analysis.push_str("   Performance Insights:\n");

        let metrics = &self.stats.performance_metrics;

        // Generation rate
        if metrics.total_generation_time.as_secs_f64() > 0.0 {
            let rate =
                self.stats.total_generated as f64 / metrics.total_generation_time.as_secs_f64();
            analysis.push_str(&format!(
                "     Generation rate: {:.0} values/second\n",
                rate
            ));
        }

        // Performance consistency
        if metrics.slowest_generation.as_nanos() > 0 && metrics.fastest_generation.as_nanos() > 0 {
            let ratio = metrics.slowest_generation.as_nanos() as f64
                / metrics.fastest_generation.as_nanos() as f64;
            if ratio > 10.0 {
                analysis.push_str(&format!(
                    "     ⚠️  High performance variance: {:.1}x difference\n",
                    ratio
                ));
            }
        }

        // Memory efficiency
        if metrics.memory_stats.total_allocations > 0 {
            let avg_allocation =
                metrics.memory_stats.peak_memory_usage / metrics.memory_stats.total_allocations;
            analysis.push_str(&format!(
                "     Average allocation size: {} bytes\n",
                avg_allocation
            ));
        }

        analysis.push('\n');
        analysis
    }

    /// Check coverage against thresholds and generate recommendations
    pub fn check_coverage_and_recommend(
        &self,
        thresholds: &CoverageThresholds,
    ) -> (CoverageReport, Vec<String>) {
        let report = self.stats.check_coverage_thresholds(thresholds);
        let mut recommendations = Vec::new();

        // Generate recommendations based on coverage gaps
        for (type_name, passed) in &report.numeric_results {
            if !passed {
                recommendations.push(format!("Increase iterations or adjust generator for {} to improve numeric range coverage", type_name));
            }
        }

        for (type_name, passed) in &report.boolean_results {
            if !passed {
                recommendations.push(format!(
                    "Ensure {} generator produces both true and false values",
                    type_name
                ));
            }
        }

        for (type_name, passed) in &report.enum_results {
            if !passed {
                recommendations.push(format!(
                    "Improve variant distribution for {} enum type",
                    type_name
                ));
            }
        }

        // Performance recommendations
        if self.stats.performance_metrics.average_generation_time > Duration::from_millis(1) {
            recommendations
                .push("Consider optimizing generators for better performance".to_string());
        }

        (report, recommendations)
    }
}

impl Default for StatisticsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            peak_usage: 0,
            current_usage: 0,
            allocations: 0,
        }
    }

    fn record_allocation(&mut self, size: usize) {
        self.current_usage += size;
        self.peak_usage = self.peak_usage.max(self.current_usage);
        self.allocations += 1;
    }

    fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            peak_memory_usage: self.peak_usage,
            average_memory_usage: if self.allocations > 0 {
                self.current_usage / self.allocations
            } else {
                0
            },
            total_allocations: self.allocations,
        }
    }

    fn reset(&mut self) {
        self.peak_usage = 0;
        self.current_usage = 0;
        self.allocations = 0;
    }
}

/// Builder for creating coverage thresholds
pub struct CoverageThresholdsBuilder {
    thresholds: CoverageThresholds,
}

impl CoverageThresholdsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            thresholds: CoverageThresholds {
                numeric_thresholds: HashMap::new(),
                require_full_boolean_coverage: false,
                enum_thresholds: HashMap::new(),
            },
        }
    }

    /// Add a numeric threshold
    pub fn numeric_threshold(
        mut self,
        type_name: &str,
        min_value: f64,
        max_value: f64,
        min_coverage: f64,
    ) -> Self {
        use crate::config::NumericThreshold;
        self.thresholds.numeric_thresholds.insert(
            type_name.to_string(),
            NumericThreshold {
                min_value,
                max_value,
                min_coverage,
            },
        );
        self
    }

    /// Require full boolean coverage
    pub fn require_full_boolean_coverage(mut self) -> Self {
        self.thresholds.require_full_boolean_coverage = true;
        self
    }

    /// Add an enum threshold
    pub fn enum_threshold(mut self, type_name: &str, min_coverage: f64) -> Self {
        use crate::config::EnumThreshold;
        self.thresholds
            .enum_thresholds
            .insert(type_name.to_string(), EnumThreshold { min_coverage });
        self
    }

    /// Build the thresholds
    pub fn build(self) -> CoverageThresholds {
        self.thresholds
    }
}

impl Default for CoverageThresholdsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_statistics_collector_basic() {
        let mut collector = StatisticsCollector::new();
        assert!(collector.is_enabled());
        assert_eq!(collector.get_stats().total_generated, 0);

        // Record some values
        collector.record_generated_value(&42i32, "i32");
        collector.record_generated_value(&true, "bool");
        collector.record_generated_value(&"hello".to_string(), "String");

        let stats = collector.get_stats();
        assert_eq!(stats.total_generated, 3);
        assert!(stats.coverage_info.numeric_coverage.contains_key("i32"));
        assert!(stats.coverage_info.boolean_coverage.contains_key("bool"));
        assert!(stats.coverage_info.string_coverage.contains_key("String"));
    }

    #[test]
    fn test_statistics_collector_disabled() {
        let mut collector = StatisticsCollector::disabled();
        assert!(!collector.is_enabled());

        collector.record_generated_value(&42i32, "i32");
        assert_eq!(collector.get_stats().total_generated, 0);

        collector.enable();
        collector.record_generated_value(&42i32, "i32");
        assert_eq!(collector.get_stats().total_generated, 1);
    }

    #[test]
    fn test_numeric_coverage() {
        let mut collector = StatisticsCollector::new();

        // Record various numeric values
        for i in 0..100 {
            collector.record_generated_value(&i, "i32");
        }

        let stats = collector.get_stats();
        let numeric_coverage = stats.coverage_info.numeric_coverage.get("i32").unwrap();

        assert_eq!(numeric_coverage.total_count, 100);
        assert_eq!(numeric_coverage.min_value, 0.0);
        assert_eq!(numeric_coverage.max_value, 99.0);
        assert!((numeric_coverage.statistics.mean - 49.5).abs() < 1.0);
    }

    #[test]
    fn test_boolean_coverage() {
        let mut collector = StatisticsCollector::new();

        // Record boolean values
        collector.record_generated_value(&true, "bool");
        collector.record_generated_value(&false, "bool");
        collector.record_generated_value(&true, "bool");

        let stats = collector.get_stats();
        let boolean_coverage = stats.coverage_info.boolean_coverage.get("bool").unwrap();

        assert_eq!(boolean_coverage.total_count, 3);
        assert_eq!(boolean_coverage.true_count, 2);
        assert_eq!(boolean_coverage.false_count, 1);
        assert!(boolean_coverage.has_full_coverage());
        assert!((boolean_coverage.true_ratio - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_string_coverage() {
        let mut collector = StatisticsCollector::new();

        // Record string values
        collector.record_generated_value(&"hello".to_string(), "String");
        collector.record_generated_value(&"world".to_string(), "String");
        collector.record_generated_value(&"test123".to_string(), "String");

        let stats = collector.get_stats();
        let string_coverage = stats.coverage_info.string_coverage.get("String").unwrap();

        assert_eq!(string_coverage.total_count, 3);
        assert!(string_coverage.length_distribution.contains_key(&5)); // "hello", "world"
        assert!(string_coverage.length_distribution.contains_key(&7)); // "test123"
        assert!(string_coverage.character_distribution.contains_key(&'h'));
        assert!(string_coverage.pattern_coverage.contains_key("alphabetic"));
        assert!(
            string_coverage
                .pattern_coverage
                .contains_key("alphanumeric")
        );
    }

    #[test]
    fn test_enum_coverage() {
        let mut collector = StatisticsCollector::new();

        // Simulate enum variants
        collector.record_enum_variant("Color", "Red", 3);
        collector.record_enum_variant("Color", "Green", 3);
        collector.record_enum_variant("Color", "Red", 3);

        let stats = collector.get_stats();
        let enum_coverage = stats.coverage_info.enum_coverage.get("Color").unwrap();

        assert_eq!(enum_coverage.total_count, 3);
        assert_eq!(enum_coverage.variant_distribution.len(), 2); // Red and Green
        assert!((enum_coverage.coverage_percentage - 2.0 / 3.0).abs() < 0.01); // 2 out of 3 variants
    }

    #[test]
    fn test_performance_timing() {
        let mut collector = StatisticsCollector::new();

        // Simulate generation timing
        collector.start_generation_timing();
        thread::sleep(Duration::from_millis(1));
        collector.end_generation_timing();
        collector.record_generated_value(&42, "i32");

        collector.start_generation_timing();
        thread::sleep(Duration::from_millis(2));
        collector.end_generation_timing();
        collector.record_generated_value(&43, "i32");

        let stats = collector.get_stats();
        assert!(stats.performance_metrics.total_generation_time > Duration::from_millis(2));
        assert!(stats.performance_metrics.average_generation_time > Duration::from_millis(1));
        assert!(
            stats.performance_metrics.fastest_generation
                < stats.performance_metrics.slowest_generation
        );
    }

    #[test]
    fn test_coverage_thresholds_builder() {
        let thresholds = CoverageThresholdsBuilder::new()
            .numeric_threshold("i32", 0.0, 100.0, 0.8)
            .require_full_boolean_coverage()
            .enum_threshold("Color", 0.9)
            .build();

        assert!(thresholds.numeric_thresholds.contains_key("i32"));
        assert!(thresholds.require_full_boolean_coverage);
        assert!(thresholds.enum_thresholds.contains_key("Color"));
    }

    #[test]
    fn test_coverage_report() {
        let mut collector = StatisticsCollector::new();

        // Generate some data
        collector.record_generated_value(&true, "bool");
        collector.record_generated_value(&false, "bool");

        for i in 0..50 {
            collector.record_generated_value(&i, "i32");
        }

        let thresholds = CoverageThresholdsBuilder::new()
            .numeric_threshold("i32", 0.0, 100.0, 0.4) // Should pass (50/100 = 0.5 > 0.4)
            .require_full_boolean_coverage() // Should pass (both true and false)
            .build();

        let (report, _recommendations) = collector.check_coverage_and_recommend(&thresholds);

        assert!(report.overall_pass);
        assert!(report.boolean_results.get("bool").unwrap_or(&false));
        // Note: numeric threshold checking is simplified in this implementation
    }

    #[test]
    fn test_analysis_report_generation() {
        let mut collector = StatisticsCollector::new();

        // Generate diverse data
        for i in 0..10 {
            collector.record_generated_value(&i, "i32");
            collector.record_generated_value(&(i % 2 == 0), "bool");
            collector.record_generated_value(&format!("string_{}", i), "String");
        }

        let report = collector.generate_analysis_report();
        assert!(report.contains("GENERATION ANALYSIS"));
        assert!(report.contains("Total values generated: 30"));
        assert!(report.contains("Coverage Pattern Analysis"));
    }

    #[test]
    fn test_collector_reset() {
        let mut collector = StatisticsCollector::new();

        collector.record_generated_value(&42, "i32");
        assert_eq!(collector.get_stats().total_generated, 1);

        collector.reset();
        assert_eq!(collector.get_stats().total_generated, 0);
        assert!(
            collector
                .get_stats()
                .coverage_info
                .numeric_coverage
                .is_empty()
        );
    }
}

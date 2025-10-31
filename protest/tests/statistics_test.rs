//! Tests for generation statistics and coverage functionality.

use protest::generator::ConstantGenerator;
use protest::primitives::{BoolGenerator, IntGenerator, StringGenerator, VecGenerator};
use protest::{
    CoverageThresholdsBuilder, Property, PropertyError, PropertyTestBuilder, StatisticsCollector,
    TestConfig, check, check_with_config,
};
use std::time::Duration;

/// Simple property that always passes
struct AlwaysPassProperty;
impl Property<i32> for AlwaysPassProperty {
    type Output = ();
    fn test(&self, _input: i32) -> Result<Self::Output, PropertyError> {
        Ok(())
    }
}

/// Property that always passes for any type
struct AlwaysPassAnyProperty<T>(std::marker::PhantomData<T>);
impl<T> AlwaysPassAnyProperty<T> {
    fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}
impl<T> Property<T> for AlwaysPassAnyProperty<T> {
    type Output = ();
    fn test(&self, _input: T) -> Result<Self::Output, PropertyError> {
        Ok(())
    }
}

#[test]
fn test_basic_statistics_collection() {
    let result = PropertyTestBuilder::new()
        .iterations(50)
        .enable_statistics()
        .run(IntGenerator::<i32>::new(1, 100), AlwaysPassProperty);

    assert!(result.is_ok());
    let success = result.unwrap();

    // Check that statistics were collected
    assert!(success.stats.is_some());
    let stats = success.stats.unwrap();
    assert_eq!(stats.total_generated, 50);

    // Check that numeric coverage was recorded
    assert!(!stats.coverage_info.numeric_coverage.is_empty());

    // Check performance metrics
    assert!(stats.performance_metrics.total_generation_time > Duration::from_nanos(0));
    assert!(stats.performance_metrics.average_generation_time > Duration::from_nanos(0));
}

#[test]
fn test_statistics_disabled() {
    let result = PropertyTestBuilder::new()
        .iterations(50)
        .disable_statistics()
        .run(IntGenerator::<i32>::new(1, 100), AlwaysPassProperty);

    assert!(result.is_ok());
    let success = result.unwrap();

    // Statistics should be None when disabled
    assert!(success.stats.is_none());
}

#[test]
fn test_numeric_coverage_tracking() {
    let result = PropertyTestBuilder::new()
        .iterations(100)
        .enable_statistics()
        .run(IntGenerator::<i32>::new(0, 50), AlwaysPassProperty);

    assert!(result.is_ok());
    let success = result.unwrap();
    let stats = success.stats.unwrap();

    // Check numeric coverage
    assert!(!stats.coverage_info.numeric_coverage.is_empty());

    if let Some(numeric_coverage) = stats.coverage_info.numeric_coverage.values().next() {
        assert_eq!(numeric_coverage.total_count, 100);
        assert!(numeric_coverage.min_value >= 0.0);
        assert!(numeric_coverage.max_value <= 50.0);
        assert!(numeric_coverage.statistics.mean >= 0.0);
        assert!(numeric_coverage.statistics.mean <= 50.0);
    }
}

#[test]
fn test_boolean_coverage_tracking() {
    let result = PropertyTestBuilder::new()
        .iterations(100)
        .enable_statistics()
        .run(BoolGenerator, AlwaysPassAnyProperty::<bool>::new());

    assert!(result.is_ok());
    let success = result.unwrap();
    let stats = success.stats.unwrap();

    // Check boolean coverage
    assert!(!stats.coverage_info.boolean_coverage.is_empty());

    if let Some(boolean_coverage) = stats.coverage_info.boolean_coverage.values().next() {
        assert_eq!(boolean_coverage.total_count, 100);
        // With 100 iterations, we should have both true and false values
        assert!(boolean_coverage.has_full_coverage());
        assert!(boolean_coverage.true_ratio > 0.0 && boolean_coverage.true_ratio < 1.0);
    }
}

#[test]
fn test_string_coverage_tracking() {
    let result = PropertyTestBuilder::new()
        .iterations(50)
        .enable_statistics()
        .run(
            StringGenerator::ascii_printable(1, 20),
            AlwaysPassAnyProperty::<String>::new(),
        );

    assert!(result.is_ok());
    let success = result.unwrap();
    let stats = success.stats.unwrap();

    // Check string coverage
    assert!(!stats.coverage_info.string_coverage.is_empty());

    if let Some(string_coverage) = stats.coverage_info.string_coverage.values().next() {
        assert_eq!(string_coverage.total_count, 50);
        assert!(string_coverage.average_length >= 0.0);
        assert!(!string_coverage.length_distribution.is_empty());
        assert!(!string_coverage.character_distribution.is_empty());
    }
}

#[test]
fn test_collection_coverage_tracking() {
    let result = PropertyTestBuilder::new()
        .iterations(30)
        .enable_statistics()
        .run(
            VecGenerator::new(IntGenerator::<i32>::new(1, 100), 0, 10),
            AlwaysPassAnyProperty::<Vec<i32>>::new(),
        );

    assert!(result.is_ok());
    let success = result.unwrap();
    let stats = success.stats.unwrap();

    // Check collection coverage
    assert!(!stats.coverage_info.collection_coverage.is_empty());

    if let Some(collection_coverage) = stats.coverage_info.collection_coverage.values().next() {
        assert_eq!(collection_coverage.total_count, 30);
        assert!(collection_coverage.average_size >= 0.0);
        assert!(!collection_coverage.size_distribution.is_empty());
    }
}

#[test]
fn test_statistics_report_generation() {
    let result = PropertyTestBuilder::new()
        .iterations(20)
        .enable_statistics()
        .run(IntGenerator::<i32>::new(1, 10), AlwaysPassProperty);

    assert!(result.is_ok());
    let success = result.unwrap();
    let stats = success.stats.unwrap();

    // Test report generation
    let report = stats.generate_report();
    assert!(report.contains("GENERATION STATISTICS"));
    assert!(report.contains("Total values generated: 20"));
    assert!(report.contains("COVERAGE INFORMATION"));
    assert!(report.contains("PERFORMANCE METRICS"));

    // Test summary
    let summary = stats.get_summary();
    assert!(summary.contains("Generated 20 values"));
}

#[test]
fn test_coverage_thresholds() {
    let result = PropertyTestBuilder::new()
        .iterations(100)
        .enable_statistics()
        .run(BoolGenerator, AlwaysPassAnyProperty::<bool>::new());

    assert!(result.is_ok());
    let success = result.unwrap();
    let stats = success.stats.unwrap();

    // Create coverage thresholds
    let thresholds = CoverageThresholdsBuilder::new()
        .require_full_boolean_coverage()
        .build();

    // Check coverage against thresholds
    let report = stats.check_coverage_thresholds(&thresholds);

    // With 100 iterations, boolean coverage should be complete
    assert!(report.overall_pass);
    if let Some(&passed) = report.boolean_results.values().next() {
        assert!(passed);
    }
}

#[test]
fn test_statistics_collector_direct_usage() {
    let mut collector = StatisticsCollector::new();

    // Record some values directly
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
fn test_statistics_collector_timing() {
    let mut collector = StatisticsCollector::new();

    // Test timing functionality
    collector.start_generation_timing();
    std::thread::sleep(Duration::from_millis(1));
    collector.end_generation_timing();
    collector.record_generated_value(&42, "i32");

    let stats = collector.get_stats();
    assert!(stats.performance_metrics.total_generation_time > Duration::from_nanos(0));
    assert!(stats.performance_metrics.average_generation_time > Duration::from_nanos(0));
}

#[test]
fn test_statistics_collector_reset() {
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

#[test]
fn test_enum_coverage_tracking() {
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
fn test_performance_analysis() {
    let mut collector = StatisticsCollector::new();

    // Generate some data with timing
    for i in 0..10 {
        collector.start_generation_timing();
        std::thread::sleep(Duration::from_millis(1));
        collector.end_generation_timing();
        collector.record_generated_value(&i, "i32");
    }

    let report = collector.generate_analysis_report();
    assert!(report.contains("GENERATION ANALYSIS"));
    assert!(report.contains("Generation Time Distribution"));
    assert!(report.contains("Coverage Pattern Analysis"));
    assert!(report.contains("Performance Insights"));
}

#[test]
fn test_coverage_recommendations() {
    let mut collector = StatisticsCollector::new();

    // Generate only true values (incomplete boolean coverage)
    for _ in 0..10 {
        collector.record_generated_value(&true, "bool");
    }

    let thresholds = CoverageThresholdsBuilder::new()
        .require_full_boolean_coverage()
        .build();

    let (report, recommendations) = collector.check_coverage_and_recommend(&thresholds);

    assert!(!report.overall_pass); // Should fail due to incomplete boolean coverage
    assert!(!recommendations.is_empty()); // Should have recommendations

    // Check that recommendations mention boolean coverage
    let recommendations_text = recommendations.join(" ");
    assert!(recommendations_text.to_lowercase().contains("bool"));
}

#[test]
fn test_statistics_with_different_types() {
    let mut collector = StatisticsCollector::new();

    // Test with various numeric types
    collector.record_generated_value(&42u8, "u8");
    collector.record_generated_value(&1000u16, "u16");
    collector.record_generated_value(&-42i32, "i32");
    collector.record_generated_value(&3.15f64, "f64");

    let stats = collector.get_stats();
    assert_eq!(stats.total_generated, 4);

    // Should have numeric coverage for different types
    assert!(!stats.coverage_info.numeric_coverage.is_empty()); // At least one type should be tracked
}

#[test]
fn test_statistics_integration_with_check_functions() {
    // Test with basic check function
    let result = check(IntGenerator::<i32>::new(1, 10), AlwaysPassProperty);
    assert!(result.is_ok());

    // Statistics should be collected by default
    let success = result.unwrap();
    assert!(success.stats.is_some());

    // Test with custom config
    let config = TestConfig {
        iterations: 25,
        ..TestConfig::default()
    };

    let result = check_with_config(IntGenerator::<i32>::new(1, 10), AlwaysPassProperty, config);
    assert!(result.is_ok());

    let success = result.unwrap();
    assert!(success.stats.is_some());
    let stats = success.stats.unwrap();
    assert_eq!(stats.total_generated, 25);
}

#[test]
fn test_memory_tracking() {
    let mut collector = StatisticsCollector::new();

    // Record some values to trigger memory tracking
    for i in 0..10 {
        collector.record_generated_value(&vec![i; 100], "Vec<i32>");
    }

    let stats = collector.get_stats();

    // Memory stats should be recorded
    assert!(stats.performance_metrics.memory_stats.total_allocations > 0);
    // Peak memory usage should be greater than 0
    assert!(stats.performance_metrics.memory_stats.peak_memory_usage > 0);
}

#[test]
fn test_statistics_with_custom_generator() {
    // Use the existing ConstantGenerator
    let custom_generator = ConstantGenerator::new(42);

    let result = PropertyTestBuilder::new()
        .iterations(20)
        .enable_statistics()
        .run(custom_generator, AlwaysPassProperty);

    assert!(result.is_ok());
    let success = result.unwrap();
    let stats = success.stats.unwrap();

    assert_eq!(stats.total_generated, 20);

    // All values should be the same (42)
    if let Some(numeric_coverage) = stats.coverage_info.numeric_coverage.values().next() {
        assert_eq!(numeric_coverage.min_value, 42.0);
        assert_eq!(numeric_coverage.max_value, 42.0);
        assert_eq!(numeric_coverage.statistics.mean, 42.0);
        assert_eq!(numeric_coverage.statistics.variance, 0.0);
    }
}

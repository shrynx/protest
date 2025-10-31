//! Demonstration of generation statistics and coverage functionality in Protest.

use protest::primitives::{BoolGenerator, IntGenerator, StringGenerator, VecGenerator};
use protest::{
    CoverageThresholdsBuilder, Property, PropertyError, PropertyTestBuilder, StatisticsCollector,
};

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

fn main() {
    println!("=== Protest Statistics and Coverage Demo ===\n");

    // Demo 1: Basic statistics collection
    println!("1. Basic Statistics Collection:");
    let result = PropertyTestBuilder::new()
        .iterations(100)
        .enable_statistics()
        .run(IntGenerator::<i32>::new(1, 100), AlwaysPassProperty);

    if let Ok(success) = result
        && let Some(stats) = success.stats
    {
        println!("   Total values generated: {}", stats.total_generated);
        println!(
            "   Generation time: {:?}",
            stats.performance_metrics.total_generation_time
        );
        println!(
            "   Average time per value: {:?}",
            stats.performance_metrics.average_generation_time
        );

        // Show numeric coverage
        if let Some(numeric_coverage) = stats.coverage_info.numeric_coverage.values().next() {
            println!(
                "   Numeric range: [{:.1}, {:.1}]",
                numeric_coverage.min_value, numeric_coverage.max_value
            );
            println!("   Mean value: {:.2}", numeric_coverage.statistics.mean);
        }
    }
    println!();

    // Demo 2: Boolean coverage tracking
    println!("2. Boolean Coverage Tracking:");
    let result = PropertyTestBuilder::new()
        .iterations(50)
        .enable_statistics()
        .run(BoolGenerator, AlwaysPassAnyProperty::<bool>::new());

    if let Ok(success) = result
        && let Some(stats) = success.stats
        && let Some(bool_coverage) = stats.coverage_info.boolean_coverage.values().next()
    {
        println!("   Total booleans generated: {}", bool_coverage.total_count);
        println!("   True count: {}", bool_coverage.true_count);
        println!("   False count: {}", bool_coverage.false_count);
        println!("   True ratio: {:.2}", bool_coverage.true_ratio);
        println!("   Full coverage: {}", bool_coverage.has_full_coverage());
    }
    println!();

    // Demo 3: String coverage tracking
    println!("3. String Coverage Tracking:");
    let result = PropertyTestBuilder::new()
        .iterations(30)
        .enable_statistics()
        .run(
            StringGenerator::ascii_printable(5, 15),
            AlwaysPassAnyProperty::<String>::new(),
        );

    if let Ok(success) = result
        && let Some(stats) = success.stats
        && let Some(string_coverage) = stats.coverage_info.string_coverage.values().next()
    {
        println!(
            "   Total strings generated: {}",
            string_coverage.total_count
        );
        println!("   Average length: {:.1}", string_coverage.average_length);
        println!(
            "   Unique characters used: {}",
            string_coverage.character_distribution.len()
        );
        println!(
            "   Length distribution: {:?}",
            string_coverage.length_distribution
        );
    }
    println!();

    // Demo 4: Collection coverage tracking
    println!("4. Collection Coverage Tracking:");
    let result = PropertyTestBuilder::new()
        .iterations(25)
        .enable_statistics()
        .run(
            VecGenerator::new(IntGenerator::<i32>::new(1, 10), 0, 5),
            AlwaysPassAnyProperty::<Vec<i32>>::new(),
        );

    if let Ok(success) = result
        && let Some(stats) = success.stats
        && let Some(collection_coverage) = stats.coverage_info.collection_coverage.values().next()
    {
        println!(
            "   Total collections generated: {}",
            collection_coverage.total_count
        );
        println!("   Average size: {:.1}", collection_coverage.average_size);
        println!(
            "   Size range: [{}, {}]",
            collection_coverage.min_size, collection_coverage.max_size
        );
        println!(
            "   Size distribution: {:?}",
            collection_coverage.size_distribution
        );
    }
    println!();

    // Demo 5: Direct statistics collector usage
    println!("5. Direct Statistics Collector Usage:");
    let mut collector = StatisticsCollector::new();

    // Record some values directly
    for i in 0..20 {
        collector.record_generated_value(&(i % 3), "i32");
        collector.record_generated_value(&(i % 2 == 0), "bool");
    }

    let stats = collector.get_stats();
    println!("   Total values recorded: {}", stats.total_generated);
    println!(
        "   Types tracked: {} numeric, {} boolean",
        stats.coverage_info.numeric_coverage.len(),
        stats.coverage_info.boolean_coverage.len()
    );
    println!();

    // Demo 6: Coverage thresholds and analysis
    println!("6. Coverage Analysis and Thresholds:");
    let mut collector = StatisticsCollector::new();

    // Generate some data with potential coverage gaps
    for i in 0..50 {
        collector.record_generated_value(&(i % 10), "i32"); // Limited range
        if i < 45 {
            // Missing some false values
            collector.record_generated_value(&true, "bool");
        } else {
            collector.record_generated_value(&false, "bool");
        }
    }

    // Set up coverage thresholds
    let thresholds = CoverageThresholdsBuilder::new()
        .numeric_threshold("i32", 0.0, 20.0, 0.5) // Expect 50% coverage of 0-20 range
        .require_full_boolean_coverage() // Expect both true and false
        .build();

    let (report, recommendations) = collector.check_coverage_and_recommend(&thresholds);

    println!("   Coverage report:");
    println!("   Overall pass: {}", report.overall_pass);
    for (type_name, passed) in &report.boolean_results {
        println!(
            "   {} boolean coverage: {}",
            type_name,
            if *passed { "✓" } else { "✗" }
        );
    }

    if !recommendations.is_empty() {
        println!("   Recommendations:");
        for rec in &recommendations {
            println!("   - {}", rec);
        }
    }
    println!();

    // Demo 7: Comprehensive analysis report
    println!("7. Comprehensive Analysis Report:");
    let result = PropertyTestBuilder::new()
        .iterations(100)
        .enable_statistics()
        .run(IntGenerator::<i32>::new(-50, 50), AlwaysPassProperty);

    if let Ok(success) = result
        && let Some(stats) = success.stats
    {
        println!("   === GENERATION STATISTICS REPORT ===");
        let summary = stats.get_summary();
        println!("   {}", summary);

        // Show a portion of the full report
        let full_report = stats.generate_report();
        let lines: Vec<&str> = full_report.lines().take(15).collect();
        for line in lines {
            println!("   {}", line);
        }
        println!("   ... (truncated for demo)");
    }

    println!("\n=== Demo Complete ===");
    println!("The statistics system provides comprehensive tracking of:");
    println!("• Generation performance metrics");
    println!("• Value distribution and coverage");
    println!("• Type-specific statistics");
    println!("• Coverage threshold validation");
    println!("• Detailed analysis and recommendations");
}

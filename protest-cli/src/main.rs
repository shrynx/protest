use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use protest::{FailureSnapshot, RegressionConfig, RegressionGenerator};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "protest")]
#[command(about = "Manage Protest property test failures", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to the failures directory
    #[arg(short, long, default_value = ".protest/failures")]
    dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all tests with saved failures
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show details of failures for a specific test
    Show {
        /// Name of the test
        test_name: String,
    },
    /// Remove saved failures
    Clean {
        /// Name of the test (omit to clean all)
        test_name: Option<String>,

        /// Specific seed to delete
        #[arg(short, long)]
        seed: Option<u64>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Show statistics about saved failures
    Stats,
    /// Generate regression tests from saved failures
    Generate {
        /// Name of the test (omit to generate for all tests)
        test_name: Option<String>,

        /// Output directory for generated test files
        #[arg(short, long, default_value = "tests/regressions")]
        output: PathBuf,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let snapshot = FailureSnapshot::new(&cli.dir).context("Failed to open failures directory")?;

    match cli.command {
        Commands::List { verbose } => list_failures(&snapshot, verbose),
        Commands::Show { test_name } => show_failures(&snapshot, &test_name),
        Commands::Clean {
            test_name,
            seed,
            yes,
        } => clean_failures(&snapshot, test_name, seed, yes),
        Commands::Stats => show_stats(&snapshot),
        Commands::Generate {
            test_name,
            output,
            yes,
        } => generate_regressions(&snapshot, test_name, output, yes),
    }
}

fn list_failures(snapshot: &FailureSnapshot, verbose: bool) -> Result<()> {
    let tests = snapshot
        .list_tests_with_failures()
        .context("Failed to list tests")?;

    if tests.is_empty() {
        println!("{}", "No saved failures found.".green());
        return Ok(());
    }

    // Filter out tests with no failures
    let mut tests_with_failures = Vec::new();
    for test_name in &tests {
        if let Ok(failures) = snapshot.load_failures(test_name)
            && !failures.is_empty()
        {
            tests_with_failures.push((test_name.clone(), failures));
        }
    }

    if tests_with_failures.is_empty() {
        println!("{}", "No saved failures found.".green());
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} test(s) with failures:", tests_with_failures.len()).bold()
    );
    println!();

    for (test_name, failures) in &tests_with_failures {
        print!("  {} ", "●".bright_red());
        print!("{}", test_name.cyan().bold());
        println!(
            " ({} failure{})",
            failures.len(),
            if failures.len() == 1 { "" } else { "s" }
        );

        if verbose {
            for failure in failures {
                println!("      {} seed={}", "→".bright_black(), failure.seed);
                println!(
                    "        {}",
                    format!("Error: {}", failure.error_message).bright_black()
                );
                if failure.shrink_steps > 0 {
                    println!(
                        "        {}",
                        format!("Shrink steps: {}", failure.shrink_steps).bright_black()
                    );
                }
            }
            println!();
        }
    }

    if !verbose {
        println!();
        println!("{}", "Tip: Use --verbose for more details".bright_black());
    }

    Ok(())
}

fn show_failures(snapshot: &FailureSnapshot, test_name: &str) -> Result<()> {
    let failures = snapshot
        .load_failures(test_name)
        .context(format!("Failed to load failures for '{}'", test_name))?;

    if failures.is_empty() {
        println!(
            "{}",
            format!("No failures found for test '{}'", test_name).yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Failures for test '{}':", test_name).bold().cyan()
    );
    println!();

    for (idx, failure) in failures.iter().enumerate() {
        println!("{}", format!("Failure #{}", idx + 1).bold());
        println!("  {}: {}", "Seed".bold(), failure.seed);
        println!("  {}: {}", "Input".bold(), failure.input);
        println!("  {}: {}", "Error".bold(), failure.error_message.red());
        println!("  {}: {}", "Shrink steps".bold(), failure.shrink_steps);

        use chrono::{DateTime, Utc};
        let datetime = DateTime::<Utc>::from(failure.timestamp);
        println!(
            "  {}: {}",
            "Timestamp".bold(),
            datetime.format("%Y-%m-%d %H:%M:%S UTC")
        );

        if !failure.metadata.is_empty() {
            println!("  {}:", "Metadata".bold());
            for (key, value) in &failure.metadata {
                println!("    {}: {}", key, value);
            }
        }

        println!();
        println!(
            "  {}",
            "Reproduce: cargo test -- --nocapture".bright_black()
        );
        println!(
            "  {}",
            format!("Or use: .seed({})", failure.seed).bright_black()
        );
        println!();
    }

    Ok(())
}

fn clean_failures(
    snapshot: &FailureSnapshot,
    test_name: Option<String>,
    seed: Option<u64>,
    yes: bool,
) -> Result<()> {
    match (test_name, seed) {
        // Clean specific seed for specific test
        (Some(test), Some(seed_val)) => {
            if !yes {
                print!(
                    "Delete failure for test '{}' with seed {}? [y/N] ",
                    test, seed_val
                );
                if !confirm()? {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            snapshot.delete_failure(&test, seed_val).context(format!(
                "Failed to delete failure for test '{}' with seed {}",
                test, seed_val
            ))?;
            println!(
                "{}",
                format!("✓ Deleted failure for '{}' with seed {}", test, seed_val).green()
            );
        }

        // Clean all failures for specific test
        (Some(test), None) => {
            let failures = snapshot
                .load_failures(&test)
                .context(format!("Failed to load failures for '{}'", test))?;

            if failures.is_empty() {
                println!(
                    "{}",
                    format!("No failures found for test '{}'", test).yellow()
                );
                return Ok(());
            }

            if !yes {
                print!(
                    "Delete {} failure(s) for test '{}'? [y/N] ",
                    failures.len(),
                    test
                );
                if !confirm()? {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let mut deleted = 0;
            for failure in failures {
                if snapshot.delete_failure(&test, failure.seed).is_ok() {
                    deleted += 1;
                }
            }

            println!(
                "{}",
                format!("✓ Deleted {} failure(s) for '{}'", deleted, test).green()
            );
        }

        // Clean all failures for all tests
        (None, _) => {
            let tests = snapshot
                .list_tests_with_failures()
                .context("Failed to list tests")?;

            if tests.is_empty() {
                println!("{}", "No saved failures found.".green());
                return Ok(());
            }

            let mut total_failures = 0;
            for test in &tests {
                if let Ok(failures) = snapshot.load_failures(test) {
                    total_failures += failures.len();
                }
            }

            if !yes {
                print!(
                    "Delete ALL {} failure(s) across {} test(s)? [y/N] ",
                    total_failures,
                    tests.len()
                );
                if !confirm()? {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let mut deleted = 0;
            for test in &tests {
                if let Ok(failures) = snapshot.load_failures(test) {
                    for failure in failures {
                        if snapshot.delete_failure(test, failure.seed).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }

            println!(
                "{}",
                format!(
                    "✓ Deleted {} failure(s) across {} test(s)",
                    deleted,
                    tests.len()
                )
                .green()
            );
        }
    }

    Ok(())
}

fn show_stats(snapshot: &FailureSnapshot) -> Result<()> {
    let tests = snapshot
        .list_tests_with_failures()
        .context("Failed to list tests")?;

    if tests.is_empty() {
        println!("{}", "No saved failures found.".green());
        return Ok(());
    }

    let mut total_failures = 0;
    let mut total_shrink_steps = 0;
    let mut oldest_timestamp = None;
    let mut newest_timestamp = None;

    for test_name in &tests {
        if let Ok(failures) = snapshot.load_failures(test_name) {
            total_failures += failures.len();

            for failure in failures {
                total_shrink_steps += failure.shrink_steps;

                match (oldest_timestamp, &failure.timestamp) {
                    (None, ts) => oldest_timestamp = Some(*ts),
                    (Some(oldest), ts) if ts < &oldest => oldest_timestamp = Some(*ts),
                    _ => {}
                }

                match (newest_timestamp, &failure.timestamp) {
                    (None, ts) => newest_timestamp = Some(*ts),
                    (Some(newest), ts) if ts > &newest => newest_timestamp = Some(*ts),
                    _ => {}
                }
            }
        }
    }

    println!("{}", "Failure Statistics".bold().cyan());
    println!();
    println!("  {}: {}", "Total tests with failures".bold(), tests.len());
    println!("  {}: {}", "Total failures".bold(), total_failures);
    println!(
        "  {}: {:.1}",
        "Average failures per test".bold(),
        total_failures as f64 / tests.len() as f64
    );
    println!("  {}: {}", "Total shrink steps".bold(), total_shrink_steps);

    if total_failures > 0 {
        println!(
            "  {}: {:.1}",
            "Average shrink steps per failure".bold(),
            total_shrink_steps as f64 / total_failures as f64
        );
    }

    if let Some(oldest) = oldest_timestamp {
        use chrono::{DateTime, Utc};
        let datetime = DateTime::<Utc>::from(oldest);
        println!(
            "  {}: {}",
            "Oldest failure".bold(),
            datetime.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    if let Some(newest) = newest_timestamp {
        use chrono::{DateTime, Utc};
        let datetime = DateTime::<Utc>::from(newest);
        println!(
            "  {}: {}",
            "Newest failure".bold(),
            datetime.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    println!();

    Ok(())
}

fn generate_regressions(
    snapshot: &FailureSnapshot,
    test_name: Option<String>,
    output_dir: PathBuf,
    yes: bool,
) -> Result<()> {
    let config = RegressionConfig::new(&output_dir);
    let generator = RegressionGenerator::new(config);

    match test_name {
        Some(test) => {
            // Generate for specific test
            let failures = snapshot
                .load_failures(&test)
                .context(format!("Failed to load failures for '{}'", test))?;

            if failures.is_empty() {
                println!(
                    "{}",
                    format!("No failures found for test '{}'", test).yellow()
                );
                return Ok(());
            }

            if !yes {
                print!(
                    "Generate regression tests for {} failure(s) in '{}'? [y/N] ",
                    failures.len(),
                    test
                );
                if !confirm()? {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let file_path = generator
                .generate_for_test(snapshot, &test)
                .context("Failed to generate regression tests")?;

            println!(
                "{}",
                format!("✓ Generated regression tests for '{}'", test).green()
            );
            println!("  {}: {}", "Output file".bold(), file_path.display());
            println!("  {}: {}", "Test count".bold(), failures.len());
            println!();
            println!("{}", "Next steps:".bold());
            println!("  1. Review the generated file: {}", file_path.display());
            println!("  2. Implement the TODO sections with your actual test logic");
            println!(
                "  3. Run: cargo test --test {}",
                file_path.file_stem().unwrap().to_str().unwrap()
            );
        }

        None => {
            // Generate for all tests
            let tests = snapshot
                .list_tests_with_failures()
                .context("Failed to list tests")?;

            if tests.is_empty() {
                println!("{}", "No saved failures found.".green());
                return Ok(());
            }

            let mut total_failures = 0;
            for test in &tests {
                if let Ok(failures) = snapshot.load_failures(test) {
                    total_failures += failures.len();
                }
            }

            if !yes {
                print!(
                    "Generate regression tests for {} failure(s) across {} test(s)? [y/N] ",
                    total_failures,
                    tests.len()
                );
                if !confirm()? {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let files = generator
                .generate_all(snapshot)
                .context("Failed to generate regression tests")?;

            println!(
                "{}",
                format!("✓ Generated regression tests for {} test(s)", files.len()).green()
            );
            println!();
            for file in &files {
                println!("  {} {}", "●".bright_cyan(), file.display());
            }
            println!();
            println!("{}", "Next steps:".bold());
            println!(
                "  1. Review the generated files in: {}",
                output_dir.display()
            );
            println!("  2. Implement the TODO sections with your actual test logic");
            println!("  3. Run: cargo test --test '*_regressions'");
        }
    }

    Ok(())
}

fn confirm() -> Result<bool> {
    use std::io::{self, BufRead};

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    Ok(line.trim().to_lowercase() == "y" || line.trim().to_lowercase() == "yes")
}

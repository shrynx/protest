# Protest CLI

Command-line tool for managing [Protest](https://github.com/shrynx/protest) property test failures.

## Installation

### From Source

```bash
cargo install --path .
```

Or use directly from the workspace:

```bash
cargo run -p protest-cli -- <command>
```

## Usage

```
protest [OPTIONS] <COMMAND>

Commands:
  list      List all tests with saved failures
  show      Show details of failures for a specific test
  clean     Remove saved failures
  stats     Show statistics about saved failures
  generate  Generate regression tests from saved failures
  help      Print help information

Options:
  -d, --dir <DIR>  Path to the failures directory [default: .protest/failures]
  -h, --help       Print help
  -V, --version    Print version
```

## Examples

### List all failures

```bash
# Basic list
protest list

# Verbose output with details
protest list --verbose
```

### Show failure details

```bash
protest show my_test_name
```

Output includes:
- Seed for reproduction
- Input value that caused the failure
- Error message
- Number of shrink steps
- Timestamp
- Metadata (if any)

### View statistics

```bash
protest stats
```

Shows aggregated statistics across all failures:
- Total tests with failures
- Total failure count
- Average failures per test
- Total/average shrink steps
- Oldest and newest failure timestamps

### Clean failures

```bash
# Remove a specific failure by seed
protest clean my_test --seed 12345

# Remove all failures for a test
protest clean my_test

# Remove all failures (with confirmation)
protest clean

# Skip confirmation prompt
protest clean my_test -y
```

### Generate regression tests

Convert saved failures into permanent regression test files that can be committed to your repository.

```bash
# Generate regression tests for a specific test
protest generate my_test

# Generate for all tests with failures
protest generate

# Specify custom output directory
protest generate my_test --output tests/regressions

# Skip confirmation prompt
protest generate -y
```

Generated test files include:
- Test functions with descriptive names based on seeds
- Comments with original error messages and inputs
- Metadata about when the failure was discovered
- TODO sections for implementing the actual test logic
- Seed values for exact reproduction

**Example generated test:**

```rust
/// Regression test for failure with seed 12345
///
/// Original error: Property failed: Value too large
/// Input: 571962454
/// Discovered: 2025-01-02 16:00:00 UTC
/// Shrink steps: 15
#[test]
fn regression_my_test_seed_12345() {
    // TODO: Implement regression test for seed 12345
    // You can reproduce this failure using:
    // PropertyTestBuilder::new()
    //     .seed(12345)
    //     .run(generator, property);
}
```

**Workflow:**
1. Run `protest generate` to create test files
2. Review the generated files in `tests/regressions/`
3. Implement the TODO sections with your actual test logic
4. Run `cargo test` to verify the regression tests pass
5. Commit the test files to your repository

### Custom failure directory

```bash
protest --dir .custom/failures list
protest --dir /path/to/failures show my_test
```

## Integration with CI/CD

### Check for unresolved failures

```bash
# Exit with error if any failures exist
protest list && exit 1 || exit 0
```

### Generate regression tests in CI

```bash
# Automatically generate regression tests from failures
protest generate --output tests/regressions -y

# Commit generated tests (optional)
git add tests/regressions/
git commit -m "Add regression tests from property test failures"
```

### Clean failures after fixing

```bash
# After fixing bugs, clean old failures
protest clean -y
```

### Store failure statistics

```bash
# Export stats to a file (pipe the output)
protest stats > failure_report.txt
```

### Example GitHub Actions workflow

```yaml
name: Property Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run property tests
        run: cargo test --features persistence
        continue-on-error: true

      - name: Check for failures
        run: |
          if protest list | grep -q "failure"; then
            echo "Property test failures detected"
            protest list --verbose
            protest generate -y
            exit 1
          fi

      - name: Upload generated tests
        if: failure()
        uses: actions/upload-artifact@v2
        with:
          name: regression-tests
          path: tests/regressions/
```

## Color Output

The CLI uses colored output for better readability:
- 🔴 Red for test names with failures
- 🔵 Cyan for test names and headers
- ⚪ Gray for hints and secondary information
- 🟢 Green for success messages

Colors can be disabled by setting `NO_COLOR=1` environment variable.

## See Also

- [Protest Documentation](../README.md)
- [Property Testing Guide](../docs/property-testing.md)
- [Failure Persistence](../docs/persistence.md)

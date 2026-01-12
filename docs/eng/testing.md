# Testing Guide

This document provides comprehensive testing instructions for the Save History project.

## Overview

The project includes two types of tests:
- **Unit Tests** - Test individual modules and functions
- **Integration Tests** - Test the entire pipeline end-to-end with real API calls

## Prerequisites

- `GEMINI_API_KEY` environment variable must be set for integration tests
- Rust and Cargo installed

## Unit Tests

Unit tests validate individual components without making external API calls.

### Running Unit Tests

```bash
# Run all unit tests
cargo test --lib

# Run tests for a specific module
cargo test --lib debrief
cargo test --lib processor
cargo test --lib gemini_utils
cargo test --lib autorater
```

### What's Tested

- **`debrief.rs`** - Prompt building, debrief formatting, JSON serialization
- **`gemini_utils.rs`** - File formatting utilities
- **`processor.rs`** - File reading, debrief writing, file marking workflow
- **`autorater.rs`** - Evaluation prompt building, response parsing

### Example Output

```
running 15 tests
test debrief::tests::test_build_prompt_new_debrief ... ok
test debrief::tests::test_format_debrief_as_markdown_empty ... ok
test processor::tests::test_read_directory_files_empty_directory ... ok
...
test result: ok. 15 passed; 0 failed; 0 ignored
```

## Integration Tests

Integration tests validate the entire pipeline using real test data and API calls.

### Running Integration Tests

Integration tests are marked with `#[ignore]` because they:
- Make real API calls to Gemini (costs money/quota)
- Require `GEMINI_API_KEY` to be set
- Take longer to run (30-60 seconds per test)

```bash
# Run all integration tests
cargo test --test integration_test -- --ignored

# Run a specific integration test
cargo test test_integration_hamstring_injury -- --ignored --nocapture
cargo test test_integration_hard_drive -- --ignored --nocapture

# Run research integration tests
cargo test test_research_hard_drive -- --ignored --nocapture
cargo test test_research_all_directories -- --ignored --nocapture
```

### Available Integration Tests

#### Debrief Generation Tests

- **`test_integration_hamstring_injury`** - Tests debrief generation for health/injury topic
- **`test_integration_hard_drive`** - Tests debrief generation for technical troubleshooting topic

These tests verify:
- DEBRIEF.md is created
- Content is substantial (>200 bytes)
- Autorater scores the debrief highly (≥8/10)
- Files are marked as read after processing

#### Research Tests

- **`test_research_hamstring_injury`** - Tests research on health topic
- **`test_research_hard_drive`** - Tests research on technical topic
- **`test_research_all_directories`** - Tests cross-topic research

These tests verify:
- DEBRIEF.md is created
- RESEARCH.md is created with `--research` flag
- RESEARCH.md contains expected header "🔍 Research Insights"
- RESEARCH.md has substantial content (>200 bytes)
- DEBRIEF.md does NOT contain research insights (separation verified)

### Test Data

Integration tests use data from `testdata/` directory:
- `testdata/hamstring_injury/` - Health/injury conversation files
- `testdata/hard_drive/` - Technical troubleshooting conversation files

Tests create temporary directories with copies of this data to avoid polluting the source.

### Example Output

```
running 1 test
Running pipeline for directory: /var/folders/.../hamstring_injury
   Compiling read_files v0.1.0
    Finished `dev` profile
     Running `target/debug/read_files /var/folders/.../hamstring_injury`
DEBRIEF.md not found. Creating it...
Successfully created DEBRIEF.md
Calling Gemini API...
Successfully updated DEBRIEF.md
Marked as read: 1.txt -> 1_read.txt
...

Autorater evaluation (first run): Score 10/10
Reasoning: The debrief is excellent...

test test_integration_hamstring_injury ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; finished in 48.57s
```

## Writing New Tests

### Unit Test Template

```rust
#[test]
fn test_my_feature() {
    // Arrange
    let input = "test data";
    
    // Act
    let result = my_function(input);
    
    // Assert
    assert_eq!(result, expected_value);
}
```

### Integration Test Template

```rust
#[test]
#[ignore]
fn test_integration_my_feature() {
    let test_dir = create_temp_test_dir();
    
    // Set up test data
    fs::write(test_dir.join("1.txt"), "Test conversation").unwrap();
    
    // Run the pipeline
    let output = run_pipeline(&test_dir);
    
    // Verify results
    assert!(output.status.success());
    
    let debrief_path = test_dir.join("DEBRIEF.md");
    assert!(debrief_path.exists());
    
    // Clean up
    fs::remove_dir_all(&test_dir).ok();
}
```

## Helper Functions

The integration test suite provides several helper functions:

- **`create_temp_test_dir()`** - Creates a temporary directory for testing
- **`run_pipeline(dir)`** - Runs the main binary on a directory
- **`run_pipeline_with_research(dir)`** - Runs with `--research` flag
- **`read_debrief(dir)`** - Reads DEBRIEF.md content
- **`read_research(dir)`** - Reads RESEARCH.md content

## Continuous Integration

When setting up CI/CD:

1. **Unit tests** can run on every commit (fast, no API calls)
2. **Integration tests** should run less frequently (slow, costs quota):
   - On pull requests to main
   - On scheduled nightly builds
   - Manually triggered

### CI Configuration Example

```yaml
# Run unit tests on every push
- name: Unit Tests
  run: cargo test --lib

# Run integration tests only on main branch
- name: Integration Tests
  if: github.ref == 'refs/heads/main'
  env:
    GEMINI_API_KEY: ${{ secrets.GEMINI_API_KEY }}
  run: cargo test --test integration_test -- --ignored
```

## Troubleshooting

### "GEMINI_API_KEY not set"

Integration tests require the API key. Set it in your environment:

```bash
export GEMINI_API_KEY="your-api-key-here"
```

Or create a `.env` file in the project root:

```
GEMINI_API_KEY=your-api-key-here
```

### Tests Timing Out

Integration tests can take 30-60 seconds per test due to API calls. This is normal. If tests consistently timeout:

1. Check your internet connection
2. Verify API key is valid
3. Check Gemini API status

### Flaky Integration Tests

Integration tests depend on external API responses, which can occasionally vary. If a test fails:

1. Check the autorater score - scores ≥8/10 are acceptable
2. Verify the API response was reasonable
3. Re-run the test to confirm it's not a transient issue

## Best Practices

1. **Run unit tests frequently** during development
2. **Run integration tests** before committing major changes
3. **Add tests** for new features
4. **Keep test data realistic** but minimal
5. **Clean up** temporary files and directories
6. **Document** any new test helpers or patterns

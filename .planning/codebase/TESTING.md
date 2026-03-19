# Testing Patterns

**Analysis Date:** 2026-03-19

## Test Framework

**Runner:**
- Rust built-in test framework (no external runner required)
- Tests invoked via `cargo test`
- Config: No explicit test configuration file (uses Cargo defaults)

**Assertion Library:**
- Standard Rust `assert!()`, `assert_eq!()` macros
- No external assertion library used

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test -- --nocapture # Run tests with output visible
cargo test --lib       # Run library tests only
```

## Test File Organization

**Location:**
- Tests co-located with implementation in same file
- Test module declared with `#[cfg(test)]` attribute
- Tests placed at end of `.rs` file after main implementation

**Naming:**
- Test functions prefixed with `test_`: `test_parse_vdf_flat()`, `test_branch_from_str()`
- Test modules named `tests` (standard Rust convention)

**Structure:**
```
src/core/steam.rs
├── Public functions and types (main code)
└── #[cfg(test)]
    └── mod tests
        ├── test_parse_vdf_flat()
        ├── test_branch_from_str()
        ├── test_branch_betakey()
        └── test_set_betakey_in_acf()

src/core/saves.rs
├── Public functions and types (main code)
└── #[cfg(test)]
    └── mod tests
        └── test_parse_save_header()
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

**Patterns:**
- Import parent module with `use super::*;`
- Test data defined as constants or constructed in test
- One assertion per test generally, or grouped assertions for related checks
- Test names describe what is tested: `test_parse_vdf_flat()` tests VDF parsing
- Test names describe expected behavior: `test_branch_from_str()` tests string conversion

## Mocking

**Framework:**
- No external mocking library used
- File system operations use temporary directory for testing
- Test data embedded as constants in test modules

**Patterns:**
```rust
const SAMPLE_ACF: &str = r#""AppState"
{
    "appid"     "526870"
    ...
}"#;

#[test]
fn test_parse_vdf_flat() {
    let map = parse_vdf_flat(SAMPLE_ACF);
    assert_eq!(map.get("AppState.appid").unwrap(), "526870");
}
```

**File System Mocking:**
From `src/core/saves.rs`:
```rust
#[test]
fn test_parse_save_header() {
    let dir = std::env::temp_dir().join("satis_switcher_test_saves");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.sav");

    let mut file = fs::File::create(&path).unwrap();
    file.write_all(&13i32.to_le_bytes()).unwrap();
    file.write_all(&46i32.to_le_bytes()).unwrap();
    file.write_all(&264901i32.to_le_bytes()).unwrap();

    let header = parse_save_header(&path).unwrap();
    assert_eq!(header.header_version, 13);
    assert_eq!(header.save_version, 46);
    assert_eq!(header.build_version, 264901);

    fs::remove_file(&path).ok();
    fs::remove_dir(&dir).ok();
}
```

**What to Mock:**
- External system dependencies (file system, processes)
- Complex data structures passed as test data
- Constants representing real-world formats (SAMPLE_ACF)

**What NOT to Mock:**
- Pure functions (parse, calculations)
- String conversions (Branch::from_str)
- Simple type behavior (betakey method)

## Fixtures and Factories

**Test Data:**
- Constants embedded in test module for static data
- Example: `SAMPLE_ACF` in `src/core/steam.rs` represents real appmanifest structure
- Temporary directories created ad-hoc for file-based tests

**Location:**
- Test constants defined at top of `#[cfg(test)] mod tests` block
- Inline construction for simple test data
- No dedicated fixtures directory (tests are co-located with code)

## Coverage

**Requirements:**
- No explicit coverage requirements enforced
- Coverage not configured

**View Coverage:**
```bash
# No built-in coverage command - would require external tool like tarpaulin
cargo tarpaulin  # If installed
```

## Test Types

**Unit Tests:**
- Core of test suite: function-level tests
- Test scope: Single function or small module
- Approach: Test input validation, output correctness, error conditions
- Files with tests: `src/core/steam.rs`, `src/core/saves.rs`
- Examples:
  - `test_parse_vdf_flat()` - VDF parsing logic
  - `test_branch_from_str()` - Branch string conversion
  - `test_parse_save_header()` - Binary file header reading

**Integration Tests:**
- Not implemented
- Would test command execution with real file system
- Currently manual testing is the integration test approach

**E2E Tests:**
- Not implemented
- CLI is tested manually by users

## Common Patterns

**Simple Assertion Testing:**
```rust
#[test]
fn test_branch_from_str() {
    assert_eq!(Branch::from_str("stable").unwrap(), Branch::Stable);
    assert_eq!(Branch::from_str("experimental").unwrap(), Branch::Experimental);
    assert_eq!(Branch::from_str("").unwrap(), Branch::Stable);
    assert_eq!(Branch::from_str("public").unwrap(), Branch::Stable);
    assert!(Branch::from_str("unknown").is_err());
}
```

**Error Testing:**
```rust
#[test]
fn test_branch_from_str() {
    // Test error case
    assert!(Branch::from_str("unknown").is_err());
}
```

**File I/O Testing:**
```rust
#[test]
fn test_parse_save_header() {
    // Create temp directory and file
    let dir = std::env::temp_dir().join("satis_switcher_test_saves");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.sav");

    // Populate with test data
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(&data).unwrap();

    // Test function
    let result = parse_save_header(&path).unwrap();
    assert_eq!(result.value, expected);

    // Cleanup
    fs::remove_file(&path).ok();
    fs::remove_dir(&dir).ok();
}
```

**Content Verification Testing:**
```rust
#[test]
fn test_set_betakey_in_acf() {
    let result = set_betakey_in_acf(SAMPLE_ACF, "");
    assert!(result.contains("\"betakey\"\t\t\"\""));
    assert!(!result.contains("\"betakey\"\t\t\"experimental\""));
}
```

## Test Attributes

**Test Attribute:**
```rust
#[test]
fn test_function() { ... }
```

**Conditional Compilation:**
```rust
#[cfg(test)]
mod tests { ... }
```

**Module Imports:**
```rust
use super::*;  // Import all from parent module
use std::io::Write;  // Import specific items for tests
```

## Running Tests

**All tests:**
```bash
cargo test
```

**Specific test:**
```bash
cargo test test_parse_vdf_flat
```

**Watch mode (requires cargo-watch):**
```bash
cargo watch -x test
```

## Current Test Coverage

**Tested modules:**
- `src/core/steam.rs` - 4 tests covering VDF parsing, branch detection, ACF modification
- `src/core/saves.rs` - 1 test covering save header binary parsing

**Untested modules:**
- `src/commands/*` - No unit tests (CLI layer tested manually)
- `src/core/backup.rs` - No tests (file operations tested manually)
- `src/core/profiles.rs` - No tests (JSON operations tested manually)
- `src/config/*` - Placeholder module, no tests needed

---

*Testing analysis: 2026-03-19*

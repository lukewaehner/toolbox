# Testing Patterns

**Analysis Date:** 2026-02-22

## Test Framework

**Runner:**
- Rust built-in test framework (`#[test]` attribute)
- Standard `cargo test` command with default test runner
- No external testing framework like `criterion` or `proptest` detected

**Assertion Library:**
- Standard Rust assertions: `assert_eq!()`, `assert!()`, `panic!()`
- No external assertion library used

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test -- --nocapture  # Run tests showing output
cargo test -- --test-threads=1  # Run tests sequentially
```

## Test File Organization

**Location:**
- Co-located with source code using `#[cfg(test)]` module pattern
- Test modules placed at bottom of implementation files
- No separate `tests/` directory currently used

**Naming:**
- Test modules use `mod tests` convention
- Individual tests prefixed with `test_` naming pattern
- Examples: `test_encrypt_decrypt()` in `src/password_manager.rs`

**Structure:**
```
src/
├── password_manager.rs      # Contains #[cfg(test)] mod tests
├── modules/
│   └── password_manager/
│       └── model/mod.rs     # Contains #[cfg(test)] mod tests
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        // Test implementation
    }
}
```

**Patterns:**
- Setup: Uses `super::*` to import parent module items for testing
- Assertion: Uses standard Rust `assert_eq!()` macro
- Cleanup: Implicit through test function scope

**Test Examples from `src/password_manager.rs`:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original_data = b"Test data for encryption";

        // Encrypt the data
        let encrypted = match encrypt(original_data) {
            Ok(data) => data,
            Err(e) => panic!("Encryption failed with error: {}", e),
        };

        // Decrypt the data
        let decrypted = match decrypt(&encrypted) {
            Ok(data) => data,
            Err(e) => panic!("Decryption failed with error: {}", e),
        };

        assert_eq!(original_data.to_vec(), decrypted);
    }
}
```

## Mocking

**Framework:** No explicit mocking framework used

**Patterns:**
- Unit tests work directly with actual implementations
- No mock struct patterns detected
- Dependencies handled through composition, not trait-based mocking

**What to Mock:**
- Currently not using mocks; prefer testing implementations directly
- For IO-heavy operations, consider adding integration tests if needed

**What NOT to Mock:**
- Core business logic tested directly
- Encryption/decryption tested with actual cryptographic operations
- Password storage tested with actual file I/O

## Fixtures and Factories

**Test Data:**
- Inline test data created within test functions
- Example from `src/password_manager.rs`:
```rust
let original_data = b"Test data for encryption";
```
- No centralized fixture library detected

**Location:**
- Test-specific data created within `#[cfg(test)]` modules
- Data factories not used; direct construction preferred
- Fixtures kept close to tests for clarity

## Coverage

**Requirements:** No coverage requirements currently enforced

**View Coverage:**
```bash
cargo tarpaulin              # If installed
cargo llvm-cov              # If installed
```

**Current Test Locations:**
- `src/password_manager.rs` - Encryption/decryption tests
- `src/modules/password_manager/model/mod.rs` - Model tests

## Test Types

**Unit Tests:**
- Scope: Test individual functions and core logic
- Approach: Direct testing of public functions
- Example: `test_encrypt_decrypt()` tests encryption and decryption roundtrip
- Execution: Fast, no external dependencies

**Integration Tests:**
- Scope: Not extensively used yet
- Approach: Could test complete workflows (e.g., save and retrieve passwords)
- Opportunity: Add integration tests for task scheduler with file I/O

**E2E Tests:**
- Framework: Not used
- Rationale: UI-based application; would require separate test harness
- Alternative: Manual testing or integration tests for core logic

## Testing Opportunities

**Current Gaps:**
1. **Network Operations** - `src/network_tools.rs`: No tests for `ping()` or speed test functions
   - Challenge: External command execution and network calls
   - Approach: Mock command output or use test servers

2. **Task Scheduler** - `src/task_scheduler.rs`: No unit tests detected
   - Coverage needed: Task creation, reminder scheduling, email configuration
   - Approach: Add tests for core scheduling logic without network operations

3. **System Utilities** - `src/system_utilities.rs`: No tests detected
   - Coverage needed: System monitoring snapshot creation
   - Approach: Unit tests for data structure creation and calculations

4. **Configuration Management** - `src/config.rs`: No tests detected
   - Coverage needed: Config loading, validation, serialization
   - Approach: Temporary file fixtures for file I/O testing

5. **Logger** - `src/logger.rs`: No tests detected
   - Coverage needed: Log level filtering, message formatting
   - Approach: Capture output or use temporary log files

## Async Testing

**Not used:** No async/await patterns detected in current tests

**If needed:**
```rust
#[tokio::test]
async fn test_async_operation() {
    // Async test code
}
```

## Error Testing

**Current Pattern:**
```rust
match decrypt(&encrypted) {
    Ok(data) => data,
    Err(e) => panic!("Decryption failed with error: {}", e),
}
```

**Recommended Pattern for Error Cases:**
```rust
#[test]
#[should_panic]
fn test_invalid_decryption() {
    // Test code that should panic
}
```

**Alternative Pattern:**
```rust
#[test]
fn test_error_handling() {
    let result = some_fallible_function();
    assert!(result.is_err());
    // Or check specific error type
}
```

## Test Configuration

**Cargo.toml Testing Setup:**
```toml
[dependencies]
# Core dependencies used in tests
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
aes = "0.8"
# ... other dependencies
```

**No special test-only dependencies** currently configured.

## Common Test Patterns to Adopt

**For Cryptography:**
```rust
#[test]
fn test_encrypt_decrypt_roundtrip() {
    let original = b"test data";
    let encrypted = encrypt(original).expect("encryption failed");
    let decrypted = decrypt(&encrypted).expect("decryption failed");
    assert_eq!(original.to_vec(), decrypted);
}
```

**For File I/O:**
```rust
#[test]
fn test_file_operations() {
    let temp_path = "/tmp/test_file.json";
    // Write test
    fs::write(temp_path, "test").expect("write failed");
    // Read test
    let content = fs::read(temp_path).expect("read failed");
    assert_eq!(content, b"test");
    // Cleanup
    let _ = fs::remove_file(temp_path);
}
```

**For Error Handling:**
```rust
#[test]
fn test_handles_invalid_input() {
    let result = process_data(invalid_input);
    assert!(result.is_err());
}
```

## CI/CD Testing

**Current Setup:** Not detected

**Recommended:** Add GitHub Actions workflow:
```yaml
name: tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --verbose
```

---

*Testing analysis: 2026-02-22*

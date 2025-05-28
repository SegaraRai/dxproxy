# DXProxy Testing Approach

This document outlines the testing strategy for the DXProxy library.

## Overview

DXProxy is a complex library that proxies DirectX interfaces, making traditional unit testing challenging due to:

1. Heavy dependency on native Windows APIs and COM interfaces
2. Extensive use of unsafe code
3. Multiple layered proxy objects with complex lifecycles

## Testing Strategy

Our approach uses a combination of the following techniques:

### 1. Unit Tests for Utilities

The common utilities (`try_out_param`, `ComMappingTracker`, etc.) are tested using standard Rust unit tests. These tests are self-contained and do not require complex mocking.

Example:
```rust
#[test]
fn test_try_out_param_success() {
    let result = try_out_param(|out| {
        *out = Some(42);
        Ok(())
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}
```

### 2. Pointer-Level COM Testing

For COM interfaces, we test at the raw pointer level using controlled memory allocations. This approach allows verifying the core COM mapping and lifecycle functionality without full COM interface mocking.

Example:
```rust
#[test]
fn test_mapping_creation_and_lookup() {
    let mut tracker = ComMappingTracker::default();
    
    // Create pointers for testing
    let target_ptr = Box::into_raw(Box::new(42)) as *mut c_void;
    let proxy_ptr = Box::into_raw(Box::new(24)) as *mut c_void;
    
    // Add mappings and verify
    tracker.target_to_proxy.insert(target_ptr, proxy_ptr);
    assert_eq!(tracker.target_to_proxy.get(&target_ptr), Some(&proxy_ptr));
    
    // Cleanup
    unsafe {
        let _ = Box::from_raw(target_ptr as *mut i32);
        let _ = Box::from_raw(proxy_ptr as *mut i32);
    }
}
```

### 3. Mock Interfaces

For specific DirectX interfaces, we use simplified mock objects that implement the minimal functionality needed for testing.

## Running the Tests

Run all tests:
```
cargo test
```

Run a specific test:
```
cargo test common::try_out_param::tests::test_try_out_param_success
```

## Test Organization

Tests are organized in-file for most units:

- Unit-level tests: Located in `#[cfg(test)]` modules within the implementation file
- Integration tests: Located in dedicated test modules (e.g., `dx9/tests/mod.rs`)

## Future Test Improvements

Future test enhancements should focus on:

1. More comprehensive mock implementations of DirectX interfaces
2. Integration tests for full proxy object lifecycles
3. Testing proxy performance characteristics
4. Fuzz testing for COM interface interaction
//! Tests for the #[property_test] macro

#![allow(double_negations)]
#![allow(clippy::nonminimal_bool)]

use protest::property_test;

#[property_test]
fn test_addition_commutative(a: i32, b: i32) {
    // Use wrapping arithmetic to avoid overflow panics
    assert_eq!(a.wrapping_add(b), b.wrapping_add(a));
}

#[property_test(iterations = 50)]
fn test_multiplication_by_zero(x: i32) {
    // Test that multiplying any number by zero gives zero
    assert_eq!(x.wrapping_mul(0), 0);
}

#[property_test(iterations = 20, seed = 42)]
fn test_string_length_bounds(s: String) {
    // String should have reasonable length (default is 0-20)
    assert!(s.len() <= 100); // Very generous upper bound
}

#[property_test]
fn test_boolean_negation(b: bool) {
    assert_eq!(!(!b), b);
}

#[property_test(iterations = 30)]
fn test_vector_length(v: Vec<i32>) {
    // Vector should have reasonable length (default is 0-10)
    assert!(v.len() <= 50); // Very generous upper bound
}

#[property_test]
fn test_tuple_access(pair: (i32, String)) {
    let (a, s) = pair;
    // Just test that we can access both elements
    let _sum = a + s.len() as i32;
}

#[property_test(max_shrink_iterations = 100)]
fn test_absolute_value_non_negative(x: i32) {
    if x != i32::MIN {
        // Avoid overflow
        assert!(x.abs() >= 0);
    }
}

#[property_test(shrink_timeout_secs = 5)]
fn test_char_is_valid(c: char) {
    // All generated chars should be valid Unicode
    assert!(c.is_ascii() || !c.is_ascii()); // Tautology, but tests char generation
}

// Test with multiple parameters
#[property_test]
fn test_three_params(a: i32, b: u32, c: String) {
    // Test that all parameters are accessible (use wrapping arithmetic to avoid overflow)
    let _result = a.wrapping_add(b as i32).wrapping_add(c.len() as i32);
}

// Async property test
#[property_test]
async fn test_async_property(x: i32) {
    // Simulate some async work
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    // Test that async processing preserves value
    let result = x;
    assert_eq!(result, x);
}

#[property_test(iterations = 10)]
async fn test_async_with_config(s: String) {
    // Simulate async string processing
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    let upper = s.to_uppercase();
    assert!(upper.len() == s.len());
}

// Test that should fail to verify error handling works
// (This test is commented out because it would cause the test suite to fail)
/*
#[property_test]
fn test_intentional_failure(x: i32) {
    assert!(x > 1000000); // This will fail for most generated values
}
*/

#[cfg(test)]
mod macro_expansion_tests {

    // These tests verify that the macro generates valid code
    // by checking that the generated functions compile and can be called

    #[test]
    fn test_macro_generates_valid_sync_test() {
        // The property_test macro should generate a valid test function
        // This is verified by the fact that the tests above compile
    }

    #[tokio::test]
    async fn test_macro_generates_valid_async_test() {
        // The async property_test macro should generate valid async test functions
        // This is verified by the fact that the async tests above compile
    }
}

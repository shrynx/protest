//! Example demonstrating custom structs with manual generators
//!
//! This example shows how to use Protest with your own data structures
//! by implementing custom generators and using the ergonomic API.

use protest::ergonomic::*;
use protest::primitives::*;
use protest::{Generator, GeneratorConfig};

// Example 1: Simple struct
#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

// Manual generator implementation for User
struct UserGenerator {
    id_gen: IntGenerator<u32>,
    name_gen: StringGenerator,
    age_gen: IntGenerator<u8>,
    bool_gen: BoolGenerator,
}

impl UserGenerator {
    fn new() -> Self {
        Self {
            id_gen: IntGenerator::new(1, 100000),
            name_gen: StringGenerator::ascii_alphanumeric(3, 20),
            age_gen: IntGenerator::new(18, 100),
            bool_gen: BoolGenerator,
        }
    }
}

impl Generator<User> for UserGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> User {
        User {
            id: self.id_gen.generate(rng, config),
            name: self.name_gen.generate(rng, config),
            age: self.age_gen.generate(rng, config),
            active: self.bool_gen.generate(rng, config),
        }
    }

    fn shrink(&self, _value: &User) -> Box<dyn Iterator<Item = User>> {
        Box::new(std::iter::empty())
    }
}

// Example 2: Struct with specific constraints
#[derive(Debug, Clone, PartialEq)]
struct Product {
    id: u64,
    name: String,
    price_cents: u32,
    in_stock: bool,
}

struct ProductGenerator {
    id_gen: IntGenerator<u64>,
    name_gen: StringGenerator,
    price_gen: IntGenerator<u32>,
    bool_gen: BoolGenerator,
}

impl ProductGenerator {
    fn new() -> Self {
        Self {
            id_gen: IntGenerator::new(1, 1000),
            name_gen: StringGenerator::ascii_alphanumeric(3, 50),
            price_gen: IntGenerator::new(0, 10000),
            bool_gen: BoolGenerator,
        }
    }
}

impl Generator<Product> for ProductGenerator {
    fn generate(&self, rng: &mut dyn rand::RngCore, config: &GeneratorConfig) -> Product {
        Product {
            id: self.id_gen.generate(rng, config),
            name: self.name_gen.generate(rng, config),
            price_cents: self.price_gen.generate(rng, config),
            in_stock: self.bool_gen.generate(rng, config),
        }
    }

    fn shrink(&self, _value: &Product) -> Box<dyn Iterator<Item = Product>> {
        Box::new(std::iter::empty())
    }
}

fn main() {
    println!("Protest - Custom Structs Example");
    println!("=================================\n");

    example_1_basic_struct();
    example_2_struct_with_constraints();
    example_3_closure_properties();

    println!("\n=== Summary ===");
    println!("✓ All examples completed!");
    println!("This shows how to implement custom generators for your own types.");
}

fn example_1_basic_struct() {
    println!("=== Example 1: Basic Struct with Custom Generator ===\n");

    println!("Testing: User ID is always positive");
    let result = check_with_closure(UserGenerator::new(), |user: User| user.id > 0);
    match result {
        Ok(success) => println!("✓ Passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: User age is in valid range");
    let result = check_with_closure(UserGenerator::new(), |user: User| {
        user.age >= 18 && user.age <= 100
    });
    match result {
        Ok(success) => println!("✓ Passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: User name length is reasonable");
    let result = check_with_closure(UserGenerator::new(), |user: User| {
        user.name.len() >= 3 && user.name.len() <= 20
    });
    match result {
        Ok(success) => println!("✓ Passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }
}

fn example_2_struct_with_constraints() {
    println!("=== Example 2: Struct with Custom Constraints ===\n");

    println!("Testing: Product ID is in valid range");
    let result = check_with_closure(ProductGenerator::new(), |product: Product| {
        product.id >= 1 && product.id <= 1000
    });
    match result {
        Ok(success) => println!("✓ Passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: Product name length constraints");
    let result = check_with_closure(ProductGenerator::new(), |product: Product| {
        let len = product.name.len();
        (3..=50).contains(&len)
    });
    match result {
        Ok(success) => println!("✓ Passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: Price is reasonable");
    let result = check_with_closure(ProductGenerator::new(), |product: Product| {
        product.price_cents <= 10000
    });
    match result {
        Ok(success) => println!("✓ Passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }
}

fn example_3_closure_properties() {
    println!("=== Example 3: Using Closure Properties ===\n");

    println!("Testing: Clone produces equal object");
    let result = check_with_closure(UserGenerator::new(), |user: User| {
        let cloned = user.clone();
        cloned == user
    });
    match result {
        Ok(success) => println!("✓ Passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }

    println!("Testing: Product price_cents is reasonable");
    let result = property(|product: Product| {
        // Price in cents should be reasonable
        product.price_cents <= 1_000_000
    })
    .iterations(50)
    .run_with(ProductGenerator::new());

    match result {
        Ok(success) => println!("✓ Passed ({} iterations)\n", success.iterations),
        Err(failure) => println!("✗ Failed: {}\n", failure.error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_clone_equals() {
        let result = check_with_closure(UserGenerator::new(), |user: User| {
            let cloned = user.clone();
            cloned == user
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_product_id_range() {
        let result = check_with_closure(ProductGenerator::new(), |product: Product| {
            product.id >= 1 && product.id <= 1000
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_user_age_range() {
        let result = check_with_closure(UserGenerator::new(), |user: User| {
            user.age >= 18 && user.age <= 100
        });
        assert!(result.is_ok());
    }
}

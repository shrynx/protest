//! Minimal example: Vector double reverse property

use protest::{
    Property, PropertyError, check,
    primitives::{IntGenerator, VecGenerator},
};

struct DoubleReverseProperty;

impl Property<Vec<i32>> for DoubleReverseProperty {
    type Output = ();

    fn test(&self, mut input: Vec<i32>) -> Result<Self::Output, PropertyError> {
        let original = input.clone();

        // Reverse twice
        input.reverse();
        input.reverse();

        // Should equal original
        if input == original {
            Ok(())
        } else {
            Err(PropertyError::property_failed("Double reverse failed"))
        }
    }
}

fn main() {
    let generator = VecGenerator::new(IntGenerator::new(0, 100), 0, 10);

    match check(generator, DoubleReverseProperty) {
        Ok(success) => println!("✓ Property passed! ({} tests)", success.iterations),
        Err(failure) => println!("✗ Failed: {:?}", failure.original_input),
    }
}

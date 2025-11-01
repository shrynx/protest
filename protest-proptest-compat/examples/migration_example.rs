//! Migration Example: From Proptest to Protest
//!
//! This example shows side-by-side comparisons of proptest code
//! and its Protest equivalent.
//!
//! Run with: cargo run --example migration_example

fn main() {
    println!("=== Proptest to Protest Migration Examples ===\n");

    println!("Example 1: Simple Integer Properties");
    println!("--------------------------------------");
    println!("BEFORE (Proptest):");
    println!(
        r#"
proptest! {{
    #[test]
    fn test_addition(a in 0..100i32, b in 0..100i32) {{
        assert!(a + b >= a);
        assert!(a + b >= b);
    }}
}}
"#
    );

    println!("AFTER (Protest):");
    println!(
        r#"
#[test]
fn test_addition() {{
    property!(generator!(i32, 0, 100), |(a, b)| {{
        a + b >= a && a + b >= b
    }});
}}
"#
    );

    println!("\nExample 2: Vector Properties");
    println!("-----------------------------");
    println!("BEFORE (Proptest):");
    println!(
        r#"
proptest! {{
    #[test]
    fn reverse_twice_is_identity(v: Vec<i32>) {{
        let mut v2 = v.clone();
        v2.reverse();
        v2.reverse();
        assert_eq!(v, v2);
    }}
}}
"#
    );

    println!("AFTER (Protest - Ergonomic API):");
    println!(
        r#"
#[test]
fn reverse_twice_is_identity() {{
    property(|mut v: Vec<i32>| {{
        let original = v.clone();
        v.reverse();
        v.reverse();
        v == original
    }})
    .iterations(100)
    .run()
    .expect("property should hold");
}}
"#
    );

    println!("\nExample 3: Using Migration Helpers");
    println!("-----------------------------------");
    println!("Using protest-proptest-compat helpers:");
    println!(
        r#"
use protest_proptest_compat::{{range_to_generator, vec_generator}};
use protest::primitives::IntGenerator;

// Range generator
let int_gen = range_to_generator(0, 100);

// Vector generator
let vec_gen = vec_generator(IntGenerator::new(0, 10), 5, 10);
"#
    );

    println!("\nExample 4: Custom Generators");
    println!("-----------------------------");
    println!("BEFORE (Proptest Strategy):");
    println!(
        r#"
use proptest::strategy::{{Strategy, Just}};

fn user_strategy() -> impl Strategy<Value = User> {{
    (0..1000u32, "[a-z]{{5,10}}").prop_map(|(id, name)| {{
        User {{ id, name }}
    }})
}}
"#
    );

    println!("AFTER (Protest Generator):");
    println!(
        r#"
use protest::{{Generator, config::GeneratorConfig}};

struct UserGenerator;
impl Generator<User> for UserGenerator {{
    fn generate(&self, rng: &mut dyn RngCore, config: &GeneratorConfig) -> User {{
        let id_gen = IntGenerator::new(0, 1000);
        let name_gen = StringGenerator::new(5, 10);

        User {{
            id: id_gen.generate(rng, config),
            name: name_gen.generate(rng, config),
        }}
    }}
}}
"#
    );

    println!("\n✅ Migration Tips:");
    println!("   1. Replace imports: proptest::prelude::* → protest::*");
    println!("   2. Remove proptest! macro wrapper");
    println!("   3. Use property! or property() inside #[test]");
    println!("   4. Convert strategies to generators using helpers");
    println!("   5. Use protest-proptest-compat for common patterns");
    println!("\n📚 See the README for complete migration guide!");
}

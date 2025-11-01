//! Example demonstrating weight-based operation generation
//!
//! This example shows how to use the WeightedGenerator to generate
//! operations according to their specified weights, creating more
//! realistic test scenarios.

use protest_stateful::{
    Operation as OperationDerive,
    dsl::StatefulTest,
    operations::{Operation, OperationSequence, WeightedGenerator},
};
use rand::{SeedableRng, thread_rng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

// Example 1: Bank Account Operations with Realistic Weights
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "BankAccount")]
enum BankOp {
    /// Deposits are common (weight 10)
    #[execute("state.deposit(*field_0)")]
    #[weight(10)]
    Deposit(u32),

    /// Small withdrawals are fairly common (weight 7)
    #[execute("state.withdraw(*field_0)")]
    #[precondition("state.balance >= *field_0")]
    #[weight(7)]
    Withdraw(u32),

    /// Balance checks are very common (weight 15)
    #[execute("let _ = state.balance")]
    #[weight(15)]
    CheckBalance,

    /// Transfers are less common (weight 3)
    #[execute("state.transfer(*field_0)")]
    #[precondition("state.balance >= *field_0")]
    #[weight(3)]
    Transfer(u32),

    /// Account closures are rare (weight 1)
    #[execute("state.close()")]
    #[weight(1)]
    Close,
}

#[derive(Debug, Clone)]
struct BankAccount {
    balance: u32,
    closed: bool,
}

impl BankAccount {
    fn new() -> Self {
        Self {
            balance: 1000,
            closed: false,
        }
    }

    fn deposit(&mut self, amount: u32) {
        if !self.closed {
            self.balance += amount;
        }
    }

    fn withdraw(&mut self, amount: u32) {
        if !self.closed && self.balance >= amount {
            self.balance -= amount;
        }
    }

    fn transfer(&mut self, amount: u32) {
        if !self.closed && self.balance >= amount {
            self.balance -= amount;
        }
    }

    fn close(&mut self) {
        self.closed = true;
    }
}

// Example 2: Cache Operations with Performance-Based Weights
#[derive(Debug, Clone, OperationDerive)]
#[operation(state = "HashMap<String, String>")]
enum CacheOp {
    /// Reads are very frequent (weight 20)
    #[execute("let _ = state.get(field_0)")]
    #[weight(20)]
    Get(String),

    /// Writes are less frequent than reads (weight 5)
    #[execute("state.insert(field_0.clone(), field_1.clone())")]
    #[weight(5)]
    Put(String, String),

    /// Deletes are rare (weight 2)
    #[execute("state.remove(field_0)")]
    #[weight(2)]
    Delete(String),

    /// Cache clears are very rare (weight 1)
    #[execute("state.clear()")]
    #[weight(1)]
    Clear,
}

fn main() {
    println!("=== Weight-Based Operation Generation Demo ===\n");

    // Example 1: Bank Account Operations
    println!("Example 1: Bank Account with Realistic Operation Frequencies");
    println!("{}", "=".repeat(70));

    let bank_variants = vec![
        BankOp::Deposit(10),
        BankOp::Withdraw(5),
        BankOp::CheckBalance,
        BankOp::Transfer(20),
        BankOp::Close,
    ];

    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut bank_generator = WeightedGenerator::new(bank_variants, rng);

    // Show weight distribution
    println!("\nOperation Weight Distribution:");
    let dist = bank_generator.weight_distribution();
    for (idx, weight, pct) in dist {
        let op_name = match idx {
            0 => "Deposit",
            1 => "Withdraw",
            2 => "CheckBalance",
            3 => "Transfer",
            4 => "Close",
            _ => "Unknown",
        };
        println!("  {:<15} weight={:<2}  ({:.1}%)", op_name, weight, pct);
    }

    println!("\nGenerating 20 weighted operations...");
    let bank_ops = bank_generator.generate(20);

    let mut account = BankAccount::new();
    println!("\nInitial balance: ${}", account.balance);
    println!("\nExecuting operations:");

    for (idx, op) in bank_ops.iter().enumerate() {
        println!("  {}: {:?}", idx, op);
        op.execute(&mut account);
    }

    println!("\nFinal balance: ${}", account.balance);
    println!("Account closed: {}", account.closed);

    // Example 2: Cache Operations with Read-Heavy Workload
    println!("\n\nExample 2: Cache with Read-Heavy Workload");
    println!("{}", "=".repeat(70));

    let cache_variants = vec![
        CacheOp::Get("key1".to_string()),
        CacheOp::Put("key1".to_string(), "value1".to_string()),
        CacheOp::Delete("key1".to_string()),
        CacheOp::Clear,
    ];

    let rng = ChaCha8Rng::seed_from_u64(100);
    let mut cache_generator = WeightedGenerator::new(cache_variants, rng);

    println!("\nOperation Weight Distribution:");
    let dist = cache_generator.weight_distribution();
    for (idx, weight, pct) in dist {
        let op_name = match idx {
            0 => "Get (read)",
            1 => "Put (write)",
            2 => "Delete",
            3 => "Clear",
            _ => "Unknown",
        };
        println!("  {:<15} weight={:<2}  ({:.1}%)", op_name, weight, pct);
    }

    println!("\nGenerating 30 operations (note read-heavy workload)...");
    let cache_ops = cache_generator.generate(30);

    let mut cache = HashMap::new();
    println!("\nOperation sequence:");

    let mut get_count = 0;
    let mut put_count = 0;
    let mut delete_count = 0;
    let mut clear_count = 0;

    for (idx, op) in cache_ops.iter().enumerate() {
        match op {
            CacheOp::Get(_) => {
                get_count += 1;
                if idx < 10 {
                    println!("  {}: Get", idx);
                }
            }
            CacheOp::Put(k, v) => {
                put_count += 1;
                if idx < 10 {
                    println!("  {}: Put({}, {})", idx, k, v);
                }
            }
            CacheOp::Delete(_) => {
                delete_count += 1;
                if idx < 10 {
                    println!("  {}: Delete", idx);
                }
            }
            CacheOp::Clear => {
                clear_count += 1;
                if idx < 10 {
                    println!("  {}: Clear", idx);
                }
            }
        }
        op.execute(&mut cache);
    }

    println!("  ... (showing first 10 operations)");

    println!("\nOperation Statistics:");
    println!(
        "  Get (reads):    {} ({:.1}%)",
        get_count,
        (get_count as f64 / 30.0) * 100.0
    );
    println!(
        "  Put (writes):   {} ({:.1}%)",
        put_count,
        (put_count as f64 / 30.0) * 100.0
    );
    println!(
        "  Delete:         {} ({:.1}%)",
        delete_count,
        (delete_count as f64 / 30.0) * 100.0
    );
    println!(
        "  Clear:          {} ({:.1}%)",
        clear_count,
        (clear_count as f64 / 30.0) * 100.0
    );

    println!("\nNote: Get operations appear ~4x more frequently than Put,");
    println!("      reflecting a typical read-heavy cache workload!");

    // Example 3: Using with StatefulTest
    println!("\n\nExample 3: Integration with StatefulTest");
    println!("{}", "=".repeat(70));

    #[derive(Debug, Clone, OperationDerive)]
    #[operation(state = "Vec<i32>")]
    enum StackOp {
        #[execute("state.push(*field_0)")]
        #[weight(5)]
        Push(i32),

        #[execute("state.pop()")]
        #[precondition("!state.is_empty()")]
        #[weight(3)]
        Pop,

        #[execute("state.clear()")]
        #[weight(1)]
        Clear,
    }

    let test = StatefulTest::new(vec![]).invariant("bounded", |s: &Vec<i32>| s.len() <= 100);

    // Generate weighted sequence
    let stack_variants = vec![StackOp::Push(42), StackOp::Pop, StackOp::Clear];
    let mut stack_generator = WeightedGenerator::new(stack_variants, thread_rng());
    let ops = stack_generator.generate(15);

    println!(
        "\nGenerated {} weighted operations for stack test",
        ops.len()
    );

    let mut seq = OperationSequence::new();
    for op in ops {
        seq.push(op);
    }

    let result = test.run(&seq);
    println!("Test result: {:?}", result);

    println!("\n✓ Weight-based generation creates more realistic test scenarios!");
    println!("  - Common operations appear more frequently");
    println!("  - Rare operations still tested but less often");
    println!("  - Reflects real-world usage patterns");
}

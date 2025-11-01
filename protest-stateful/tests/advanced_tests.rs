use protest_stateful::prelude::*;
use protest_stateful::temporal::*;

// Complex state for advanced testing
#[derive(Debug, Clone, PartialEq)]
struct BankAccount {
    balance: i32,
    transaction_count: usize,
    is_locked: bool,
}

#[derive(Debug, Clone)]
enum BankOp {
    Deposit(i32),
    Withdraw(i32),
    Lock,
    Unlock,
}

impl Operation for BankOp {
    type State = BankAccount;

    fn execute(&self, state: &mut Self::State) {
        match self {
            BankOp::Deposit(amount) => {
                state.balance += amount;
                state.transaction_count += 1;
            }
            BankOp::Withdraw(amount) => {
                state.balance -= amount;
                state.transaction_count += 1;
            }
            BankOp::Lock => {
                state.is_locked = true;
            }
            BankOp::Unlock => {
                state.is_locked = false;
            }
        }
    }

    fn precondition(&self, state: &Self::State) -> bool {
        match self {
            BankOp::Deposit(_) | BankOp::Withdraw(_) => !state.is_locked,
            BankOp::Lock => !state.is_locked,
            BankOp::Unlock => state.is_locked,
        }
    }

    fn description(&self) -> String {
        match self {
            BankOp::Deposit(amt) => format!("Deposit({})", amt),
            BankOp::Withdraw(amt) => format!("Withdraw({})", amt),
            BankOp::Lock => "Lock".to_string(),
            BankOp::Unlock => "Unlock".to_string(),
        }
    }
}

#[test]
fn test_complex_invariants() {
    let test = StatefulTest::new(BankAccount {
        balance: 0,
        transaction_count: 0,
        is_locked: false,
    })
    .invariant("balance_non_negative", |s: &BankAccount| s.balance >= 0)
    .invariant("transaction_count_reasonable", |s: &BankAccount| {
        s.transaction_count < 1000
    });

    let mut seq = OperationSequence::new();
    seq.push(BankOp::Deposit(100));
    seq.push(BankOp::Withdraw(50));
    seq.push(BankOp::Lock);
    seq.push(BankOp::Unlock);
    seq.push(BankOp::Deposit(25));

    let result = test.run(&seq);
    assert!(result.is_ok());
    let final_state = result.unwrap();
    assert_eq!(final_state.balance, 75);
    assert_eq!(final_state.transaction_count, 3);
}

#[test]
fn test_precondition_enforcement() {
    let test = StatefulTest::new(BankAccount {
        balance: 100,
        transaction_count: 0,
        is_locked: false,
    });

    // Lock then try to deposit (should fail precondition)
    let mut seq = OperationSequence::new();
    seq.push(BankOp::Lock);
    seq.push(BankOp::Deposit(50)); // Should fail

    let result = test.run(&seq);
    assert!(result.is_err());
}

#[test]
fn test_sequence_shrinking_effectiveness() {
    let mut seq = OperationSequence::new();
    for _ in 0..10 {
        seq.push(BankOp::Deposit(10));
    }
    for _ in 0..5 {
        seq.push(BankOp::Withdraw(5));
    }

    let shrunk = seq.shrink();
    assert!(!shrunk.is_empty());

    // All shrunk sequences should be strictly smaller
    for s in shrunk {
        assert!(s.len() < seq.len());
        assert!(!s.is_empty());
    }
}

#[test]
fn test_execution_trace_detail() {
    let test = StatefulTest::new(BankAccount {
        balance: 0,
        transaction_count: 0,
        is_locked: false,
    });

    let mut seq = OperationSequence::new();
    seq.push(BankOp::Deposit(100));
    seq.push(BankOp::Withdraw(30));
    seq.push(BankOp::Deposit(50));

    let trace = test.run_with_trace(&seq).unwrap();

    assert_eq!(trace.initial_state().balance, 0);
    assert_eq!(trace.steps().len(), 3);

    let final_state = trace.final_state().unwrap();
    assert_eq!(final_state.balance, 120);
    assert_eq!(final_state.transaction_count, 3);
}

// Model-based testing for bank account
#[derive(Debug, Clone)]
struct SimpleBankModel {
    balance: i32,
}

impl Model for SimpleBankModel {
    type SystemState = BankAccount;
    type Operation = BankOp;

    fn execute_model(&mut self, op: &Self::Operation) {
        match op {
            BankOp::Deposit(amt) => self.balance += amt,
            BankOp::Withdraw(amt) => self.balance -= amt,
            BankOp::Lock | BankOp::Unlock => {} // Model doesn't track lock state
        }
    }

    fn matches(&self, system: &Self::SystemState) -> bool {
        self.balance == system.balance
    }
}

#[test]
fn test_model_based_bank() {
    let model = SimpleBankModel { balance: 0 };
    let system = BankAccount {
        balance: 0,
        transaction_count: 0,
        is_locked: false,
    };

    let test = ModelBasedTest::new(model, system);

    let mut seq = OperationSequence::new();
    seq.push(BankOp::Deposit(100));
    seq.push(BankOp::Withdraw(25));
    seq.push(BankOp::Deposit(50));

    let result = test.run(&seq);
    assert!(result.is_ok());
}

#[test]
fn test_model_detects_mismatch() {
    // Intentionally broken system for testing
    #[derive(Debug, Clone)]
    struct BrokenAccount {
        balance: i32,
    }

    #[derive(Debug, Clone)]
    enum BrokenOp {
        Deposit(i32),
    }

    impl Operation for BrokenOp {
        type State = BrokenAccount;

        fn execute(&self, state: &mut Self::State) {
            match self {
                BrokenOp::Deposit(amt) => {
                    // Bug: deposits twice!
                    state.balance += amt * 2;
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    struct CorrectModel {
        balance: i32,
    }

    impl Model for CorrectModel {
        type SystemState = BrokenAccount;
        type Operation = BrokenOp;

        fn execute_model(&mut self, op: &Self::Operation) {
            match op {
                BrokenOp::Deposit(amt) => self.balance += amt,
            }
        }

        fn matches(&self, system: &Self::SystemState) -> bool {
            self.balance == system.balance
        }
    }

    let model = CorrectModel { balance: 0 };
    let system = BrokenAccount { balance: 0 };
    let test = ModelBasedTest::new(model, system);

    let mut seq = OperationSequence::new();
    seq.push(BrokenOp::Deposit(100));

    let result = test.run(&seq);
    assert!(result.is_err()); // Should detect the bug
}

// Temporal properties tests
#[test]
fn test_temporal_eventually_with_complex_state() {
    let states = vec![
        BankAccount {
            balance: 0,
            transaction_count: 0,
            is_locked: false,
        },
        BankAccount {
            balance: 50,
            transaction_count: 1,
            is_locked: false,
        },
        BankAccount {
            balance: 100,
            transaction_count: 2,
            is_locked: true,
        },
    ];

    let prop = Eventually::new("becomes_locked", |s: &BankAccount| s.is_locked);
    assert!(prop.check(&states));

    let prop2 = Eventually::new("reaches_200", |s: &BankAccount| s.balance >= 200);
    assert!(!prop2.check(&states));
}

#[test]
fn test_temporal_always_with_violations() {
    let states = vec![
        BankAccount {
            balance: 100,
            transaction_count: 0,
            is_locked: false,
        },
        BankAccount {
            balance: -50, // Violation!
            transaction_count: 1,
            is_locked: false,
        },
        BankAccount {
            balance: 0,
            transaction_count: 2,
            is_locked: false,
        },
    ];

    let prop = Always::new("balance_non_negative", |s: &BankAccount| s.balance >= 0);
    assert!(!prop.check(&states));
}

#[test]
fn test_temporal_leads_to_complex() {
    let states = vec![
        BankAccount {
            balance: 0,
            transaction_count: 0,
            is_locked: false,
        },
        BankAccount {
            balance: 100,
            transaction_count: 1,
            is_locked: false,
        },
        BankAccount {
            balance: 100,
            transaction_count: 1,
            is_locked: true,
        },
    ];

    // Once we have transactions (P), we eventually lock (Q)
    let prop = LeadsTo::new(
        "transaction_leads_to_lock",
        |s: &BankAccount| s.transaction_count > 0,
        |s: &BankAccount| s.is_locked,
    );

    assert!(prop.check(&states));
}

#[test]
fn test_multiple_invariants_simultaneously() {
    let test = StatefulTest::new(BankAccount {
        balance: 0,
        transaction_count: 0,
        is_locked: false,
    })
    .invariant("balance_non_negative", |s: &BankAccount| s.balance >= 0)
    .invariant("reasonable_balance", |s: &BankAccount| s.balance < 1000000)
    .invariant("transaction_count_matches", |s: &BankAccount| {
        s.transaction_count <= 1000
    })
    .invariant("locked_means_no_new_transactions", |s: &BankAccount| {
        if s.is_locked {
            s.transaction_count == 0
        } else {
            true
        }
    });

    let mut seq = OperationSequence::new();
    seq.push(BankOp::Deposit(100));
    seq.push(BankOp::Deposit(200));
    seq.push(BankOp::Withdraw(50));

    let result = test.run(&seq);
    assert!(result.is_ok());
}

#[test]
fn test_empty_sequence() {
    let test = StatefulTest::new(BankAccount {
        balance: 100,
        transaction_count: 0,
        is_locked: false,
    })
    .invariant("balance_positive", |s: &BankAccount| s.balance > 0);

    let seq: OperationSequence<BankOp> = OperationSequence::new();
    let result = test.run(&seq);
    assert!(result.is_ok());
}

#[test]
fn test_single_operation_sequence() {
    let test = StatefulTest::new(BankAccount {
        balance: 0,
        transaction_count: 0,
        is_locked: false,
    });

    let mut seq = OperationSequence::new();
    seq.push(BankOp::Deposit(100));

    let result = test.run(&seq);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().balance, 100);
}

#[test]
fn test_long_operation_sequence() {
    let test = StatefulTest::new(BankAccount {
        balance: 1000,
        transaction_count: 0,
        is_locked: false,
    });

    let mut seq = OperationSequence::new();
    for _ in 0..50 {
        seq.push(BankOp::Deposit(10));
    }
    for _ in 0..25 {
        seq.push(BankOp::Withdraw(5));
    }

    let result = test.run(&seq);
    assert!(result.is_ok());
    let final_state = result.unwrap();
    // 1000 + (50 * 10) - (25 * 5) = 1000 + 500 - 125 = 1375
    assert_eq!(final_state.balance, 1375);
    assert_eq!(final_state.transaction_count, 75);
}

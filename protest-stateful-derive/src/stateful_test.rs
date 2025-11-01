//! Implementation of the stateful_test! macro

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, LitInt, LitStr, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Parsed structure for stateful_test! macro
struct StatefulTestInput {
    name: Ident,
    state_type: Type,
    state_init: Expr,
    operations: Type,
    invariants: Vec<(String, Expr)>,
    config: TestConfig,
}

#[derive(Default)]
struct TestConfig {
    iterations: Option<u32>,
    max_sequence_length: Option<u32>,
    min_sequence_length: Option<u32>,
    seed: Option<u64>,
}

impl Parse for StatefulTestInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut state_type = None;
        let mut state_init = None;
        let mut operations = None;
        let mut invariants = Vec::new();
        let mut config = TestConfig::default();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => {
                    name = Some(input.parse()?);
                }
                "state" => {
                    // Parse: Type = expr
                    state_type = Some(input.parse()?);
                    input.parse::<Token![=]>()?;
                    state_init = Some(input.parse()?);
                }
                "operations" => {
                    operations = Some(input.parse()?);
                }
                "invariants" => {
                    // Parse: { "name" => |state| expr, ... }
                    let content;
                    syn::braced!(content in input);

                    while !content.is_empty() {
                        let name_lit: LitStr = content.parse()?;
                        content.parse::<Token![=>]>()?;
                        let closure: Expr = content.parse()?;

                        invariants.push((name_lit.value(), closure));

                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                "config" => {
                    // Parse: { key: value, ... }
                    let content;
                    syn::braced!(content in input);

                    while !content.is_empty() {
                        let config_key: Ident = content.parse()?;
                        content.parse::<Token![:]>()?;

                        match config_key.to_string().as_str() {
                            "iterations" => {
                                let lit: LitInt = content.parse()?;
                                config.iterations = Some(lit.base10_parse()?);
                            }
                            "max_sequence_length" => {
                                let lit: LitInt = content.parse()?;
                                config.max_sequence_length = Some(lit.base10_parse()?);
                            }
                            "min_sequence_length" => {
                                let lit: LitInt = content.parse()?;
                                config.min_sequence_length = Some(lit.base10_parse()?);
                            }
                            "seed" => {
                                let lit: LitInt = content.parse()?;
                                config.seed = Some(lit.base10_parse()?);
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &config_key,
                                    format!("Unknown config key: {}", config_key),
                                ));
                            }
                        }

                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!("Unknown key: {}", key),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(StatefulTestInput {
            name: name.ok_or_else(|| input.error("Missing required field: name"))?,
            state_type: state_type.ok_or_else(|| input.error("Missing required field: state"))?,
            state_init: state_init
                .ok_or_else(|| input.error("Missing required field: state initialization"))?,
            operations: operations
                .ok_or_else(|| input.error("Missing required field: operations"))?,
            invariants,
            config,
        })
    }
}

pub fn stateful_test_impl(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as StatefulTestInput);

    let test_name = &parsed.name;
    let state_type = &parsed.state_type;
    let state_init = &parsed.state_init;
    let operations = &parsed.operations;

    // Build invariants
    let invariant_calls = parsed.invariants.iter().map(|(name, closure)| {
        quote! {
            test = test.invariant(#name, #closure);
        }
    });

    // Build config
    let iterations = parsed.config.iterations.unwrap_or(100);
    let max_len = parsed.config.max_sequence_length.unwrap_or(10);
    let min_len = parsed.config.min_sequence_length.unwrap_or(1);

    let seed_setup = if let Some(seed) = parsed.config.seed {
        quote! {
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(#seed);
        }
    } else {
        quote! {
            let mut rng = rand::thread_rng();
        }
    };

    let expanded = quote! {
        #[test]
        fn #test_name() {
            use protest_stateful::dsl::StatefulTest;
            use protest_stateful::operations::{Operation, OperationSequence};
            use rand::Rng;

            let initial_state: #state_type = #state_init;

            let mut test = StatefulTest::<#state_type, #operations>::new(initial_state.clone());

            #(#invariant_calls)*

            #seed_setup

            // Run multiple iterations
            for iteration in 0..#iterations {
                let mut state = initial_state.clone();

                // Generate random sequence length
                let seq_len = rng.gen_range(#min_len..=#max_len);

                // Generate operation sequence
                let mut sequence = OperationSequence::new();
                for _ in 0..seq_len {
                    // This is a placeholder - in real usage, you'd use a Generator
                    // For now, we'll rely on manual operation generation
                    // TODO: Integrate with Generator trait once available
                }

                // Run the test
                match test.run(&sequence) {
                    Ok(_) => {
                        // Test passed
                    }
                    Err(failure) => {
                        panic!(
                            "Stateful test failed at iteration {}: {}",
                            iteration, failure
                        );
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

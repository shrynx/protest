//! Implementation of the #[property_test] attribute macro

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    FnArg, Ident, ItemFn, Lit, Meta, MetaNameValue, Pat, PatType, Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// Configuration for property test macro
#[derive(Default)]
struct PropertyTestConfig {
    iterations: Option<usize>,
    seed: Option<u64>,
    max_shrink_iterations: Option<usize>,
    shrink_timeout_secs: Option<u64>,
}

impl Parse for PropertyTestConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut config = PropertyTestConfig::default();

        if input.is_empty() {
            return Ok(config);
        }

        let punctuated: Punctuated<Meta, Token![,]> =
            input.parse_terminated(Meta::parse, Token![,])?;

        for meta in punctuated {
            match meta {
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    let name = path.get_ident().ok_or_else(|| {
                        syn::Error::new_spanned(&path, "Expected simple identifier")
                    })?;

                    match name.to_string().as_str() {
                        "iterations" => {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: Lit::Int(lit_int),
                                ..
                            }) = value
                            {
                                config.iterations = Some(lit_int.base10_parse()?);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "Expected integer literal",
                                ));
                            }
                        }
                        "seed" => {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: Lit::Int(lit_int),
                                ..
                            }) = value
                            {
                                config.seed = Some(lit_int.base10_parse()?);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "Expected integer literal",
                                ));
                            }
                        }
                        "max_shrink_iterations" => {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: Lit::Int(lit_int),
                                ..
                            }) = value
                            {
                                config.max_shrink_iterations = Some(lit_int.base10_parse()?);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "Expected integer literal",
                                ));
                            }
                        }
                        "shrink_timeout_secs" => {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: Lit::Int(lit_int),
                                ..
                            }) = value
                            {
                                config.shrink_timeout_secs = Some(lit_int.base10_parse()?);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "Expected integer literal",
                                ));
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                name,
                                "Unknown configuration option. Supported: iterations, seed, max_shrink_iterations, shrink_timeout_secs",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "Expected name-value pairs like 'iterations = 100'",
                    ));
                }
            }
        }

        Ok(config)
    }
}

/// Extract parameter information from function signature
struct ParameterInfo {
    name: Ident,
    ty: Type,
}

impl ParameterInfo {
    fn from_fn_arg(arg: &FnArg) -> Result<Self> {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                if let Pat::Ident(pat_ident) = pat.as_ref() {
                    Ok(ParameterInfo {
                        name: pat_ident.ident.clone(),
                        ty: (**ty).clone(),
                    })
                } else {
                    Err(syn::Error::new_spanned(
                        pat,
                        "Property test functions must have simple parameter names",
                    ))
                }
            }
            FnArg::Receiver(_) => Err(syn::Error::new_spanned(
                arg,
                "Property test functions cannot have self parameters",
            )),
        }
    }
}

/// Generate a generator expression for a given type
///
/// Prefers AutoGen (ergonomic API) for common types
fn generate_generator_for_type(ty: &Type) -> TokenStream2 {
    // Use AutoGen for ergonomic API support
    quote! {
        <#ty as ::protest::ergonomic::AutoGen>::auto_generator()
    }
}

/// Generate test configuration from macro attributes
fn generate_test_config(config: &PropertyTestConfig) -> TokenStream2 {
    let mut config_fields = Vec::new();

    if let Some(iterations) = config.iterations {
        config_fields.push(quote! { iterations: #iterations });
    }

    if let Some(seed) = config.seed {
        config_fields.push(quote! { seed: Some(#seed) });
    }

    if let Some(max_shrink) = config.max_shrink_iterations {
        config_fields.push(quote! { max_shrink_iterations: #max_shrink });
    }

    if let Some(timeout_secs) = config.shrink_timeout_secs {
        config_fields.push(quote! {
            shrink_timeout: ::std::time::Duration::from_secs(#timeout_secs)
        });
    }

    if config_fields.is_empty() {
        quote! { ::protest::TestConfig::default() }
    } else {
        quote! {
            ::protest::TestConfig {
                #(#config_fields,)*
                ..::protest::TestConfig::default()
            }
        }
    }
}

/// Check if a function is async
fn is_async_fn(item_fn: &ItemFn) -> bool {
    item_fn.sig.asyncness.is_some()
}

/// Generate the property test implementation
pub fn property_test_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(args as PropertyTestConfig);
    let item_fn = parse_macro_input!(input as ItemFn);

    // Validate function signature
    if !item_fn.sig.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &item_fn.sig.generics,
            "Property test functions cannot have generic parameters",
        )
        .to_compile_error()
        .into();
    }

    // Extract parameter information
    let params: Result<Vec<ParameterInfo>> = item_fn
        .sig
        .inputs
        .iter()
        .map(ParameterInfo::from_fn_arg)
        .collect();

    let params = match params {
        Ok(params) => params,
        Err(e) => return e.to_compile_error().into(),
    };

    if params.is_empty() {
        return syn::Error::new_spanned(
            &item_fn.sig.inputs,
            "Property test functions must have at least one parameter",
        )
        .to_compile_error()
        .into();
    }

    // Generate the test implementation
    let test_name = &item_fn.sig.ident;
    let test_config = generate_test_config(&config);
    let is_async = is_async_fn(&item_fn);

    // Create the original function with a different name
    let original_fn_name = Ident::new(&format!("__{}_original", test_name), Span::call_site());
    let mut original_fn = item_fn.clone();
    original_fn.sig.ident = original_fn_name.clone();

    // Remove test attributes from the original function
    original_fn
        .attrs
        .retain(|attr| !attr.path().is_ident("test") && !attr.path().is_ident("tokio::test"));

    let generated_test = if params.len() == 1 {
        // Single parameter case - simpler implementation
        let param = &params[0];
        let param_name = &param.name;
        let param_type = &param.ty;
        let generator = generate_generator_for_type(&param.ty);

        if is_async {
            quote! {
                #[::tokio::test]
                async fn #test_name() {
                    use ::protest::{check_async_with_config, AsyncProperty, PropertyError};

                    struct TestProperty;
                    impl AsyncProperty<#param_type> for TestProperty {
                        type Output = ();
                        async fn test(&self, #param_name: #param_type) -> Result<Self::Output, PropertyError> {
                            #original_fn_name(#param_name).await;
                            Ok(())
                        }
                    }

                    let generator = #generator;
                    let property = TestProperty;
                    let config = #test_config;

                    match check_async_with_config(generator, property, config).await {
                        Ok(_) => {},
                        Err(failure) => {
                            panic!("Property test failed: {}", failure);
                        }
                    }
                }
            }
        } else {
            quote! {
                #[test]
                fn #test_name() {
                    use ::protest::{check_with_config, Property, PropertyError};

                    struct TestProperty;
                    impl Property<#param_type> for TestProperty {
                        type Output = ();
                        fn test(&self, #param_name: #param_type) -> Result<Self::Output, PropertyError> {
                            #original_fn_name(#param_name);
                            Ok(())
                        }
                    }

                    let generator = #generator;
                    let property = TestProperty;
                    let config = #test_config;

                    match check_with_config(generator, property, config) {
                        Ok(_) => {},
                        Err(failure) => {
                            panic!("Property test failed: {}", failure);
                        }
                    }
                }
            }
        }
    } else {
        // Multiple parameters case - use tuple generator
        let param_names: Vec<_> = params.iter().map(|p| &p.name).collect();
        let param_types: Vec<_> = params.iter().map(|p| &p.ty).collect();
        let generators: Vec<_> = params
            .iter()
            .map(|p| generate_generator_for_type(&p.ty))
            .collect();

        // Create tuple type and generator
        let tuple_type = if param_types.len() == 2 {
            quote! { (#(#param_types),*) }
        } else {
            quote! { (#(#param_types,)*) }
        };

        let tuple_generator = if generators.len() == 2 {
            quote! { (#(#generators),*) }
        } else {
            quote! { (#(#generators,)*) }
        };

        // Create parameter destructuring
        let param_destructure = if param_names.len() == 2 {
            quote! { (#(#param_names),*) }
        } else {
            quote! { (#(#param_names,)*) }
        };

        if is_async {
            quote! {
                #[::tokio::test]
                async fn #test_name() {
                    use ::protest::{check_async_with_config, AsyncProperty, PropertyError};

                    struct TestProperty;
                    impl AsyncProperty<#tuple_type> for TestProperty {
                        type Output = ();
                        async fn test(&self, input: #tuple_type) -> Result<Self::Output, PropertyError> {
                            let #param_destructure = input;
                            #original_fn_name(#(#param_names),*).await;
                            Ok(())
                        }
                    }

                    let generator = #tuple_generator;
                    let property = TestProperty;
                    let config = #test_config;

                    match check_async_with_config(generator, property, config).await {
                        Ok(_) => {},
                        Err(failure) => {
                            panic!("Property test failed: {}", failure);
                        }
                    }
                }
            }
        } else {
            quote! {
                #[test]
                fn #test_name() {
                    use ::protest::{check_with_config, Property, PropertyError};

                    struct TestProperty;
                    impl Property<#tuple_type> for TestProperty {
                        type Output = ();
                        fn test(&self, input: #tuple_type) -> Result<Self::Output, PropertyError> {
                            let #param_destructure = input;
                            #original_fn_name(#(#param_names),*);
                            Ok(())
                        }
                    }

                    let generator = #tuple_generator;
                    let property = TestProperty;
                    let config = #test_config;

                    match check_with_config(generator, property, config) {
                        Ok(_) => {},
                        Err(failure) => {
                            panic!("Property test failed: {}", failure);
                        }
                    }
                }
            }
        }
    };

    let result = quote! {
        #original_fn
        #generated_test
    };

    result.into()
}

/// Implementation of the test_builder macro
pub fn test_builder_impl(input: TokenStream) -> TokenStream {
    let builder_config = parse_macro_input!(input as TestBuilderConfig);

    let test_name = &builder_config.test_name;
    let generator = &builder_config.generator;
    let property = &builder_config.property;

    // Generate test configuration
    let mut config_fields = Vec::new();

    if let Some(iterations) = builder_config.iterations {
        config_fields.push(quote! { iterations: #iterations });
    }

    if let Some(seed) = builder_config.seed {
        config_fields.push(quote! { seed: Some(#seed) });
    }

    if let Some(max_shrink) = builder_config.max_shrink_iterations {
        config_fields.push(quote! { max_shrink_iterations: #max_shrink });
    }

    if let Some(timeout_secs) = builder_config.shrink_timeout_secs {
        config_fields.push(quote! {
            shrink_timeout: ::std::time::Duration::from_secs(#timeout_secs)
        });
    }

    let test_config = if config_fields.is_empty() {
        quote! { ::protest::TestConfig::default() }
    } else {
        quote! {
            ::protest::TestConfig {
                #(#config_fields,)*
                ..::protest::TestConfig::default()
            }
        }
    };

    // Generate the test function
    let result = quote! {
        #[test]
        fn #test_name() {
            use ::protest::ergonomic::check_with_closure_config;

            let generator = #generator;
            let property_fn = #property;
            let config = #test_config;

            match check_with_closure_config(generator, property_fn, config) {
                Ok(_) => {},
                Err(failure) => {
                    panic!("Property test failed: {}", failure);
                }
            }
        }
    };

    result.into()
}

/// Configuration for the test_builder macro
struct TestBuilderConfig {
    test_name: Ident,
    generator: syn::Expr,
    property: syn::Expr,
    iterations: Option<usize>,
    seed: Option<u64>,
    max_shrink_iterations: Option<usize>,
    shrink_timeout_secs: Option<u64>,
}

impl Parse for TestBuilderConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut test_name = None;
        let mut generator = None;
        let mut property = None;
        let mut iterations = None;
        let mut seed = None;
        let mut max_shrink_iterations = None;
        let mut shrink_timeout_secs = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match ident.to_string().as_str() {
                "test_name" => {
                    test_name = Some(input.parse()?);
                }
                "generator" => {
                    generator = Some(input.parse()?);
                }
                "property" => {
                    property = Some(input.parse()?);
                }
                "iterations" => {
                    let lit: syn::LitInt = input.parse()?;
                    iterations = Some(lit.base10_parse()?);
                }
                "seed" => {
                    let lit: syn::LitInt = input.parse()?;
                    seed = Some(lit.base10_parse()?);
                }
                "max_shrink_iterations" => {
                    let lit: syn::LitInt = input.parse()?;
                    max_shrink_iterations = Some(lit.base10_parse()?);
                }
                "shrink_timeout_secs" => {
                    let lit: syn::LitInt = input.parse()?;
                    shrink_timeout_secs = Some(lit.base10_parse()?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "Unknown configuration option. Supported: test_name, generator, property, iterations, seed, max_shrink_iterations, shrink_timeout_secs",
                    ));
                }
            }

            // Parse optional comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let test_name =
            test_name.ok_or_else(|| syn::Error::new(input.span(), "test_name is required"))?;

        let generator =
            generator.ok_or_else(|| syn::Error::new(input.span(), "generator is required"))?;

        let property =
            property.ok_or_else(|| syn::Error::new(input.span(), "property is required"))?;

        Ok(TestBuilderConfig {
            test_name,
            generator,
            property,
            iterations,
            seed,
            max_shrink_iterations,
            shrink_timeout_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_property_test_config_parsing() {
        let input: TokenStream2 = quote! { iterations = 500, seed = 42 };
        let config: PropertyTestConfig = syn::parse2(input).unwrap();

        assert_eq!(config.iterations, Some(500));
        assert_eq!(config.seed, Some(42));
        assert_eq!(config.max_shrink_iterations, None);
        assert_eq!(config.shrink_timeout_secs, None);
    }

    #[test]
    fn test_property_test_config_empty() {
        let input: TokenStream2 = quote! {};
        let config: PropertyTestConfig = syn::parse2(input).unwrap();

        assert_eq!(config.iterations, None);
        assert_eq!(config.seed, None);
        assert_eq!(config.max_shrink_iterations, None);
        assert_eq!(config.shrink_timeout_secs, None);
    }

    #[test]
    fn test_parameter_info_extraction() {
        let fn_arg: FnArg = parse_quote! { x: i32 };
        let param_info = ParameterInfo::from_fn_arg(&fn_arg).unwrap();

        assert_eq!(param_info.name.to_string(), "x");
        // Type comparison is complex, so we just check it parses
    }

    #[test]
    fn test_is_async_fn_detection() {
        let sync_fn: ItemFn = parse_quote! {
            fn test_sync(x: i32) {
                assert!(x > 0);
            }
        };
        assert!(!is_async_fn(&sync_fn));

        let async_fn: ItemFn = parse_quote! {
            async fn test_async(x: i32) {
                assert!(x > 0);
            }
        };
        assert!(is_async_fn(&async_fn));
    }

    #[test]
    fn test_generate_test_config_default() {
        let config = PropertyTestConfig::default();
        let generated = generate_test_config(&config);
        let expected = quote! { ::protest::TestConfig::default() };

        assert_eq!(generated.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_test_config_with_values() {
        let config = PropertyTestConfig {
            iterations: Some(1000),
            seed: Some(42),
            max_shrink_iterations: None,
            shrink_timeout_secs: Some(30),
        };
        let generated = generate_test_config(&config);

        // Check that the generated config contains the expected fields
        let generated_str = generated.to_string();
        assert!(generated_str.contains("iterations : 1000"));
        assert!(generated_str.contains("seed : Some (42"));
        assert!(
            generated_str.contains("shrink_timeout")
                && generated_str.contains("Duration")
                && generated_str.contains("from_secs (30")
        );
    }
}

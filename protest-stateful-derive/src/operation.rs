//! Implementation of the #[derive(Operation)] macro

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Meta, Type, parse_macro_input};

pub fn derive_operation_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract state type from container attribute
    let state_type = extract_state_type(&input.attrs);

    // Only support enums
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new_spanned(
                name,
                "#[derive(Operation)] can only be applied to enums",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate execute match arms
    let execute_arms =
        variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            let execute_expr = extract_attribute_string(&variant.attrs, "execute");
            let fields = &variant.fields;

            match fields {
                Fields::Unit => {
                    if let Some(expr) = execute_expr {
                        let expr_tokens: proc_macro2::TokenStream = expr.parse().unwrap();
                        quote! {
                            Self::#variant_name => { #expr_tokens; }
                        }
                    } else {
                        quote! {
                            Self::#variant_name => {}
                        }
                    }
                }
                Fields::Unnamed(fields_unnamed) => {
                    let field_names: Vec<_> = (0..fields_unnamed.unnamed.len())
                        .map(|i| {
                            let ident = quote::format_ident!("field_{}", i);
                            ident
                        })
                        .collect();

                    if let Some(expr) = execute_expr {
                        // Replace self.0, self.1, etc. with field names
                        let mut expr_str = expr.clone();
                        for (i, _) in field_names.iter().enumerate() {
                            expr_str =
                                expr_str.replace(&format!("self.{}", i), &format!("field_{}", i));
                        }
                        let expr_tokens: proc_macro2::TokenStream = expr_str.parse().unwrap();

                        quote! {
                            Self::#variant_name(#(#field_names),*) => { #expr_tokens; }
                        }
                    } else {
                        quote! {
                            Self::#variant_name(#(#field_names),*) => {}
                        }
                    }
                }
                Fields::Named(fields_named) => {
                    let field_names: Vec<_> = fields_named
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();

                    if let Some(expr) = execute_expr {
                        let expr_tokens: proc_macro2::TokenStream = expr.parse().unwrap();
                        // Prefix field names with _ to avoid unused variable warnings
                        let prefixed_names: Vec<_> = field_names
                            .iter()
                            .map(|name| quote::format_ident!("_{}", name))
                            .collect();
                        // Create bindings like: _key: key, _value: value
                        let bindings = field_names.iter().zip(prefixed_names.iter()).map(
                            |(field, prefixed)| {
                                quote! { #field: #prefixed }
                            },
                        );
                        quote! {
                            Self::#variant_name { #(#bindings),* } => {
                                // Rebind with original names for use in expression
                                #(let #field_names = #prefixed_names;)*
                                #expr_tokens;
                            }
                        }
                    } else {
                        // For variants without execute, just ignore the fields
                        let prefixed_names: Vec<_> = field_names
                            .iter()
                            .map(|name| quote::format_ident!("_{}", name))
                            .collect();
                        let bindings = field_names.iter().zip(prefixed_names.iter()).map(
                            |(field, prefixed)| {
                                quote! { #field: #prefixed }
                            },
                        );
                        quote! {
                            Self::#variant_name { #(#bindings),* } => {}
                        }
                    }
                }
            }
        });

    // Generate precondition match arms
    let precondition_arms =
        variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            let precondition_expr = extract_attribute_string(&variant.attrs, "precondition");
            let fields = &variant.fields;

            match fields {
                Fields::Unit => {
                    if let Some(expr) = precondition_expr {
                        let expr_tokens: proc_macro2::TokenStream = expr.parse().unwrap();
                        quote! {
                            Self::#variant_name => #expr_tokens
                        }
                    } else {
                        quote! {
                            Self::#variant_name => true
                        }
                    }
                }
                Fields::Unnamed(fields_unnamed) => {
                    let field_names: Vec<_> = (0..fields_unnamed.unnamed.len())
                        .map(|i| quote::format_ident!("field_{}", i))
                        .collect();

                    if let Some(expr) = precondition_expr {
                        // Replace self.0, self.1, etc. with field names
                        let mut expr_str = expr.clone();
                        for (i, _) in field_names.iter().enumerate() {
                            expr_str =
                                expr_str.replace(&format!("self.{}", i), &format!("field_{}", i));
                        }
                        let expr_tokens: proc_macro2::TokenStream = expr_str.parse().unwrap();

                        quote! {
                            Self::#variant_name(#(#field_names),*) => #expr_tokens
                        }
                    } else {
                        quote! {
                            Self::#variant_name(#(#field_names),*) => true
                        }
                    }
                }
                Fields::Named(fields_named) => {
                    let field_names: Vec<_> = fields_named
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();

                    if let Some(expr) = precondition_expr {
                        let expr_tokens: proc_macro2::TokenStream = expr.parse().unwrap();
                        // Prefix field names with _ to avoid unused variable warnings
                        let prefixed_names: Vec<_> = field_names
                            .iter()
                            .map(|name| quote::format_ident!("_{}", name))
                            .collect();
                        let bindings = field_names.iter().zip(prefixed_names.iter()).map(
                            |(field, prefixed)| {
                                quote! { #field: #prefixed }
                            },
                        );
                        quote! {
                            Self::#variant_name { #(#bindings),* } => {
                                #(let #field_names = #prefixed_names;)*
                                #expr_tokens
                            }
                        }
                    } else {
                        // For variants without precondition, just ignore the fields
                        let prefixed_names: Vec<_> = field_names
                            .iter()
                            .map(|name| quote::format_ident!("_{}", name))
                            .collect();
                        let bindings = field_names.iter().zip(prefixed_names.iter()).map(
                            |(field, prefixed)| {
                                quote! { #field: #prefixed }
                            },
                        );
                        quote! {
                            Self::#variant_name { #(#bindings),* } => true
                        }
                    }
                }
            }
        });

    // Generate description match arms
    let description_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let description = extract_attribute_string(&variant.attrs, "description");
        let fields = &variant.fields;

        // If custom description is provided, use it; otherwise use Debug format
        if let Some(desc) = description {
            match fields {
                Fields::Unit => quote! {
                    Self::#variant_name => #desc.to_string()
                },
                Fields::Unnamed(_) => quote! {
                    Self::#variant_name(..) => #desc.to_string()
                },
                Fields::Named(_) => quote! {
                    Self::#variant_name { .. } => #desc.to_string()
                },
            }
        } else {
            match fields {
                Fields::Unit => quote! {
                    Self::#variant_name => format!("{:?}", self)
                },
                Fields::Unnamed(_) => quote! {
                    Self::#variant_name(..) => format!("{:?}", self)
                },
                Fields::Named(_) => quote! {
                    Self::#variant_name { .. } => format!("{:?}", self)
                },
            }
        }
    });

    // Generate weight match arms
    let weight_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let weight = extract_weight(&variant.attrs).unwrap_or(1);
        let fields = &variant.fields;

        match fields {
            Fields::Unit => quote! {
                Self::#variant_name => #weight
            },
            Fields::Unnamed(_) => quote! {
                Self::#variant_name(..) => #weight
            },
            Fields::Named(_) => quote! {
                Self::#variant_name { .. } => #weight
            },
        }
    });

    let expanded = quote! {
        impl protest_stateful::operations::Operation for #name {
            type State = #state_type;

            fn execute(&self, state: &mut Self::State) {
                match self {
                    #(#execute_arms)*
                }
            }

            fn precondition(&self, state: &Self::State) -> bool {
                match self {
                    #(#precondition_arms),*
                }
            }

            fn description(&self) -> String {
                match self {
                    #(#description_arms),*
                }
            }

            fn weight(&self) -> u32 {
                match self {
                    #(#weight_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extract the state type from #[operation(state = "Type")]
fn extract_state_type(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("operation")
            && let Meta::List(meta_list) = &attr.meta
        {
            let tokens = &meta_list.tokens;
            let tokens_str = tokens.to_string();

            // Parse "state = "Type""
            if let Some(start) = tokens_str.find("state") {
                let rest = &tokens_str[start..];
                if let Some(eq_pos) = rest.find('=') {
                    let after_eq = rest[eq_pos + 1..].trim();
                    // Remove quotes if present
                    let type_str = after_eq.trim_matches('"').trim();
                    if let Ok(ty) = syn::parse_str::<Type>(type_str) {
                        return quote! { #ty };
                    }
                }
            }
        }
    }

    // Default to a generic placeholder if not specified
    quote! { () }
}

/// Extract a string value from an attribute like #[execute("...")]
fn extract_attribute_string(attrs: &[Attribute], name: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(name)
            && let Meta::List(meta_list) = &attr.meta
        {
            let tokens_str = meta_list.tokens.to_string();
            // Remove surrounding quotes
            return Some(tokens_str.trim_matches('"').to_string());
        }
    }
    None
}

/// Extract weight from #[weight(N)]
pub fn extract_weight(attrs: &[Attribute]) -> Option<u32> {
    for attr in attrs {
        if attr.path().is_ident("weight")
            && let Meta::List(meta_list) = &attr.meta
        {
            let tokens_str = meta_list.tokens.to_string();
            if let Ok(weight) = tokens_str.parse::<u32>() {
                return Some(weight);
            }
        }
    }
    None
}

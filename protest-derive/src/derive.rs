//! Derive macro implementation for automatic Generator trait derivation
//!
//! This module provides procedural macros for automatically implementing the Generator trait
//! for structs and enums, with support for customization through attributes.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{
    Attribute, Data, DeriveInput, Error, Field, Fields, FieldsNamed, FieldsUnnamed, GenericParam,
    Lit, Meta, MetaList, MetaNameValue, Result, Type, Variant, parse_macro_input, parse_quote,
};

/// Main entry point for the Generator derive macro
pub fn derive_generator_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_generator_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Generate the Generator implementation for the given input
fn generate_generator_impl(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Create generator struct name
    let generator_name = format_ident!("{}Generator", name);

    // Add bounds for Generator trait requirements
    let mut bounded_generics = generics.clone();
    add_trait_bounds(&mut bounded_generics);
    let (bounded_impl_generics, _, bounded_where_clause) = bounded_generics.split_for_impl();

    let generate_body = match &input.data {
        Data::Struct(data_struct) => generate_struct_body(name, &data_struct.fields)?,
        Data::Enum(data_enum) => {
            generate_enum_body(name, &data_enum.variants.iter().collect::<Vec<_>>())?
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(
                input,
                "Generator derive is not supported for unions",
            ));
        }
    };

    let shrink_body = generate_shrink_body(&input.data)?;

    // Generate phantom data fields for generics
    let (phantom_fields, phantom_data_init) = if generics.params.is_empty() {
        (quote! {}, quote! {})
    } else {
        let type_params: Vec<_> = generics
            .params
            .iter()
            .filter_map(|param| {
                if let GenericParam::Type(type_param) = param {
                    Some(&type_param.ident)
                } else {
                    None
                }
            })
            .collect();

        if type_params.is_empty() {
            (quote! {}, quote! {})
        } else {
            (
                quote! {
                    _phantom: std::marker::PhantomData<(#(#type_params,)*)>,
                },
                quote! {
                    _phantom: std::marker::PhantomData,
                },
            )
        }
    };

    Ok(quote! {
        // Generate the generator struct
        #[derive(Debug, Clone)]
        struct #generator_name #impl_generics #where_clause {
            #phantom_fields
        }

        // Implement Default for the generator struct
        impl #bounded_impl_generics Default for #generator_name #ty_generics
        #bounded_where_clause
        {
            fn default() -> Self {
                Self {
                    #phantom_data_init
                }
            }
        }

        // Implement Strategy trait for the generator struct
        impl #bounded_impl_generics protest::Strategy for #generator_name #ty_generics
        #bounded_where_clause
        {
            type Value = #name #ty_generics;

            fn generate<R: rand::Rng>(&self, rng: &mut R, config: &protest::GeneratorConfig) -> Self::Value {
                use protest::Strategy;

                #generate_body
            }

            fn shrink(&self, value: &Self::Value) -> Box<dyn Iterator<Item = Self::Value>> {
                #shrink_body
            }
        }

        // Implement Arbitrary trait for the original type
        impl #bounded_impl_generics protest::Arbitrary for #name #ty_generics
        #bounded_where_clause
        {
            type Strategy = #generator_name #ty_generics;
            type Parameters = ();

            fn arbitrary() -> Self::Strategy {
                #generator_name::default()
            }

            fn arbitrary_with(_params: Self::Parameters) -> Self::Strategy {
                #generator_name::default()
            }
        }
    })
}

/// Add necessary trait bounds to generic parameters
fn add_trait_bounds(generics: &mut syn::Generics) {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(protest::Arbitrary));
            type_param.bounds.push(parse_quote!(Clone));
            type_param.bounds.push(parse_quote!('static));
        }
    }
}

/// Generate the body for struct generation
fn generate_struct_body(name: &syn::Ident, fields: &Fields) -> Result<TokenStream> {
    match fields {
        Fields::Named(fields_named) => generate_named_fields_body(name, fields_named),
        Fields::Unnamed(fields_unnamed) => generate_unnamed_fields_body(name, fields_unnamed),
        Fields::Unit => Ok(quote! { #name }),
    }
}

/// Generate body for structs with named fields
fn generate_named_fields_body(name: &syn::Ident, fields: &FieldsNamed) -> Result<TokenStream> {
    let field_generators = fields
        .named
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();

            // Check for field-level customization attributes
            let generator_expr = parse_field_attributes(field)?;

            Ok(quote! {
                #field_name: {
                    #generator_expr
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #name {
            #(#field_generators,)*
        }
    })
}

/// Generate body for structs with unnamed fields (tuple structs)
fn generate_unnamed_fields_body(name: &syn::Ident, fields: &FieldsUnnamed) -> Result<TokenStream> {
    let field_generators = fields
        .unnamed
        .iter()
        .map(|field| {
            let generator_expr = parse_field_attributes(field)?;

            Ok(quote! {
                {
                    #generator_expr
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #name(
            #(#field_generators,)*
        )
    })
}

/// Generate the body for enum generation
fn generate_enum_body(name: &syn::Ident, variants: &[&Variant]) -> Result<TokenStream> {
    if variants.is_empty() {
        return Err(Error::new_spanned(
            name,
            "Cannot derive Generator for empty enum",
        ));
    }

    let variant_count = variants.len();
    let variant_arms = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let variant_name = &variant.ident;
            let variant_body = match &variant.fields {
                Fields::Named(fields_named) => {
                    let field_generators = fields_named
                        .named
                        .iter()
                        .map(|field| {
                            let field_name = field.ident.as_ref().unwrap();
                            let generator_expr = parse_field_attributes(field)?;

                            Ok(quote! {
                                #field_name: {
                                    #generator_expr
                                }
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;

                    Ok::<TokenStream, Error>(quote! {
                        #name::#variant_name {
                            #(#field_generators,)*
                        }
                    })
                }
                Fields::Unnamed(fields_unnamed) => {
                    let field_generators = fields_unnamed
                        .unnamed
                        .iter()
                        .map(|field| {
                            let generator_expr = parse_field_attributes(field)?;

                            Ok(quote! {
                                {
                                    #generator_expr
                                }
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;

                    Ok::<TokenStream, Error>(quote! {
                        #name::#variant_name(
                            #(#field_generators,)*
                        )
                    })
                }
                Fields::Unit => Ok::<TokenStream, Error>(quote! { #name::#variant_name }),
            }?;

            Ok(quote! {
                #index => #variant_body
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        {
            use rand::Rng;
            let variant_index = rng.gen_range(0..#variant_count);
            match variant_index {
                #(#variant_arms,)*
                _ => unreachable!("Invalid variant index")
            }
        }
    })
}

/// Parse field-level attributes for customization
fn parse_field_attributes(field: &Field) -> Result<TokenStream> {
    let field_type = &field.ty;

    // Look for #[generator(...)] attributes
    for attr in &field.attrs {
        if attr.path().is_ident("generator") {
            return parse_generator_attribute(attr, field_type);
        }
    }

    // Default generation using Arbitrary trait
    Ok(quote! {
        {
            let strategy = <#field_type as protest::Arbitrary>::arbitrary();
            protest::Strategy::generate(&strategy, rng, config)
        }
    })
}

/// Parse a #[generator(...)] attribute
fn parse_generator_attribute(attr: &Attribute, field_type: &Type) -> Result<TokenStream> {
    let meta = attr.meta.clone();

    match meta {
        Meta::List(MetaList { tokens, .. }) => {
            // Parse the tokens inside the attribute
            let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
            let parsed = parser.parse2(tokens)?;

            for meta in parsed {
                match meta {
                    Meta::NameValue(MetaNameValue { path, value, .. }) => {
                        if path.is_ident("range") {
                            return parse_range_attribute(&value, field_type);
                        } else if path.is_ident("length") {
                            return parse_length_attribute(&value, field_type);
                        } else if path.is_ident("custom") {
                            return parse_custom_attribute(&value, field_type);
                        }
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            meta,
                            "Unsupported generator attribute format",
                        ));
                    }
                }
            }
        }
        _ => {
            return Err(Error::new_spanned(
                attr,
                "Generator attribute must be a list",
            ));
        }
    }

    // Fallback to default generation
    Ok(quote! {
        {
            let strategy = <#field_type as crate::Arbitrary>::arbitrary();
            protest::Strategy::generate(&strategy, &mut local_rng, config)
        }
    })
}

/// Parse range attribute (e.g., range = "1..100")
fn parse_range_attribute(value: &syn::Expr, field_type: &Type) -> Result<TokenStream> {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = value
    {
        let range_str = lit_str.value();

        // Parse range string (simple implementation for "min..max" format)
        if let Some((start, end)) = parse_range_string(&range_str) {
            return Ok(quote! {
                {
                    let strategy = <#field_type as protest::Arbitrary>::arbitrary_with((#start, #end));
                    protest::Strategy::generate(&strategy, rng, config)
                }
            });
        }
    }

    Err(Error::new_spanned(
        value,
        "Range attribute must be a string literal in format \"min..max\"",
    ))
}

/// Parse length attribute for collections (e.g., length = "5..20")
fn parse_length_attribute(value: &syn::Expr, field_type: &Type) -> Result<TokenStream> {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = value
    {
        let length_str = lit_str.value();

        if let Some((min, max)) = parse_range_string(&length_str) {
            return Ok(quote! {
                {
                    let strategy = <#field_type as protest::Arbitrary>::arbitrary_with((#min, #max));
                    protest::Strategy::generate(&strategy, rng, config)
                }
            });
        }
    }

    Err(Error::new_spanned(
        value,
        "Length attribute must be a string literal in format \"min..max\"",
    ))
}

/// Parse custom generator attribute (e.g., custom = "always_true")
fn parse_custom_attribute(value: &syn::Expr, _field_type: &Type) -> Result<TokenStream> {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = value
    {
        let custom_fn = format_ident!("{}", lit_str.value());

        return Ok(quote! {
            {
                #custom_fn()
            }
        });
    }

    Err(Error::new_spanned(
        value,
        "Custom attribute must be a string literal with function name",
    ))
}

/// Parse a range string like "1..100" into (start, end) as token streams
fn parse_range_string(range_str: &str) -> Option<(TokenStream, TokenStream)> {
    if let Some(pos) = range_str.find("..") {
        let start_str = &range_str[..pos];
        let end_str = &range_str[pos + 2..];

        // Parse as token streams to preserve the original type
        if let (Ok(start), Ok(end)) = (
            start_str.parse::<TokenStream>(),
            end_str.parse::<TokenStream>(),
        ) {
            return Some((start, end));
        }
    }
    None
}

/// Generate shrinking implementation
fn generate_shrink_body(data: &Data) -> Result<TokenStream> {
    match data {
        Data::Struct(_) => {
            // For now, basic shrinking implementation
            Ok(quote! {
                // Basic shrinking - return empty iterator for now
                // TODO: Implement proper shrinking strategies
                Box::new(std::iter::empty())
            })
        }
        Data::Enum(_) => {
            Ok(quote! {
                // Basic enum shrinking - return empty iterator for now
                // TODO: Implement enum shrinking strategies
                Box::new(std::iter::empty())
            })
        }
        Data::Union(_) => Err(Error::new(
            Span::call_site(),
            "Shrinking not supported for unions",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_parse_range_string() {
        let result = parse_range_string("1..100");
        assert!(result.is_some());
        if let Some((start, end)) = result {
            assert_eq!(start.to_string(), "1");
            assert_eq!(end.to_string(), "100");
        }

        let result = parse_range_string("0..10");
        assert!(result.is_some());
        if let Some((start, end)) = result {
            assert_eq!(start.to_string(), "0");
            assert_eq!(end.to_string(), "10");
        }

        let result = parse_range_string("-5..5");
        assert!(result.is_some());
        if let Some((start, end)) = result {
            assert_eq!(start.to_string(), "- 5");
            assert_eq!(end.to_string(), "5");
        }

        assert!(parse_range_string("invalid").is_none());
    }

    #[test]
    fn test_add_trait_bounds() {
        let mut generics: syn::Generics = parse_quote! { <T, U> };
        add_trait_bounds(&mut generics);

        // Check that bounds were added
        if let GenericParam::Type(type_param) = &generics.params[0] {
            assert_eq!(type_param.bounds.len(), 3); // Arbitrary, Clone, 'static
        }
    }

    #[test]
    fn test_generate_struct_body_unit() {
        let name: syn::Ident = parse_quote! { UnitStruct };
        let fields = Fields::Unit;

        let result = generate_struct_body(&name, &fields).unwrap();
        let expected = quote! { UnitStruct };

        assert_eq!(result.to_string(), expected.to_string());
    }
}

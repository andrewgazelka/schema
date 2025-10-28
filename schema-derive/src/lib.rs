use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, Meta, parse_macro_input};

/// Extract documentation comments from attributes
fn extract_docs(attrs: &[syn::Attribute]) -> Option<String> {
    let mut docs = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc")
            && let Meta::NameValue(meta) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &meta.value
            && let Lit::Str(lit_str) = &expr_lit.lit
        {
            docs.push(lit_str.value());
        }
    }

    if docs.is_empty() {
        None
    } else {
        // Join lines and trim whitespace
        let joined = docs.iter().map(|s| s.trim()).collect::<Vec<_>>().join("\n");
        Some(joined.trim().to_string())
    }
}

/// Check if field has #[schema(skip)] attribute
fn is_skipped(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("schema")
            && let Ok(meta) = attr.meta.require_list()
        {
            return meta.tokens.to_string() == "skip";
        }
        false
    })
}

#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let schema_impl = match &input.data {
        Data::Struct(data) => derive_struct(data, &input.attrs),
        Data::Enum(data) => derive_enum(data, &input.attrs),
        Data::Union(_) => {
            return quote! {
                compile_error!("Schema derive does not support unions");
            }
            .into();
        }
    };

    let expanded = quote! {
        impl #impl_generics schema::Schema for #name #ty_generics #where_clause {
            fn schema() -> schema::SchemaType {
                #schema_impl
            }

            fn type_name() -> Option<&'static str> {
                Some(stringify!(#name))
            }
        }
    };

    TokenStream::from(expanded)
}

fn description_expr(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    match extract_docs(attrs) {
        Some(desc) => quote! { Some(#desc.to_string()) },
        None => quote! { None },
    }
}

fn schema_with_description(
    field_type: &syn::Type,
    field_attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    match extract_docs(field_attrs) {
        Some(desc) => quote! {
            {
                let mut schema = <#field_type as schema::Schema>::schema();
                schema.description = Some(#desc.to_string());
                schema
            }
        },
        None => quote! { <#field_type as schema::Schema>::schema() },
    }
}

fn derive_struct(data: &syn::DataStruct, attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let description_expr = description_expr(attrs);

    match &data.fields {
        Fields::Named(fields) => {
            let mut properties = vec![];
            let mut required = vec![];

            for field in &fields.named {
                // Skip fields with #[schema(skip)] attribute
                if is_skipped(&field.attrs) {
                    continue;
                }

                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let field_type = &field.ty;

                // Check if field is Option<T> - if not, it's required
                let is_optional = is_option_type(field_type);

                // Get base schema and add description
                let schema_expr = schema_with_description(field_type, &field.attrs);

                properties.push(quote! {
                    properties.insert(
                        #field_name_str.to_string(),
                        #schema_expr
                    );
                });

                if !is_optional {
                    required.push(quote! {
                        required.push(#field_name_str.to_string());
                    });
                }
            }

            quote! {
                {
                    let mut properties = std::collections::HashMap::new();
                    let mut required = Vec::new();
                    #(#properties)*
                    #(#required)*
                    schema::SchemaType {
                        kind: schema::TypeKind::Object {
                            properties,
                            required,
                        },
                        description: #description_expr,
                    }
                }
            }
        }
        Fields::Unnamed(_) => {
            quote! {
                compile_error!("Schema derive does not support tuple structs");
            }
        }
        Fields::Unit => quote! {
            schema::SchemaType {
                kind: schema::TypeKind::Object {
                    properties: std::collections::HashMap::new(),
                    required: Vec::new(),
                },
                description: #description_expr,
            }
        },
    }
}

fn derive_enum(data: &syn::DataEnum, attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let description_expr = description_expr(attrs);

    // Check if this is a simple enum (all variants are unit) or tagged union
    let all_unit = data
        .variants
        .iter()
        .all(|v| matches!(v.fields, Fields::Unit));

    if all_unit {
        // Simple enum - generate Enum schema
        let variants: Vec<_> = data
            .variants
            .iter()
            .map(|v| {
                let variant_name = v.ident.to_string().to_lowercase();
                quote! { variants.push(#variant_name.to_string()); }
            })
            .collect();

        quote! {
            {
                let mut variants = Vec::new();
                #(#variants)*
                schema::SchemaType {
                    kind: schema::TypeKind::Enum {
                        variants,
                    },
                    description: #description_expr,
                }
            }
        }
    } else {
        // Tagged union - flatten into discriminator + data fields
        let mut tag_variants = vec![];
        let mut all_data_fields = std::collections::HashMap::new();

        for variant in &data.variants {
            let variant_name = variant.ident.to_string().to_lowercase();
            tag_variants.push(quote! {
                tag_variants.push(#variant_name.to_string());
            });

            // Collect all possible data fields from this variant
            #[allow(clippy::excessive_nesting)]
            if let Fields::Named(fields) = &variant.fields {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap().to_string();
                    if !all_data_fields.contains_key(&field_name) {
                        let field_type = &field.ty;
                        let schema_expr = schema_with_description(field_type, &field.attrs);

                        all_data_fields.insert(
                            field_name.clone(),
                            quote! {
                                data_fields.insert(
                                    #field_name.to_string(),
                                    #schema_expr
                                );
                            },
                        );
                    }
                }
            }
        }

        let data_field_inserts: Vec<_> = all_data_fields.values().collect();

        quote! {
            {
                let mut tag_variants = Vec::new();
                let mut data_fields = std::collections::HashMap::new();
                #(#tag_variants)*
                #(#data_field_inserts)*
                schema::SchemaType {
                    kind: schema::TypeKind::TaggedUnion {
                        tag_field: "type".to_string(),
                        tag_variants,
                        data_fields,
                    },
                    description: #description_expr,
                }
            }
        }
    }
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

//! Procedural macro implementation for envsense detection merging

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed};

/// Derive macro for automatic detection merging
/// 
/// This macro generates a `DetectionMerger` implementation that automatically
/// merges detection results based on field names.
#[proc_macro_derive(DetectionMerger)]
pub fn derive_detection_merger(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let struct_name = input.ident;
    let fields = parse_fields(&input.data);
    
    let merge_impl = generate_merge_impl(&struct_name, &fields);
    
    TokenStream::from(quote! {
        impl DetectionMerger for #struct_name {
            #merge_impl
        }
    })
}

/// Custom attribute macro for detection_merge
/// 
/// This attribute can be used on struct fields to specify how they should be merged.
#[proc_macro_attribute]
pub fn detection_merge(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    // For now, this is just a marker attribute
    // The actual parsing happens in the derive macro
    _item
}

#[derive(Debug)]
struct FieldMapping {
    field_name: String,
    mapping_type: MappingType,
    field_type: FieldType,
}

#[derive(Debug)]
enum MappingType {
    Contexts,
    Facets,
    Traits,
    Evidence,
    Ignore,
}

#[derive(Debug)]
enum FieldType {
    Contexts,
    Facets,
    Traits,
    Evidence,
    Other,
}

fn parse_fields(data: &syn::Data) -> Vec<FieldMapping> {
    match data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => parse_named_fields(fields),
            Fields::Unnamed(fields) => parse_unnamed_fields(fields),
            Fields::Unit => vec![],
        },
        _ => vec![],
    }
}

fn parse_named_fields(fields: &FieldsNamed) -> Vec<FieldMapping> {
    fields.named.iter().filter_map(parse_field).collect()
}

fn parse_unnamed_fields(_fields: &FieldsUnnamed) -> Vec<FieldMapping> {
    // For now, we only support named fields
    vec![]
}

fn parse_field(field: &Field) -> Option<FieldMapping> {
    let field_name = field.ident.as_ref()?.to_string();
    
    // Determine field type based on the type path
    let field_type = detect_field_type(field);
    
    // Map based on field name
    match field_name.as_str() {
        "contexts" => Some(FieldMapping {
            field_name,
            mapping_type: MappingType::Contexts,
            field_type,
        }),
        "facets" => Some(FieldMapping {
            field_name,
            mapping_type: MappingType::Facets,
            field_type,
        }),
        "traits" => Some(FieldMapping {
            field_name,
            mapping_type: MappingType::Traits,
            field_type,
        }),
        "evidence" => Some(FieldMapping {
            field_name,
            mapping_type: MappingType::Evidence,
            field_type,
        }),
        _ => Some(FieldMapping {
            field_name,
            mapping_type: MappingType::Ignore,
            field_type: FieldType::Other,
        }),
    }
}

fn detect_field_type(field: &Field) -> FieldType {
    // Simple type detection based on the type path
    if let syn::Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "Contexts" => FieldType::Contexts,
                "Facets" => FieldType::Facets,
                "Traits" => FieldType::Traits,
                "Vec" => FieldType::Evidence, // Assuming Vec is evidence
                _ => FieldType::Other,
            }
        } else {
            FieldType::Other
        }
    } else {
        FieldType::Other
    }
}

fn generate_merge_impl(_struct_name: &syn::Ident, fields: &[FieldMapping]) -> proc_macro2::TokenStream {
    let mut merge_statements = Vec::new();
    
    // Generate data collection
    merge_statements.push(quote! {
        let mut all_contexts = std::collections::HashSet::new();
        let mut all_traits: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
        let mut all_facets: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

        // Collect all detection data
        for detection in detections {
            all_contexts.extend(detection.contexts_add.iter().cloned());
            all_traits.extend(detection.traits_patch.clone());
            all_facets.extend(detection.facets_patch.clone());
        }
    });
    
    // Generate field-specific merging logic
    for field in fields {
        let field_name = syn::Ident::new(&field.field_name, proc_macro2::Span::call_site());
        
        match (&field.mapping_type, &field.field_type) {
            (MappingType::Contexts, FieldType::Contexts) => {
                merge_statements.push(quote! {
                    // Merge contexts - set boolean fields based on presence in all_contexts
                    self.#field_name.agent = all_contexts.contains("agent");
                    self.#field_name.ide = all_contexts.contains("ide");
                    self.#field_name.ci = all_contexts.contains("ci");
                    self.#field_name.container = all_contexts.contains("container");
                    self.#field_name.remote = all_contexts.contains("remote");
                });
            }
            (MappingType::Facets, FieldType::Facets) => {
                merge_statements.push(quote! {
                    // Merge facets - extract string values from all_facets
                    if let Some(value) = all_facets.get("agent_id").and_then(|v| v.as_str()) {
                        self.#field_name.agent_id = Some(value.to_string());
                    }
                    if let Some(value) = all_facets.get("ide_id").and_then(|v| v.as_str()) {
                        self.#field_name.ide_id = Some(value.to_string());
                    }
                    if let Some(value) = all_facets.get("ci_id").and_then(|v| v.as_str()) {
                        self.#field_name.ci_id = Some(value.to_string());
                    }
                    if let Some(value) = all_facets.get("container_id").and_then(|v| v.as_str()) {
                        self.#field_name.container_id = Some(value.to_string());
                    }
                    // Handle CI facet struct
                    if let Some(ci_facet_value) = all_facets.get("ci") {
                        if let Ok(ci_facet) = serde_json::from_value::<crate::ci::CiFacet>(ci_facet_value.clone()) {
                            self.#field_name.ci = ci_facet;
                        }
                    }
                });
            }
            (MappingType::Traits, FieldType::Traits) => {
                merge_statements.push(quote! {
                    // Merge traits - extract boolean and enum values from all_traits
                    if let Some(value) = all_traits.get("is_interactive").and_then(|v| v.as_bool()) {
                        self.#field_name.is_interactive = value;
                    }
                    if let Some(value) = all_traits.get("is_tty_stdin").and_then(|v| v.as_bool()) {
                        self.#field_name.is_tty_stdin = value;
                    }
                    if let Some(value) = all_traits.get("is_tty_stdout").and_then(|v| v.as_bool()) {
                        self.#field_name.is_tty_stdout = value;
                    }
                    if let Some(value) = all_traits.get("is_tty_stderr").and_then(|v| v.as_bool()) {
                        self.#field_name.is_tty_stderr = value;
                    }
                    if let Some(value) = all_traits.get("is_piped_stdin").and_then(|v| v.as_bool()) {
                        self.#field_name.is_piped_stdin = value;
                    }
                    if let Some(value) = all_traits.get("is_piped_stdout").and_then(|v| v.as_bool()) {
                        self.#field_name.is_piped_stdout = value;
                    }
                    if let Some(value) = all_traits.get("supports_hyperlinks").and_then(|v| v.as_bool()) {
                        self.#field_name.supports_hyperlinks = value;
                    }
                    // Handle color level enum
                    if let Some(color_level_str) = all_traits.get("color_level").and_then(|v| v.as_str()) {
                        self.#field_name.color_level = match color_level_str {
                            "none" => crate::traits::terminal::ColorLevel::None,
                            "ansi16" => crate::traits::terminal::ColorLevel::Ansi16,
                            "ansi256" => crate::traits::terminal::ColorLevel::Ansi256,
                            "truecolor" => crate::traits::terminal::ColorLevel::Truecolor,
                            _ => crate::traits::terminal::ColorLevel::None,
                        };
                    }
                });
            }
            (MappingType::Evidence, FieldType::Evidence) => {
                merge_statements.push(quote! {
                    // Merge evidence - convert from serde_json::Value back to Evidence
                    for detection in detections {
                        for evidence_value in &detection.evidence {
                            if let Ok(evidence) = serde_json::from_value::<crate::schema::Evidence>(evidence_value.clone()) {
                                self.#field_name.push(evidence);
                            }
                        }
                    }
                });
            }
            _ => {
                // Do nothing for mismatched or ignored fields
            }
        }
    }
    
    quote! {
        fn merge_detections(&mut self, detections: &[Detection]) {
            #(#merge_statements)*
        }
    }
}

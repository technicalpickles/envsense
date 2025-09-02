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
    NestedTraits, // New: for nested trait structures
    Evidence,
    SimpleBool,     // New: for simple boolean fields
    OptionalString, // New: for Option<String> fields like host
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
    let mut field_type = detect_field_type(field);

    // Refine field type based on field name and actual type
    if matches!(field_type, FieldType::Other) {
        if let syn::Type::Path(type_path) = &field.ty {
            if let Some(segment) = type_path.path.segments.last() {
                match (field_name.as_str(), segment.ident.to_string().as_str()) {
                    ("contexts", "Vec") => field_type = FieldType::Other, // Vec<String> contexts
                    ("evidence", "Vec") => field_type = FieldType::Evidence, // Vec<Evidence>
                    ("contexts", "bool") => field_type = FieldType::SimpleBool, // Simple bool contexts
                    ("contexts", _) => field_type = FieldType::Contexts,        // Contexts struct
                    _ => {}
                }
            }
        }
    }

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
        "host" => Some(FieldMapping {
            field_name,
            mapping_type: MappingType::Facets,
            field_type: FieldType::OptionalString,
        }),
        _ => Some(FieldMapping {
            field_name,
            mapping_type: MappingType::Ignore,
            field_type: FieldType::Other,
        }),
    }
}

fn detect_field_type(field: &Field) -> FieldType {
    // Enhanced type detection for nested structures
    if let syn::Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "Contexts" => FieldType::Contexts,
                "Facets" => FieldType::Facets,
                "Traits" => FieldType::Traits,
                "TerminalTraits" => FieldType::Traits, // Flat terminal traits
                "NestedTraits" => FieldType::NestedTraits, // New: detect nested traits
                "Vec" => {
                    // Check if this is Vec<String> for contexts or Vec<Evidence> for evidence
                    // For now, we'll determine this based on the field name in parse_field
                    FieldType::Other // Will be refined in parse_field based on field name
                }
                _ => FieldType::Other,
            }
        } else {
            FieldType::Other
        }
    } else {
        FieldType::Other
    }
}

/// Helper function to generate nested field merging logic
fn generate_nested_trait_merge(field_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        // Merge nested traits - handle both nested objects and flat keys

        // Agent traits - handle both nested object and flat key formats
        if let Some(agent_obj) = all_traits.get("agent").and_then(|v| v.as_object()) {
            if let Some(id) = agent_obj.get("id").and_then(|v| v.as_str()) {
                self.#field_name.agent.id = Some(id.to_string());
            }
        } else if let Some(value) = all_traits.get("agent.id").and_then(|v| v.as_str()) {
            self.#field_name.agent.id = Some(value.to_string());
        }

        // IDE traits - handle both nested object and flat key formats
        if let Some(ide_obj) = all_traits.get("ide").and_then(|v| v.as_object()) {
            if let Some(id) = ide_obj.get("id").and_then(|v| v.as_str()) {
                self.#field_name.ide.id = Some(id.to_string());
            }
        } else if let Some(value) = all_traits.get("ide.id").and_then(|v| v.as_str()) {
            self.#field_name.ide.id = Some(value.to_string());
        }

        // CI traits - handle both nested object and flat key formats
        if let Some(ci_obj) = all_traits.get("ci").and_then(|v| v.as_object()) {
            if let Some(id) = ci_obj.get("id").and_then(|v| v.as_str()) {
                self.#field_name.ci.id = Some(id.to_string());
            }
            if let Some(vendor) = ci_obj.get("vendor").and_then(|v| v.as_str()) {
                self.#field_name.ci.vendor = Some(vendor.to_string());
            }
            if let Some(name) = ci_obj.get("name").and_then(|v| v.as_str()) {
                self.#field_name.ci.name = Some(name.to_string());
            }
            if let Some(is_pr) = ci_obj.get("is_pr").and_then(|v| v.as_bool()) {
                self.#field_name.ci.is_pr = Some(is_pr);
            }
            if let Some(branch) = ci_obj.get("branch").and_then(|v| v.as_str()) {
                self.#field_name.ci.branch = Some(branch.to_string());
            }
        } else {
            // Fallback to flat key format
            if let Some(value) = all_traits.get("ci.id").and_then(|v| v.as_str()) {
                self.#field_name.ci.id = Some(value.to_string());
            }
            if let Some(value) = all_traits.get("ci.vendor").and_then(|v| v.as_str()) {
                self.#field_name.ci.vendor = Some(value.to_string());
            }
            if let Some(value) = all_traits.get("ci.name").and_then(|v| v.as_str()) {
                self.#field_name.ci.name = Some(value.to_string());
            }
            if let Some(value) = all_traits.get("ci.is_pr").and_then(|v| v.as_bool()) {
                self.#field_name.ci.is_pr = Some(value);
            }
            if let Some(value) = all_traits.get("ci.branch").and_then(|v| v.as_str()) {
                self.#field_name.ci.branch = Some(value.to_string());
            }
        }

        // Terminal traits - handle both nested object and flat key formats
        if let Some(terminal_obj) = all_traits.get("terminal").and_then(|v| v.as_object()) {
            // Handle nested terminal object
            if let Some(interactive) = terminal_obj.get("interactive").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.interactive = interactive;
            }
            if let Some(stdin_obj) = terminal_obj.get("stdin").and_then(|v| v.as_object()) {
                if let Some(tty) = stdin_obj.get("tty").and_then(|v| v.as_bool()) {
                    self.#field_name.terminal.stdin.tty = tty;
                }
                if let Some(piped) = stdin_obj.get("piped").and_then(|v| v.as_bool()) {
                    self.#field_name.terminal.stdin.piped = piped;
                }
            }
            if let Some(stdout_obj) = terminal_obj.get("stdout").and_then(|v| v.as_object()) {
                if let Some(tty) = stdout_obj.get("tty").and_then(|v| v.as_bool()) {
                    self.#field_name.terminal.stdout.tty = tty;
                }
                if let Some(piped) = stdout_obj.get("piped").and_then(|v| v.as_bool()) {
                    self.#field_name.terminal.stdout.piped = piped;
                }
            }
            if let Some(stderr_obj) = terminal_obj.get("stderr").and_then(|v| v.as_object()) {
                if let Some(tty) = stderr_obj.get("tty").and_then(|v| v.as_bool()) {
                    self.#field_name.terminal.stderr.tty = tty;
                }
                if let Some(piped) = stderr_obj.get("piped").and_then(|v| v.as_bool()) {
                    self.#field_name.terminal.stderr.piped = piped;
                }
            }
            if let Some(supports_hyperlinks) = terminal_obj.get("supports_hyperlinks").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.supports_hyperlinks = supports_hyperlinks;
            }
            if let Some(color_level_str) = terminal_obj.get("color_level").and_then(|v| v.as_str()) {
                if let Ok(color_level) = serde_json::from_str::<serde_json::Value>(&format!("\"{}\"", color_level_str))
                    .and_then(|v| serde_json::from_value(v)) {
                    self.#field_name.terminal.color_level = color_level;
                }
            }
        } else {
            // Fallback to flat key format for all terminal fields
            if let Some(value) = all_traits.get("terminal.interactive").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.interactive = value;
            }
            if let Some(value) = all_traits.get("terminal.stdin.tty").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stdin.tty = value;
            }
            if let Some(value) = all_traits.get("terminal.stdin.piped").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stdin.piped = value;
            }
            if let Some(value) = all_traits.get("terminal.stdout.tty").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stdout.tty = value;
            }
            if let Some(value) = all_traits.get("terminal.stdout.piped").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stdout.piped = value;
            }
            if let Some(value) = all_traits.get("terminal.stderr.tty").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stderr.tty = value;
            }
            if let Some(value) = all_traits.get("terminal.stderr.piped").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stderr.piped = value;
            }
            if let Some(value) = all_traits.get("terminal.supports_hyperlinks").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.supports_hyperlinks = value;
            }
            // Handle color level enum
            if let Some(color_level_str) = all_traits.get("terminal.color_level").and_then(|v| v.as_str()) {
                // Parse string to enum - this will work regardless of import context
                if let Ok(color_level) = serde_json::from_str::<serde_json::Value>(&format!("\"{}\"", color_level_str))
                    .and_then(|v| serde_json::from_value(v)) {
                    self.#field_name.terminal.color_level = color_level;
                }
            }
        }

        // CI traits
        if let Some(value) = all_traits.get("ci.id").and_then(|v| v.as_str()) {
            self.#field_name.ci.id = Some(value.to_string());
        }
        if let Some(value) = all_traits.get("ci.vendor").and_then(|v| v.as_str()) {
            self.#field_name.ci.vendor = Some(value.to_string());
        }
        if let Some(value) = all_traits.get("ci.name").and_then(|v| v.as_str()) {
            self.#field_name.ci.name = Some(value.to_string());
        }
        if let Some(value) = all_traits.get("ci.is_pr").and_then(|v| v.as_bool()) {
            self.#field_name.ci.is_pr = Some(value);
        }
        if let Some(value) = all_traits.get("ci.branch").and_then(|v| v.as_str()) {
            self.#field_name.ci.branch = Some(value.to_string());
        }

        // Backward compatibility: handle flat trait keys for migration (only if nested key not present)
        if !all_traits.contains_key("terminal.interactive") {
            if let Some(value) = all_traits.get("is_interactive").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.interactive = value;
            }
        }
        if !all_traits.contains_key("terminal.stdin.tty") {
            if let Some(value) = all_traits.get("is_tty_stdin").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stdin.tty = value;
            }
        }
        if !all_traits.contains_key("terminal.stdout.tty") {
            if let Some(value) = all_traits.get("is_tty_stdout").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stdout.tty = value;
            }
        }
        if !all_traits.contains_key("terminal.stderr.tty") {
            if let Some(value) = all_traits.get("is_tty_stderr").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stderr.tty = value;
            }
        }
        if !all_traits.contains_key("terminal.stdin.piped") {
            if let Some(value) = all_traits.get("is_piped_stdin").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stdin.piped = value;
            }
        }
        if !all_traits.contains_key("terminal.stdout.piped") {
            if let Some(value) = all_traits.get("is_piped_stdout").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.stdout.piped = value;
            }
        }
        if !all_traits.contains_key("terminal.supports_hyperlinks") {
            if let Some(value) = all_traits.get("supports_hyperlinks").and_then(|v| v.as_bool()) {
                self.#field_name.terminal.supports_hyperlinks = value;
            }
        }
        if !all_traits.contains_key("terminal.color_level") {
            if let Some(color_level_str) = all_traits.get("color_level").and_then(|v| v.as_str()) {
                // Parse string to enum - this will work regardless of import context
                if let Ok(color_level) = serde_json::from_str::<serde_json::Value>(&format!("\"{}\"", color_level_str))
                    .and_then(|v| serde_json::from_value(v)) {
                    self.#field_name.terminal.color_level = color_level;
                }
            }
        }
    }
}

fn generate_merge_impl(
    _struct_name: &syn::Ident,
    fields: &[FieldMapping],
) -> proc_macro2::TokenStream {
    let mut merge_statements = Vec::new();

    // Generate data collection
    merge_statements.push(quote! {
        let mut all_contexts = Vec::new();
        let mut all_traits: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
        let mut all_facets: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

        // Collect all detection data
        for detection in detections {
            for context in &detection.contexts_add {
                if !all_contexts.contains(context) {
                    all_contexts.push(context.clone());
                }
            }
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
                    self.#field_name.agent = all_contexts.contains(&"agent".to_string());
                    self.#field_name.ide = all_contexts.contains(&"ide".to_string());
                    self.#field_name.ci = all_contexts.contains(&"ci".to_string());
                    self.#field_name.container = all_contexts.contains(&"container".to_string());
                    self.#field_name.remote = all_contexts.contains(&"remote".to_string());
                });
            }
            (MappingType::Contexts, FieldType::SimpleBool) => {
                // Handle simple boolean contexts field
                merge_statements.push(quote! {
                    // Merge contexts - set boolean to true if any contexts exist
                    self.#field_name = !all_contexts.is_empty();
                });
            }
            (MappingType::Contexts, FieldType::Other) => {
                // Handle Vec<String> contexts field
                merge_statements.push(quote! {
                    // Merge contexts - extend Vec<String> with all contexts
                    self.#field_name.extend(all_contexts);
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
                    if let Some(value) = all_facets.get("host").and_then(|v| v.as_str()) {
                        self.#field_name.host = Some(value.to_string());
                    }
                    // Legacy CI facet handling removed - CI information now comes from declarative detection
                });
            }
            (MappingType::Traits, FieldType::Traits) => {
                merge_statements.push(quote! {
                    // Merge traits - handle both legacy flat traits and TerminalTraits

                    // Handle TerminalTraits structure
                    if let Some(value) = all_traits.get("is_interactive").and_then(|v| v.as_bool()) {
                        self.#field_name.interactive = value;
                    }
                    if let Some(value) = all_traits.get("is_tty_stdin").and_then(|v| v.as_bool()) {
                        self.#field_name.stdin.tty = value;
                    }
                    if let Some(value) = all_traits.get("is_tty_stdout").and_then(|v| v.as_bool()) {
                        self.#field_name.stdout.tty = value;
                    }
                    if let Some(value) = all_traits.get("is_tty_stderr").and_then(|v| v.as_bool()) {
                        self.#field_name.stderr.tty = value;
                    }
                    if let Some(value) = all_traits.get("is_piped_stdin").and_then(|v| v.as_bool()) {
                        self.#field_name.stdin.piped = value;
                    }
                    if let Some(value) = all_traits.get("is_piped_stdout").and_then(|v| v.as_bool()) {
                        self.#field_name.stdout.piped = value;
                    }
                    if let Some(value) = all_traits.get("supports_hyperlinks").and_then(|v| v.as_bool()) {
                        self.#field_name.supports_hyperlinks = value;
                    }
                    // Handle color level enum
                    if let Some(color_level_str) = all_traits.get("color_level").and_then(|v| v.as_str()) {
                        // Parse string to enum - this will work regardless of import context
                        if let Ok(color_level) = serde_json::from_str::<serde_json::Value>(&format!("\"{}\"", color_level_str))
                            .and_then(|v| serde_json::from_value(v)) {
                            self.#field_name.color_level = color_level;
                        }
                    }

                    // Legacy flat traits support (for old Traits struct if still used)
                    // These fields might not exist on TerminalTraits, so we ignore compilation errors
                    // by using a more generic approach or conditional compilation
                });
            }
            (MappingType::Traits, FieldType::NestedTraits) => {
                // New: Handle nested traits structure
                let nested_merge = generate_nested_trait_merge(&field_name);
                merge_statements.push(nested_merge);
            }
            (MappingType::Evidence, FieldType::Evidence) => {
                merge_statements.push(quote! {
                    // Merge evidence - convert from serde_json::Value back to Evidence
                    for detection in detections {
                        for evidence_value in &detection.evidence {
                            // Try to deserialize as Evidence - this will work regardless of import context
                            if let Ok(evidence) = serde_json::from_value(evidence_value.clone()) {
                                self.#field_name.push(evidence);
                            }
                        }
                    }
                });
            }
            (MappingType::Facets, FieldType::OptionalString) => {
                // Handle standalone optional string fields like host
                let field_name_str = field_name.to_string();
                merge_statements.push(quote! {
                    // Merge host field from facets
                    if let Some(value) = all_facets.get(#field_name_str).and_then(|v| v.as_str()) {
                        self.#field_name = Some(value.to_string());
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

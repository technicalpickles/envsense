//! Procedural macros for envsense detection merging
//!
//! This crate provides the `DetectionMerger` derive macro that automatically
//! generates merging logic for the envsense detection engine.
//!
//! # Usage
//!
//! Apply the `#[derive(DetectionMergerDerive)]` macro to your struct:
//!
//! ```rust
//! use envsense_macros::{DetectionMergerDerive, DetectionMerger, Detection};
//! use std::collections::HashMap;
//!
//! // Simple example struct
//! #[derive(DetectionMergerDerive, Default)]
//! pub struct SimpleStruct {
//!     pub contexts: bool,
//!     pub facets: String,
//!     pub traits: bool,
//! }
//!
//! // The macro automatically implements DetectionMerger trait
//! let mut simple = SimpleStruct::default();
//! let detections = vec![
//!     Detection {
//!         contexts_add: vec!["agent".to_string()],
//!         traits_patch: HashMap::new(),
//!         facets_patch: HashMap::new(),
//!         evidence: vec![],
//!         confidence: 1.0,
//!     }
//! ];
//!
//! simple.merge_detections(&detections);
//! ```
//!
//! # Field Mapping
//!
//! The macro automatically maps fields based on their names and types:
//!
//! - **`contexts`**: Maps to `contexts_add` from detections
//! - **`facets`**: Maps to `facets_patch` from detections  
//! - **`traits`**: Maps to `traits_patch` from detections
//! - **`evidence`**: Maps to `evidence` from detections
//! - **Other fields**: Ignored (no mapping applied)
//!
//! # Supported Types
//!
//! The macro handles various field types automatically:
//!
//! - **Boolean fields**: Direct assignment from detection values
//! - **String fields**: Extraction and assignment from detection values
//! - **Enum fields**: String-to-enum conversion (e.g., ColorLevel)
//! - **Struct fields**: JSON deserialization (e.g., CiFacet)
//! - **Collection fields**: Extend with detection values (e.g., Vec<Evidence>)
//!
//! # Benefits
//!
//! - **Reduced complexity**: 80+ lines of manual merging → ~20 lines of macro annotations
//! - **Type safety**: Compile-time validation of field mappings
//! - **Maintainability**: Automatic field mapping reduces maintenance burden
//! - **Extensibility**: Easy to add new detector fields without manual merging code

mod detection_merger; // Contains DetectionMerger trait and Detection struct

pub use detection_merger::{Detection, DetectionMerger};

// Re-export the derive macro
pub use envsense_macros_impl::DetectionMerger as DetectionMergerDerive;

// Re-export the attribute macro
pub use envsense_macros_impl::detection_merge;

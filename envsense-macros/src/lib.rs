//! Procedural macros for envsense detection merging
//! 
//! This crate provides the `DetectionMerger` derive macro that automatically
//! generates merging logic for the envsense detection engine.

mod detection_merger;

pub use detection_merger::{DetectionMerger, Detection};

// Re-export the derive macro
pub use envsense_macros_impl::DetectionMerger as DetectionMergerDerive;

// Re-export the attribute macro
pub use envsense_macros_impl::detection_merge;

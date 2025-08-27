//! Basic test for the DetectionMerger macro

use envsense_macros::{DetectionMerger, Detection, DetectionMergerDerive};

#[derive(DetectionMergerDerive, Default, Debug)]
struct TestStruct {
    pub contexts: bool,
    pub facets: String,
    pub traits: bool,
}

#[test]
fn test_macro_compiles() {
    let mut test = TestStruct::default();
    let detections = vec![
        Detection {
            contexts_add: vec!["test".to_string()],
            traits_patch: std::collections::HashMap::new(),
            facets_patch: std::collections::HashMap::new(),
            evidence: vec![],
            confidence: 1.0,
        }
    ];
    
    // This should compile even if the implementation is just a placeholder
    test.merge_detections(&detections);
    
    // For now, just verify the struct exists and has the expected fields
    assert_eq!(test.contexts, false);
    assert_eq!(test.facets, "");
    assert_eq!(test.traits, false);
}

//! Basic test for the DetectionMerger macro

use envsense_macros::{Detection, DetectionMerger, DetectionMergerDerive};

#[derive(DetectionMergerDerive, Default, Debug)]
struct TestStruct {
    pub contexts: bool,
    pub facets: String,
    pub traits: bool,
}

#[test]
fn test_macro_compiles() {
    let mut test = TestStruct::default();
    let detections = vec![Detection {
        contexts_add: vec!["test".to_string()],
        traits_patch: std::collections::HashMap::new(),
        facets_patch: std::collections::HashMap::new(),
        evidence: vec![],
        confidence: 1.0,
    }];

    // This should compile even if the implementation is just a placeholder
    test.merge_detections(&detections);

    // Verify that the macro correctly merges the detection data
    assert!(test.contexts); // Should be true because contexts were added
    assert_eq!(test.facets, ""); // Should remain empty (no facets in detection)
    assert!(!test.traits); // Should remain false (no traits in detection)
}

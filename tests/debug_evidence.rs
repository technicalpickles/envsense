use envsense::schema::EnvSense;

#[test]
fn debug_evidence_generation() {
    let envsense = EnvSense::detect();

    println!("Evidence count: {}", envsense.evidence.len());
    for (i, evidence) in envsense.evidence.iter().enumerate() {
        println!("Evidence {}: {:?}", i, evidence);
    }

    let json = serde_json::to_string_pretty(&envsense).unwrap();
    println!("\nFull JSON:");
    println!("{}", json);

    // Check if evidence is in JSON
    assert!(
        json.contains("\"evidence\""),
        "Evidence field should be in JSON"
    );
}

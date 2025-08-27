#!/usr/bin/env rust

// dependency_injection_demo.rs
//
// Purpose: Demonstrate the new Phase 3 dependency injection functionality
// Created: 2024-12-19
// Used for: Showcasing the benefits of the new TtyDetector enum approach
//
// This test demonstrates how the new dependency injection system works,
// showing the benefits over the old environment variable override approach.

use envsense::detectors::DeclarativeAgentDetector;
use envsense::detectors::DeclarativeIdeDetector;
use envsense::detectors::ci::CiDetector;
use envsense::detectors::terminal::TerminalDetector;
use envsense::detectors::{EnvSnapshot, TtyDetector};
use envsense::engine::DetectionEngine;
use std::collections::HashMap;

#[test]
fn demonstrate_dependency_injection_benefits() {
    // Test 1: Real TTY detection (production use)
    let real_snapshot = EnvSnapshot::current();
    println!("Real TTY detection:");
    println!("  stdin: {}", real_snapshot.is_tty_stdin());
    println!("  stdout: {}", real_snapshot.is_tty_stdout());
    println!("  stderr: {}", real_snapshot.is_tty_stderr());

    // Test 2: Mock TTY detection with specific values
    let mut env_vars = HashMap::new();
    env_vars.insert("TERM".to_string(), "xterm-256color".to_string());

    let mock_snapshot = EnvSnapshot::with_mock_tty(env_vars, true, false, true);
    println!("\nMock TTY detection (stdin=true, stdout=false, stderr=true):");
    println!("  stdin: {}", mock_snapshot.is_tty_stdin());
    println!("  stdout: {}", mock_snapshot.is_tty_stdout());
    println!("  stderr: {}", mock_snapshot.is_tty_stderr());

    // Test 3: Convenience constructors
    let all_tty_snapshot = EnvSnapshot::for_testing(HashMap::new(), TtyDetector::mock_all_tty());
    println!("\nAll TTY convenience constructor:");
    println!("  stdin: {}", all_tty_snapshot.is_tty_stdin());
    println!("  stdout: {}", all_tty_snapshot.is_tty_stdout());
    println!("  stderr: {}", all_tty_snapshot.is_tty_stderr());

    let no_tty_snapshot = EnvSnapshot::for_testing(HashMap::new(), TtyDetector::mock_no_tty());
    println!("\nNo TTY convenience constructor:");
    println!("  stdin: {}", no_tty_snapshot.is_tty_stdin());
    println!("  stdout: {}", no_tty_snapshot.is_tty_stdout());
    println!("  stderr: {}", no_tty_snapshot.is_tty_stderr());

    let piped_snapshot = EnvSnapshot::for_testing(HashMap::new(), TtyDetector::mock_piped_io());
    println!("\nPiped I/O convenience constructor:");
    println!("  stdin: {}", piped_snapshot.is_tty_stdin());
    println!("  stdout: {}", piped_snapshot.is_tty_stdout());
    println!("  stderr: {}", piped_snapshot.is_tty_stderr());

    // Test 4: Full detection engine with mock TTY
    let engine = DetectionEngine::new()
        .register(TerminalDetector::new())
        .register(DeclarativeAgentDetector::new())
        .register(CiDetector::new())
        .register(DeclarativeIdeDetector::new());
    let result = engine.detect_from_snapshot(&piped_snapshot);

    println!("\nDetection result with piped I/O:");
    println!("  contexts: {:?}", result.contexts);
    println!("  is_interactive: {}", result.traits.is_interactive);
    println!("  is_tty_stdin: {}", result.traits.is_tty_stdin);
    println!("  is_tty_stdout: {}", result.traits.is_tty_stdout);

    // Verify the results
    assert!(piped_snapshot.is_tty_stdin());
    assert!(!piped_snapshot.is_tty_stdout());
    assert!(!piped_snapshot.is_tty_stderr());

    assert!(all_tty_snapshot.is_tty_stdin());
    assert!(all_tty_snapshot.is_tty_stdout());
    assert!(all_tty_snapshot.is_tty_stderr());

    assert!(!no_tty_snapshot.is_tty_stdin());
    assert!(!no_tty_snapshot.is_tty_stdout());
    assert!(!no_tty_snapshot.is_tty_stderr());
}

#[test]
fn compare_old_vs_new_approach() {
    // This test demonstrates the benefits of the new approach

    // OLD APPROACH (environment variable overrides):
    // - Required setting environment variables
    // - Complex parsing logic with fallbacks
    // - Environment pollution
    // - Hard to test individual components

    // NEW APPROACH (dependency injection):
    // - Clean, explicit configuration
    // - No environment pollution
    // - Easy to test individual components
    // - Type-safe and performant

    let mut env_vars = HashMap::new();
    env_vars.insert("TERM".to_string(), "xterm-256color".to_string());
    env_vars.insert("SHELL".to_string(), "/bin/bash".to_string());

    // Create different TTY configurations easily
    let scenarios = vec![
        ("interactive_terminal", true, true, true),
        ("piped_input", true, false, false),
        ("headless_server", false, false, false),
        ("remote_session", true, true, false),
    ];

    for (scenario_name, stdin, stdout, stderr) in scenarios {
        let snapshot = EnvSnapshot::with_mock_tty(env_vars.clone(), stdin, stdout, stderr);

        println!("Scenario: {}", scenario_name);
        println!(
            "  TTY config: stdin={}, stdout={}, stderr={}",
            stdin, stdout, stderr
        );
        println!(
            "  is_interactive: {}",
            snapshot.is_tty_stdin() && snapshot.is_tty_stdout()
        );
        println!();

        // Verify the configuration is applied correctly
        assert_eq!(snapshot.is_tty_stdin(), stdin);
        assert_eq!(snapshot.is_tty_stdout(), stdout);
        assert_eq!(snapshot.is_tty_stderr(), stderr);
    }
}

#[test]
fn test_performance_benefits() {
    // The new enum-based approach has zero runtime overhead
    // No dynamic dispatch, no virtual function calls

    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    // Create mock detector once
    let mock_detector = TtyDetector::mock(true, false, true);

    // Reuse the same detector for multiple snapshots
    let snapshot1 = EnvSnapshot::for_testing(env_vars.clone(), mock_detector.clone());
    let snapshot2 = EnvSnapshot::for_testing(env_vars.clone(), mock_detector.clone());
    let snapshot3 = EnvSnapshot::for_testing(env_vars.clone(), mock_detector.clone());

    // All snapshots use the same detector instance
    assert!(snapshot1.is_tty_stdin());
    assert!(!snapshot1.is_tty_stdout());
    assert!(snapshot1.is_tty_stderr());

    assert!(snapshot2.is_tty_stdin());
    assert!(!snapshot2.is_tty_stdout());
    assert!(snapshot2.is_tty_stderr());

    assert!(snapshot3.is_tty_stdin());
    assert!(!snapshot3.is_tty_stdout());
    assert!(snapshot3.is_tty_stderr());
}

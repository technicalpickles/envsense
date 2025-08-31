# EnvSense Documentation

This directory contains the current, actively maintained documentation for the
EnvSense project.

## 📁 Current Documentation

### **Core Documentation**

- **[Architecture](architecture.md)** - Current system architecture and design
- **[Testing](testing.md)** - Testing guidelines and best practices
- **[Extending](extending.md)** - How to extend EnvSense with new detectors
- **[Script Integration](script-integration.md)** - Integration with external
  scripts
- **[Terminal Traits](terminal-traits.md)** - Terminal capability detection

### **Macro System Documentation**

- **[Macro Migration Guide](macro-migration-guide.md)** - Complete guide for
  migrating to macro-based merging
- **[Macro API Documentation](../envsense-macros/src/lib.rs)** - API
  documentation for the macro system

### **Development Documentation**

- **[Debugging CI](debugging-ci.md)** - CI/CD debugging and troubleshooting

## 🏗️ Architecture Overview

EnvSense uses a macro-based detection engine that automatically merges detection
results from multiple detectors. The system provides:

- **Automatic field mapping** based on field names and types
- **Type-safe merging** with compile-time validation
- **Zero runtime overhead** with optimized generated code
- **Easy extensibility** for new detector fields

### **Key Components**

#### **Detection Engine**

- Automatically merges detection results from multiple detectors
- Uses macro-generated code for type-safe field mapping
- Supports contexts, facets, traits, and evidence merging

#### **Macro System**

- `envsense-macros/` - Library crate with public API
- `envsense-macros/envsense-macros-impl/` - Proc-macro implementation
- Automatic field mapping based on struct field names and types

#### **Detectors**

- **Terminal Detector**: Detects terminal capabilities and TTY status
- **Agent Detector**: Detects development environment agents
- **IDE Detector**: Detects integrated development environments
- **CI Detector**: Detects continuous integration environments

## 🚀 Quick Start

### **Using the Macro System**

```rust
use envsense_macros::{DetectionMergerDerive, DetectionMerger};

#[derive(DetectionMergerDerive)]
pub struct EnvSense {
    pub contexts: Contexts,      // Maps to contexts_add
    pub facets: Facets,         // Maps to facets_patch
    pub traits: Traits,         // Maps to traits_patch
    pub evidence: Vec<Evidence>, // Maps to evidence
    pub version: String,        // Ignored (no mapping)
}

// Usage: Automatic merging with 1 line
result.merge_detections(&detections);
```

### **Adding New Detectors**

1. Implement the `Detector` trait
2. Register with `DetectionEngine::register()`
3. The macro automatically handles field mapping

### **Testing**

```bash
# Run all tests
cargo test

# Run macro-specific tests
cargo test macro

# Run performance benchmarks
cargo test macro_performance_test
```

## 📚 Historical Documentation

For historical planning documents and completed work, see:

- **[Documentation Archive](archive/)** - Completed planning and implementation
  documents

## 🔧 Development

### **Building**

```bash
cargo build
cargo build --release
```

### **Testing**

```bash
cargo test
cargo test --release
```

### **Documentation**

```bash
cargo doc --open
```

## 📈 Performance

- **Detection Time**: ~7ms for full environment detection
- **Macro Merging**: 4.6ms for 1000 detections
- **Memory Usage**: No increase over manual merging
- **Compile Time**: Minimal impact from macro processing

## 🎯 Key Features

- **60-80% code reduction** in engine merging logic
- **Compile-time safety** with type-safe field mappings
- **Automatic field mapping** for new detector fields
- **Zero runtime overhead** with optimized generated code
- **Comprehensive testing** with 100% test coverage
- **Production ready** with full backward compatibility

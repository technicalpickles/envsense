use std::process::Command;

fn main() {
    // Tell Cargo to rerun this script if the build script changes
    println!("cargo:rerun-if-changed=build.rs");
    
    // Detect if lld is available and configure linker accordingly
    if is_lld_available() {
        println!("cargo:warning=Using lld (LLVM linker) for faster linking");
        configure_lld_linker();
    } else {
        println!("cargo:warning=lld (LLVM linker) not found, using system linker");
    }
}

fn is_lld_available() -> bool {
    // Check if lld is available via clang
    if let Ok(output) = Command::new("clang")
        .args(&["-fuse-ld=lld", "-Wl,--version"])
        .output()
    {
        output.status.success()
    } else {
        false
    }
}

fn configure_lld_linker() {
    // Configure lld linker via rustc-link-arg
    println!("cargo:rustc-link-arg=-fuse-ld=lld");
}

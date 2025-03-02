use std::process::Command;
use std::path::Path;
use std::fs;

fn main() {
    // Path to the other crate
    let shader_crate_path = "./crates/build-shaders/";
    
    // Check if the directory exists
    if !Path::new(shader_crate_path).exists() {
        panic!("Shader crate directory not found: {}", shader_crate_path);
    }

    // Check for rust-toolchain.toml
    let toolchain_path = format!("{}/rust-toolchain.toml", shader_crate_path);
    
    let output = if Path::new(&toolchain_path).exists() {
        // Read and parse the toolchain file to extract the channel
        let toolchain_content = fs::read_to_string(&toolchain_path)
            .expect("Failed to read rust-toolchain.toml");
        
        // Extract the channel/version from the TOML
        // This is a simple approach - consider using a proper TOML parser for robustness
        let channel = toolchain_content.lines()
            .find(|line| line.contains("channel"))
            .and_then(|line| line.split('=').nth(1))
            .map(|s| s.trim().trim_matches('"').trim_matches('\''))
            .unwrap_or_else(|| panic!("Failed to extract channel from rust-toolchain.toml"));
        
        println!("Using toolchain: {}", channel);
        
        // Use rustup run with the specific toolchain
        Command::new("rustup")
            .args(["run", channel, "cargo", "build", "--release"])
            .current_dir(shader_crate_path)
            .output()
            .expect("Failed to build shader crate with rustup")
    } else {
        // Fallback to regular cargo if no specific toolchain is defined
        Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(shader_crate_path)
            .output()
            .expect("Failed to build shader crate")
    };

    // Check the build result
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Print the full output for debugging
        eprintln!("Shader build stdout: {}", stdout);
        eprintln!("Shader build stderr: {}", stderr);
        
        panic!("Failed to build shader crate");
    }
    
    // Tell Cargo to re-run the build script if files in the shader crate change
    println!("cargo:rerun-if-changed=./crates/build-shaders/src");
    println!("cargo:rerun-if-changed=./crates/build-shaders/Cargo.toml");
    println!("cargo:rerun-if-changed=./crates/build-shaders/build.rs");
    println!("cargo:rerun-if-changed=./crates/build-shaders/rust-toolchain.toml");
    println!("cargo:rerun-if-changed=./crates/compute-shader/src");
    println!("cargo:rerun-if-changed=./crates/compute-shader/Cargo.toml");
}

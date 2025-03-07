#[cfg(feature = "spirv")]
use std::fs;
#[cfg(feature = "spirv")]
use std::path::Path;
#[cfg(feature = "spirv")]
use std::process::Command;
#[cfg(feature = "spirv")]
use std::str;

#[cfg(not(feature = "spirv"))]
pub fn main() {}

// Main entry point for the build script, currently only relevant for the spirv feature.
#[cfg(feature = "spirv")]
pub fn main() {
    println!("cargo:rerun-if-changed=src/processing.cl");
    println!("cargo:rerun-if-changed=src/processing.bc");
    println!("cargo:rerun-if-changed=src/processing.spv");

    let src_dir = "src";
    let cl_file = Path::new(src_dir).join("processing.cl");
    let bc_file = Path::new(src_dir).join("processing.bc");
    let spv_file = Path::new(src_dir).join("processing.spv");

    // Check if we need to recompile
    let should_compile = !bc_file.exists()
        || !spv_file.exists()
        || fs::metadata(&cl_file).unwrap().modified().unwrap()
            > fs::metadata(&spv_file)
                .unwrap_or_else(|_| fs::metadata(&cl_file).unwrap())
                .modified()
                .unwrap();

    if should_compile {
        println!("Compiling OpenCL kernel to SPIR-V...");

        // Find the highest matching version of clang and llvm-spirv
        let (clang_cmd, spirv_cmd) = find_highest_matching_version();

        // Step 1: Compile OpenCL to LLVM bitcode
        let clang_status = Command::new(&clang_cmd)
            .args([
                "-c",
                "-target",
                "spir64",
                "-O0",
                "-emit-llvm",
                "-Xclang",
                "-finclude-default-header",
                "-cl-std=CL2.0",
                cl_file.to_str().unwrap(),
                "-o",
                bc_file.to_str().unwrap(),
            ])
            .status()
            .unwrap_or_else(|_| {
                panic!("Failed to execute {}. Make sure it's installed.", clang_cmd)
            });

        if !clang_status.success() {
            panic!("Failed to compile OpenCL to LLVM bitcode");
        }

        // Step 2: Convert LLVM bitcode to SPIR-V
        let spirv_status = Command::new(&spirv_cmd)
            .args([bc_file.to_str().unwrap(), "-o", spv_file.to_str().unwrap()])
            .status()
            .unwrap_or_else(|_| {
                panic!("Failed to execute {}. Make sure it's installed.", spirv_cmd)
            });

        if !spirv_status.success() {
            panic!("Failed to convert LLVM bitcode to SPIR-V");
        }

        println!("Successfully compiled OpenCL kernel to SPIR-V");
    } else {
        println!("OpenCL kernel is up to date");
    }
}

#[cfg(feature = "spirv")]
fn find_highest_matching_version() -> (String, String) {
    // Find all available clang versions
    let clang_output = Command::new("bash")
        .args(["-c", "compgen -c | grep '^clang-[0-9]\\+$' | sort -V"])
        .output()
        .unwrap_or_else(|_| {
            // Fallback if compgen is not available
            Command::new("bash")
                .args([
                    "-c",
                    "find /usr/bin /usr/local/bin -name 'clang-[0-9]*' 2>/dev/null | sort -V",
                ])
                .output()
                .expect("Failed to find clang versions")
        });

    let clang_versions: Vec<String> = str::from_utf8(&clang_output.stdout)
        .expect("Invalid UTF-8 in clang output")
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Find all available llvm-spirv versions
    let spirv_output = Command::new("bash")
        .args(["-c", "compgen -c | grep '^llvm-spirv-[0-9]\\+$' | sort -V"])
        .output()
        .unwrap_or_else(|_| {
            // Fallback if compgen is not available
            Command::new("bash")
                .args([
                    "-c",
                    "find /usr/bin /usr/local/bin -name 'llvm-spirv-[0-9]*' 2>/dev/null | sort -V",
                ])
                .output()
                .expect("Failed to find llvm-spirv versions")
        });

    let spirv_versions: Vec<String> = str::from_utf8(&spirv_output.stdout)
        .expect("Invalid UTF-8 in llvm-spirv output")
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Extract version numbers
    let mut matching_versions = Vec::new();

    for clang in &clang_versions {
        let clang_ver = clang.strip_prefix("clang-").unwrap_or("");
        for spirv in &spirv_versions {
            let spirv_ver = spirv.strip_prefix("llvm-spirv-").unwrap_or("");
            if clang_ver == spirv_ver && !clang_ver.is_empty() {
                matching_versions.push((clang_ver, clang.clone(), spirv.clone()));
            }
        }
    }

    // Sort by version number (highest first)
    matching_versions.sort_by(|a, b| {
        let a_ver = a.0.parse::<i32>().unwrap_or(0);
        let b_ver = b.0.parse::<i32>().unwrap_or(0);
        b_ver.cmp(&a_ver)
    });

    // If no matching versions found, try to use default commands
    if matching_versions.is_empty() {
        println!("cargo:warning=No matching clang/llvm-spirv versions found. Falling back to default commands.");
        return ("clang".to_string(), "llvm-spirv".to_string());
    }

    let (_, clang, spirv) = &matching_versions[0];
    println!(
        "cargo:warning=Using {} and {} for OpenCL compilation",
        clang, spirv
    );

    (clang.clone(), spirv.clone())
}

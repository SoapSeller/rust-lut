# Rust-LUT

[![Rust CI](https://github.com/SoapSeller/rust-lut/actions/workflows/ci.yml/badge.svg)](https://github.com/SoapSeller/rust-lut/actions/workflows/ci.yml)
[![Benchmark](https://github.com/SoapSeller/rust-lut/actions/workflows/benchmark.yml/badge.svg)](https://github.com/SoapSeller/rust-lut/actions/workflows/benchmark.yml)

A Rust implementation for applying 3D Lookup Tables (LUTs) to images. This tool allows you to transform images using industry-standard .cube LUT files commonly used in photography and video production.

## Features

- Parse and apply 3D LUT files (.cube format)
- Support for trilinear interpolation for smooth color transformations
- Fast image processing with Rust's performance

## Installation

### Prerequisites

- Rust and Cargo (latest stable version)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/SoapSeller/rust-lut.git
cd rust-lut

# Build the project
cargo build --release

# Run the binary
cargo run --release
```

## Usage

The tool currently processes a sample image using a provided LUT:

```bash
cargo run --release
```

This will:
1. Read the LUT file from `data/example.cube`
2. Load the image from `data/example.jpg`
3. Apply the LUT to the image
4. Save the processed image as `data/example_processed.png`

### Custom Usage

To use your own images and LUTs, replace the files in the `data` directory:
- Place your LUT file at `data/example.cube`
- Place your image at `data/example.jpg`

## LUT File Format

The tool supports the standard .cube LUT format:

```
# Comment line
TITLE LUT_Name
LUT_3D_SIZE Size

r g b values...
```

Example:
```
#Sony LookProfile LUT, SLog3SGamut3.CineToLC_709 full in full out v1.08.04

TITLE SLog3SGamut3.CineToLC_709
LUT_3D_SIZE 33

0.006644 0.007144 0.000000
0.015137 0.000000 0.000037
...
```

## Implementation Details

### Core Components

- **LUT Parser**: Reads and parses .cube files into a 3D lookup table
- **Image Processor**: Applies the LUT to images using trilinear interpolation

## Dependencies

- [image](https://crates.io/crates/image) - For image processing
- [glam](https://crates.io/crates/glam) - For vector math operations
- [rayon](https://crates.io/crates/rayon) - For parallel processing (optional)
- [ocl](https://crates.io/crates/ocl) - For OpenCL integration

## Optional Features

The library supports the following optional features that can be enabled or disabled:

### Rayon

```bash
# Enable rayon (enabled by default)
cargo build --release --features rayon

# Disable rayon
cargo build --release --no-default-features
```

The `rayon` feature enables parallel processing of image rows using Rayon's parallel iterator. This can significantly improve performance on multi-core CPUs by processing multiple rows of the image simultaneously.

### SPIR-V

```bash
# Enable SPIR-V (enabled by default)
cargo build --release --features spirv

# Disable SPIR-V
cargo build --release --no-default-features
```

The `spirv` feature enables the use of pre-compiled SPIR-V binary shaders for OpenCL processing instead of compiling OpenCL source code at runtime. This can provide better performance as the shader is pre-compiled during the build process.

When this feature is enabled, the build script:
1. Compiles the OpenCL kernel to LLVM bitcode using clang
2. Converts the LLVM bitcode to SPIR-V using llvm-spirv

**Requirements for SPIR-V compilation:**
- clang (with OpenCL support)
- llvm-spirv

Both features are enabled by default. To disable both:

```bash
cargo build --release --no-default-features
```

## License

MIT License. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## CI/CD

This project uses GitHub Actions for continuous integration and delivery:

- **CI Workflow**: Automatically builds, lints, and tests the code on every push and pull request to the main branch.
- **Benchmark Workflow**: Runs performance benchmarks and tracks changes over time. Runs on every push to main and weekly.
- **Release Workflow**: Automatically builds and publishes releases when a new tag is pushed.

### Running Benchmarks Locally

You can run the benchmarks locally using:

```bash
cargo bench
```

This will run all benchmarks and output the results to the terminal.

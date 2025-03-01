# Rust-LUT

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

## License

MIT License. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

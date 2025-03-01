use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use image::RgbImage;
// Import the crate modules directly
use rust_lut::{lut, processing};

fn bench_apply(c: &mut Criterion) {
    // Load the LUT file
    let lut = lut::cube3d("data/example.cube").expect("Failed to load LUT file");
    
    // Load the image
    let img_path = "data/example.jpg";
    let img = image::open(img_path)
        .expect("Failed to open image")
        .to_rgb8();
    
    // Create a target image with the same dimensions
    let width = img.width();
    let height = img.height();
    
    // Create a benchmark group for the apply function
    let mut group = c.benchmark_group("apply");
    
    // Benchmark the full image
    group.bench_function(BenchmarkId::new("full_image", format!("{}x{}", width, height)), |b| {
        b.iter(|| {
            let mut target = RgbImage::new(width, height);
            processing::apply(black_box(&lut), black_box(&img), black_box(&mut target));
        })
    });
    
    group.finish();
}

// Benchmark the process_pixel function separately
fn bench_process_pixel(c: &mut Criterion) {
    // Load the LUT file
    let lut = lut::cube3d("data/example.cube").expect("Failed to load LUT file");
    
    // Create a few test pixels
    let pixels = [
        image::Rgb([0, 0, 0]),       // Black
        image::Rgb([255, 255, 255]), // White
        image::Rgb([255, 0, 0]),     // Red
        image::Rgb([0, 255, 0]),     // Green
        image::Rgb([0, 0, 255]),     // Blue
        image::Rgb([128, 128, 128]), // Gray
    ];
    
    let mut group = c.benchmark_group("process_pixel");
    
    for (i, pixel) in pixels.iter().enumerate() {
        let color_name = match i {
            0 => "black",
            1 => "white",
            2 => "red",
            3 => "green",
            4 => "blue",
            _ => "gray",
        };
        
        group.bench_function(BenchmarkId::new("color", color_name), |b| {
            b.iter(|| {
                processing::process_pixel(black_box(&lut), black_box(pixel), black_box(0), black_box(0))
            })
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_apply, bench_process_pixel);
criterion_main!(benches);

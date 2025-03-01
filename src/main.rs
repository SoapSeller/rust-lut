use std::error::Error;
use std::path::Path;
use std::time::Instant;

mod lut;
mod processing;

fn main() -> Result<(), Box<dyn Error>> {
    // Test the LUT parser
    println!("Reading LUT file...");
    let lut: lut::Cube3D = lut::cube3d("data/example.cube")?;
    println!("LUT Title: {}", lut.title);
    println!("LUT Size: {}", lut.size);
    let expected_vectors = lut.size * lut.size * lut.size;
    println!(
        "Number of vectors: {}[Expected: {}]",
        lut.vectors.len(),
        expected_vectors
    );
    println!(
        "First vector: {:?}",
        lut.vectors.first().unwrap_or(&glam::DVec3::ZERO)
    );

    // Read the JPG file
    println!("\nReading JPG file...");

    // Replace this with the path to your JPG file
    let img_path = Path::new("./data/example.jpg");

    // Check if the file exists
    if !img_path.exists() {
        println!("Image file not found: {:?}", img_path);
        return Result::Ok(());
    } else {
        // Read the existing image
        let img = image::open(img_path)?;

        let img = match img {
            image::DynamicImage::ImageRgb8(img) => img,
            _ => return Err("Unsupported image format".into()),
        };

        let mut target = image::RgbImage::new(img.width(), img.height());
        // Process the image using the LUT
        println!("Processing image...");
        let start = Instant::now();
        processing::apply(&lut, &img, &mut target);
        let duration = start.elapsed();
        println!("Image processing took: {:?}", duration);
        target.save("./data/example_processed.png")?;
    }

    Ok(())
}

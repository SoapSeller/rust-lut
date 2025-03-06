use std::error::Error;
use std::path::Path;
use std::time::Instant;

mod lut;
mod processing;
mod processing_ocl;

extern crate ocl;

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

        // Process the image using CPU implementation
        let mut cpu_target = image::RgbImage::new(img.width(), img.height());
        println!("\nProcessing image with CPU implementation...");
        let cpu_start = Instant::now();
        processing::apply(&lut, &img, &mut cpu_target);
        let cpu_duration = cpu_start.elapsed();
        println!("CPU processing took: {:?}", cpu_duration);
        cpu_target.save("./data/example_processed_cpu.png")?;

        // Process the image using OpenCL implementation
        let mut ocl_target = image::RgbImage::new(img.width(), img.height());
        println!("\nProcessing image with OpenCL implementation...");
        let ocl_start = Instant::now();
        match processing_ocl::apply(&lut, &img, &mut ocl_target) {
            Ok(_) => {
                let ocl_duration = ocl_start.elapsed();
                println!("OpenCL processing took: {:?}", ocl_duration);
                ocl_target.save("./data/example_processed_ocl.png")?;

                // Print speedup
                let speedup = cpu_duration.as_secs_f64() / ocl_duration.as_secs_f64();
                println!("\nOpenCL speedup: {:.2}x", speedup);
            }
            Err(e) => {
                println!("OpenCL processing failed: {}", e);
            }
        }


        // Process the image using OpenCL struct implementation
        let mut ocl_target = image::RgbImage::new(img.width(), img.height());
        println!("\nProcessing image with OpenCL struct implementation...");
        let oclp = processing_ocl::ProcessingOcl::new(&lut)?;
        match oclp.apply(&img, &mut ocl_target) {
            Ok(_) => {
                ocl_target.save("./data/example_processed_ocl_struct.png")?;
            }
            Err(e) => {
                println!("OpenCL processing failed: {}", e);
            }
        }
        let ocl_start: Instant = Instant::now();

        for _i in 0..100 {
            let _ = oclp.apply(&img, &mut ocl_target);
        }

        let ocl_duration = ocl_start.elapsed() / 100;
        println!("OpenCL processing took: {:?}", ocl_duration);
        // Print speedup
        let speedup = cpu_duration.as_secs_f64() / ocl_duration.as_secs_f64();
        println!("\nOpenCL speedup: {:.2}x", speedup);

    }

    Ok(())
}

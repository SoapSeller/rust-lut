use std::path::Path;
use std::error::Error;
use std::time::Instant;

use glam::DVec3;
mod lut;

fn main() -> Result<(), Box<dyn Error>> {
    // Test the LUT parser
    println!("Reading LUT file...");
    let lut: lut::Cube3D = lut::cube3d("data/example.cube")?;
    println!("LUT Title: {}", lut.title);
    println!("LUT Size: {}", lut.size);
    let expected_vectors = lut.size * lut.size * lut.size;
    println!("Number of vectors: {}[Expected: {}]", lut.vectors.len(), expected_vectors);
    println!("First vector: {:?}", lut.vectors.first().unwrap_or(&glam::DVec3::ZERO));

    // Read the JPG file
    println!("\nReading JPG file...");

    // Replace this with the path to your JPG file
    let img_path = Path::new("./data/example.jpg");

    // Check if the file exists
    if !img_path.exists() {
        println!("Image file not found: {:?}", img_path);
        return Result::Ok(())
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
        process_image(&lut, false, &img, &mut target);
        let duration = start.elapsed();
        println!("Image processing took: {:?}", duration);
        target.save("./data/example_processed.png")?;
    }

    Ok(())
}


fn linear_to_srgb(x: f64) -> f64 {
    if x >= 0.0031308 {
        ((1.055 * x).powf(1.0/2.4)) - 0.055
    } else {
        12.92 * x
    }
}

fn srgb_to_linear(x: f64) -> f64 {
    if x >= 0.04045 {
        ((x + 0.055)/(1. + 0.055)).powf(2.4)
    }
    else  {
         x / 12.92
    }
}


fn process_pixel(srgb: bool, lut: &lut::Cube3D, pixel: &image::Rgb<u8>, _x: u32, _y: u32) -> image::Rgb<u8> {

    let rgb = if srgb {
        DVec3::new(srgb_to_linear(pixel[0] as f64 / 255.0), srgb_to_linear(pixel[1] as f64 / 255.0), srgb_to_linear(pixel[2] as f64 / 255.0))
        
    } else {
        DVec3::new(pixel[0] as f64 / 255.0, pixel[1] as f64 / 255.0, pixel[2] as f64 / 255.0)
    };

    // Do Trilinear Interpolation(https://paulbourke.net/miscellaneous/interpolation/)

    let lut_mul = (lut.size-1) as f64;

    let lut_coord_r = rgb[0] * lut_mul;
    let lut_coord_g = rgb[1] * lut_mul;
    let lut_coord_b = rgb[2] * lut_mul;

    let lut_coord_r_floor = lut_coord_r.floor() as usize;
    let lut_coord_g_floor = lut_coord_g.floor() as usize;
    let lut_coord_b_floor = lut_coord_b.floor() as usize;
    let lut_coord_r_ceil = lut_coord_r.ceil() as usize;
    let lut_coord_g_ceil = lut_coord_g.ceil() as usize;
    let lut_coord_b_ceil = lut_coord_b.ceil() as usize;

    let r = lut_coord_r - lut_coord_r_floor as f64;
    let g = lut_coord_g - lut_coord_g_floor as f64;
    let b = lut_coord_b - lut_coord_b_floor as f64;

    // Find cube(8 vectors) from the LUT around our sampled pixel values in 3D space. 
    let v000 = lut.accessor(lut_coord_r_floor, lut_coord_g_floor, lut_coord_b_floor);
    let v100 = lut.accessor(lut_coord_r_ceil, lut_coord_g_floor, lut_coord_b_floor);
    let v010 = lut.accessor(lut_coord_r_floor, lut_coord_g_ceil, lut_coord_b_floor);
    let v001 = lut.accessor(lut_coord_r_floor, lut_coord_g_floor, lut_coord_b_ceil);
    let v101 = lut.accessor(lut_coord_r_ceil, lut_coord_g_floor, lut_coord_b_ceil);
    let v011 = lut.accessor(lut_coord_r_floor, lut_coord_g_ceil, lut_coord_b_ceil);
    let v110 = lut.accessor(lut_coord_r_ceil, lut_coord_g_ceil, lut_coord_b_floor);
    let v111 = lut.accessor(lut_coord_r_ceil, lut_coord_g_ceil, lut_coord_b_ceil);

    // Interpolate
    let rgb =
         v000 * ((1.0 - r) * (1.0 - g) * (1.0 - b)) +
         v100 * (r * (1.0 - g) * (1.0 - b)) +
         v010 * ((1.0 - r) * g * (1.0 - b)) +
         v001 * ((1.0 - r) * (1.0 - g) * b) +
         v101 * (r * (1.0 - g) * b) +
         v011 * ((1.0 - r) * g * b) +
         v110 * (r * g * (1.0 - b)) +
         v111 * (r * g * b);

    if rgb[0] < 0.0 || rgb[0] > 1.0 ||
       rgb[1] < 0.0 || rgb[1] > 1.0 ||
       rgb[2] < 0.0 || rgb[2] > 1.0 {
        println!("Bad pixel: {:?}", rgb);
    }

    let rgb = rgb.clamp(DVec3::new(0.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 1.0));

    let mut out = [0_u8; 3];
    if srgb {
        for i in 0..3 {
            out[i] = (linear_to_srgb(rgb[i]) * 255.0) as u8; 
        }
    } else {
        for i in 0..3 {
            out[i] = (rgb[i] * 255.0) as u8; 
        }
    }

    return image::Rgb(out);

}


fn process_image(lut: &lut::Cube3D, srgb: bool, src: &image::RgbImage, dst: &mut image::RgbImage) {

    for x in 0..src.width() {
        for y in 0..src.height() {
            let pixel = src.get_pixel(x, y);

            let out = process_pixel(srgb, &lut, &pixel, x, y);
            dst.put_pixel(x, y, out);
        }
    }

}

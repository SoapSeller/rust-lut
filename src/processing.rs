use glam::DVec3;
use image::RgbImage;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::lut;

pub fn process_pixel(lut: &lut::Cube3D, pixel: &image::Rgb<u8>, _x: u32, _y: u32) -> image::Rgb<u8> {
    let rgb = DVec3::new(pixel[0] as f64 / 255.0, pixel[1] as f64 / 255.0, pixel[2] as f64 / 255.0);

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
    for i in 0..3 {
        out[i] = (rgb[i] * 255.0) as u8;
    }

    return image::Rgb(out);
}

pub fn apply(lut: &lut::Cube3D, src: &RgbImage, dst: &mut RgbImage) {
    assert_eq!(src.width(), dst.width());
    assert_eq!(src.height(), dst.height());

    #[cfg(feature = "rayon")]
    {
        println!("Processing with rayon parallel implementation");
        dst.rows_mut().enumerate().par_bridge().for_each(|(y, mut row)| {
            let y = y as u32;
            for x in 0..src.width() {
                let pixel = src.get_pixel(x, y);
                let out = process_pixel(&lut, &pixel, x, y);
                let dst = row.next().unwrap();
                *dst = out;
            }
        });
    }

    #[cfg(not(feature = "rayon"))]
    {
        println!("Processing with single-threaded implementation");
        for y in 0..src.height() {
            for x in 0..src.width() {
                let pixel = src.get_pixel(x, y);
                let out = process_pixel(&lut, &pixel, x, y);
                dst.put_pixel(x, y, out);
            }
        }
    }
}

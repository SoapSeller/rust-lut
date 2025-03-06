use image::RgbImage;
use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
use std::error::Error;

use crate::lut;
use crate::processing;

// OpenCL kernel for trilinear interpolation
const KERNEL_SRC: &str = include_str!("processing.cl");

// Function to apply the LUT using OpenCL
fn apply_ocl(lut: &lut::Cube3D, src: &RgbImage, dst: &mut RgbImage) -> Result<(), Box<dyn Error>> {
    let width = src.width() as usize;
    let height = src.height() as usize;

    // Set up OpenCL - use try/catch pattern to handle errors
    let platform = Platform::default();
    let device = Device::first(platform)?;
    let context = Context::builder()
        .platform(platform)
        .devices(device)
        .build()?;
    let queue = Queue::new(&context, device, None)?;

    // Prepare LUT data for OpenCL
    let lut_size = lut.size;
    let mut lut_data = Vec::with_capacity(lut.vectors.len() * 3);
    for vec in &lut.vectors {
        lut_data.push(vec.x as f32);
        lut_data.push(vec.y as f32);
        lut_data.push(vec.z as f32);
    }

    // Create buffers
    let input_buffer = Buffer::<u8>::builder()
        .queue(queue.clone())
        .flags(ocl::flags::MEM_READ_ONLY)
        .len(width * height * 3)
        .build()?;

    let output_buffer = Buffer::<u8>::builder()
        .queue(queue.clone())
        .flags(ocl::flags::MEM_WRITE_ONLY)
        .len(width * height * 3)
        .build()?;

    let lut_buffer = Buffer::<f32>::builder()
        .queue(queue.clone())
        .flags(ocl::flags::MEM_READ_ONLY)
        .len(lut_data.len())
        .build()?;

    // Write data to buffers
    // let src_bytes: Vec<u8> = src.pixels().flat_map(|p| {
    //     let mut vec = p.0.to_vec();
    //     //vec.push(0); // Dummy
    //     vec
    // }).collect();
    let src_bytes: Vec<u8> = src.pixels().flat_map(|p| p.0.to_vec() ).collect();
    input_buffer.write(&src_bytes).enq()?;
    lut_buffer.write(&lut_data).enq()?;

    // Build program and kernel
    let program = Program::builder()
        .devices(device)
        .src(KERNEL_SRC)
        .build(&context)?;

    let kernel = Kernel::builder()
        .program(&program)
        .name("process_image")
        .queue(queue)
        .arg(&input_buffer)
        .arg(&output_buffer)
        .arg(&lut_buffer)
        .arg(lut_size as i32)
        .arg(width as i32)
        .arg(height as i32)
        .build()?;

    // Execute kernel
    let gws = [width as u64, height as u64, 1];
    unsafe { kernel.cmd().global_work_size(gws).enq()?; }

    // Read results
    let mut result = vec![0u8; width * height * 3];
    output_buffer.read(&mut result).enq()?;

    // Copy results to output image
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            dst.put_pixel(
                x as u32,
                y as u32,
                image::Rgb([result[idx], result[idx + 1], result[idx + 2]]),
            );
        }
    }

    Ok(())
}

// Public function to apply the LUT, falling back to CPU if OpenCL fails
pub fn apply(lut: &lut::Cube3D, src: &RgbImage, dst: &mut RgbImage) -> Result<(), Box<dyn Error>> {
    assert_eq!(src.width(), dst.width());
    assert_eq!(src.height(), dst.height());

    // Try to use OpenCL
    match apply_ocl(lut, src, dst) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Fall back to CPU implementation
            processing::apply(lut, src, dst);
            Err(e)
        }
    }
}

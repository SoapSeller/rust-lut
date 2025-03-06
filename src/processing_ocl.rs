use image::RgbImage;
use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
use std::error::Error;

use crate::lut;
// OpenCL kernel for trilinear interpolation
const KERNEL_SRC: &str = include_str!("processing.cl");

pub struct ProcessingOcl {
    program: Program,
    queue: Queue,
    lut: Buffer<f32>,
    lut_size: usize,
}

impl ProcessingOcl {
    pub fn new(lut: &lut::Cube3D) -> Result<Self, Box<dyn Error>> {
        let platform = Platform::default();
        let device = Device::first(platform)?;
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()?;
        let queue = Queue::new(&context, device, None)?;
        // Create OpenCL program and kernel
        let program = Program::builder()
            .devices(device)
            .src(KERNEL_SRC)
            .build(&context)?;

        // Create buffer for LUT data
        let mut lut_data = Vec::with_capacity(lut.vectors.len() * 3);
        for vec in &lut.vectors {
            lut_data.push(vec.x as f32);
            lut_data.push(vec.y as f32);
            lut_data.push(vec.z as f32);
        }

        let lut_buffer = Buffer::<f32>::builder()
            .queue(queue.clone())
            .flags(ocl::flags::MEM_READ_ONLY)
            .len(lut_data.len())
            .build()?;

        lut_buffer.write(&lut_data).enq()?;

        Ok(Self {
            program,
            queue,
            lut: lut_buffer,
            lut_size: lut.size
        })
    }

    pub fn apply(&self, src: &RgbImage, dst: &mut RgbImage) -> Result<(), Box<dyn Error>> {
        let width = src.width() as usize;
        let height = src.height() as usize;

        // Create buffers
        let input_buffer = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .flags(ocl::flags::MEM_READ_ONLY)
            .len(width * height * 3)
            .build()?;

        let output_buffer = Buffer::<u8>::builder()
            .queue(self.queue.clone())
            .flags(ocl::flags::MEM_WRITE_ONLY)
            .len(width * height * 3)
            .build()?;

        // let src_bytes: Vec<u8> = src.pixels().flat_map(|p| p.0.to_vec()).collect();
        // input_buffer.write(&src_bytes).enq()?;
        unsafe { input_buffer.write(src as &[u8]).block(false).enq()?; }

        let kernel = Kernel::builder()
            .program(&self.program)
            .name("process_image")
            .queue(self.queue.clone())
            .arg(&input_buffer)
            .arg(&output_buffer)
            .arg(&self.lut)
            .arg(self.lut_size as i32)
            .arg(width as i32)
            .arg(height as i32)
            .build()?;

        // Execute kernel
        let gws = [width as u64, height as u64, 1];
        unsafe {
            kernel.cmd().global_work_size(gws).enq()?;
        }


        output_buffer.read(dst as &mut [u8]).enq()?;
        // // Read results
        // let mut result = vec![0u8; width * height * 3];
        // output_buffer.read(&mut result).enq()?;

        // // Copy results to output image
        // for y in 0..height {
        //     for x in 0..width {
        //         let idx = (y * width + x) * 3;
        //         dst.put_pixel(
        //             x as u32,
        //             y as u32,
        //             image::Rgb([result[idx], result[idx + 1], result[idx + 2]]),
        //         );
        //     }
        // }

        Ok(())
    }
}

// Function to apply the LUT using OpenCL
pub fn apply(lut: &lut::Cube3D, src: &RgbImage, dst: &mut RgbImage) -> Result<(), Box<dyn Error>> {
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

    // let src_bytes: Vec<u8> = src.pixels().flat_map(|p| p.0.to_vec()).collect();
    // input_buffer.write(&src_bytes).enq()?;
    input_buffer.write(src as &[u8]).enq()?;
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
    unsafe {
        kernel.cmd().global_work_size(gws).enq()?;
    }

    // Read results
    output_buffer.read(dst as &mut [u8]).enq()?;
    //let mut result = vec![0u8; width * height * 3];
    //output_buffer.read(&mut result).enq()?;

    // // Copy results to output image
    // for y in 0..height {
    //     for x in 0..width {
    //         let idx = (y * width + x) * 3;
    //         dst.put_pixel(
    //             x as u32,
    //             y as u32,
    //             image::Rgb([result[idx], result[idx + 1], result[idx + 2]]),
    //         );
    //     }
    // }

    Ok(())
}


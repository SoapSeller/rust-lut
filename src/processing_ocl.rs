use image::RgbImage;
use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
use std::error::Error;

use crate::lut;
// OpenCL kernel for trilinear interpolation

#[cfg(feature = "spirv")]
type Iot = i8;

#[cfg(not(feature = "spirv"))]
type Iot = u8;

#[cfg(feature = "spirv")]
const KERNEL_SPIRV: &[u8] = include_bytes!("processing.spv");

#[cfg(not(feature = "spirv"))]
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
        #[cfg(feature = "spirv")]
        let program = Program::builder()
            .devices(device)
            .il(KERNEL_SPIRV)
            .build(&context)?;

        #[cfg(not(feature = "spirv"))]
        let program = Program::builder()
            .devices(device)
            .source(KERNEL_SRC)
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
            lut_size: lut.size,
        })
    }

    pub fn apply(&self, src: &RgbImage, dst: &mut RgbImage) -> Result<(), Box<dyn Error>> {
        let width = src.width() as usize;
        let height = src.height() as usize;

        // Create buffers
        let input_buffer = Buffer::<Iot>::builder()
            .queue(self.queue.clone())
            .flags(ocl::flags::MEM_READ_ONLY)
            .len(width * height * 3)
            .build()?;

        let output_buffer = Buffer::<Iot>::builder()
            .queue(self.queue.clone())
            .flags(ocl::flags::MEM_WRITE_ONLY)
            .len(width * height * 3)
            .build()?;

        // let src_bytes: Vec<u8> = src.pixels().flat_map(|p| p.0.to_vec()).collect();
        // input_buffer.write(&src_bytes).enq()?;
        unsafe {
            #[cfg(feature = "spirv")]
            {
                let src_ref = src as &[u8];
                let i8_ref = &*(src_ref as *const _ as *const [i8]);
                input_buffer.write(i8_ref).block(false).enq()?;
            }

            #[cfg(not(feature = "spirv"))]
            input_buffer.write(src as &[u8]).block(false).enq()?;
        }

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

        #[cfg(feature = "spirv")]
        unsafe {
            let dst_ref = dst as &mut [u8];
            let i8_ref = &mut *(dst_ref as *mut _ as *mut [i8]);
            output_buffer.read(i8_ref as &mut [i8]).enq()?;
        }

        #[cfg(not(feature = "spirv"))]
        output_buffer.read(dst as &mut [u8]).enq()?;

        Ok(())
    }
}

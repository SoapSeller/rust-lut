__kernel void process_image(
    __global const uchar* input,
    __global uchar* output,
    __global const float* lut,
    const int lut_size,
    const int width,
    const int height)
{
    const int x = get_global_id(0);
    const int y = get_global_id(1);
    
    if (x >= width || y >= height) {
        return;
    }
    
    size_t idx = y * width + x;
    const uchar3 pixel = vload3(idx, input);
    
    // Normalize RGB values to [0, 1]
    const float3 rgb = (float3)(
        pixel.x / 255.0f,
        pixel.y / 255.0f,
        pixel.z / 255.0f
    );
    
    // Calculate LUT coordinates
    const float lut_mul = (float)(lut_size - 1);
    const float lut_coord_r = rgb.x * lut_mul;
    const float lut_coord_g = rgb.y * lut_mul;
    const float lut_coord_b = rgb.z * lut_mul;
    
    // Get floor and ceiling coordinates
    const int lut_coord_r_floor = (int)floor(lut_coord_r);
    const int lut_coord_g_floor = (int)floor(lut_coord_g);
    const int lut_coord_b_floor = (int)floor(lut_coord_b);
    const int lut_coord_r_ceil = (int)ceil(lut_coord_r);
    const int lut_coord_g_ceil = (int)ceil(lut_coord_g);
    const int lut_coord_b_ceil = (int)ceil(lut_coord_b);
    
    // Calculate interpolation factors
    const float r = lut_coord_r - lut_coord_r_floor;
    const float g = lut_coord_g - lut_coord_g_floor;
    const float b = lut_coord_b - lut_coord_b_floor;
    
    // Get indices for the 8 corners of the cube
    size_t idx000 = lut_coord_r_floor + lut_size * lut_coord_g_floor + lut_size * lut_size * lut_coord_b_floor;
    size_t idx100 = lut_coord_r_ceil + lut_size * lut_coord_g_floor + lut_size * lut_size * lut_coord_b_floor;
    size_t idx010 = lut_coord_r_floor + lut_size * lut_coord_g_ceil + lut_size * lut_size * lut_coord_b_floor;
    size_t idx001 = lut_coord_r_floor + lut_size * lut_coord_g_floor + lut_size * lut_size * lut_coord_b_ceil;
    size_t idx101 = lut_coord_r_ceil + lut_size * lut_coord_g_floor + lut_size * lut_size * lut_coord_b_ceil;
    size_t idx011 = lut_coord_r_floor + lut_size * lut_coord_g_ceil + lut_size * lut_size * lut_coord_b_ceil;
    size_t idx110 = lut_coord_r_ceil + lut_size * lut_coord_g_ceil + lut_size * lut_size * lut_coord_b_floor;
    size_t idx111 = lut_coord_r_ceil + lut_size * lut_coord_g_ceil + lut_size * lut_size * lut_coord_b_ceil;
    
    // Get values from the LUT
    const float3 v000 = vload3(idx000, lut);
    const float3 v100 = vload3(idx100, lut);
    const float3 v010 = vload3(idx010, lut);
    const float3 v001 = vload3(idx001, lut);
    const float3 v101 = vload3(idx101, lut);
    const float3 v011 = vload3(idx011, lut);
    const float3 v110 = vload3(idx110, lut);
    const float3 v111 = vload3(idx111, lut);

    
    // Perform trilinear interpolation
    float3 result = v000 * ((1.0f - r) * (1.0f - g) * (1.0f - b))
                  + v100 * (r * (1.0f - g) * (1.0f - b))
                  + v010 * ((1.0f - r) * g * (1.0f - b))
                  + v001 * ((1.0f - r) * (1.0f - g) * b)
                  + v101 * (r * (1.0f - g) * b)
                  + v011 * ((1.0f - r) * g * b)
                  + v110 * (r * g * (1.0f - b))
                  + v111 * (r * g * b);
    
    // Clamp result to [0, 1]
    result = clamp(result, (float3)(0.0f, 0.0f, 0.0f), (float3)(1.0f, 1.0f, 1.0f));
    
    uchar3 final = (uchar3)(
        (uchar)(result.x * 255.0f),
        (uchar)(result.y * 255.0f),
        (uchar)(result.z * 255.0f)
    );
    vstore3(final, idx, output);

}

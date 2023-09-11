use image::{ImageBuffer, Rgba, Pixel, imageops::FilterType};
use noise::{NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::error::Error;
use super::constants::COLORS;


pub fn generate_perlin_noise_buffer(width: u32, height: u32, offset_x: f64, offset_y: f64, scale: f64, opacity: f64, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let perlin = Perlin::new(seed);

    ImageBuffer::from_fn(width, height, |x, y| {
        let x = (x as f64 + offset_x) * scale;
        let y = (y as f64 + offset_y) * scale;
        let noise_val = perlin.get([x, y]) * 0.5 + 0.5;
        let color = (noise_val * 255.0) as u8;
        Rgba([color, color, color,(opacity * 255.0) as u8])
    })
}

pub fn blend_buffers(buffer_a: &ImageBuffer<Rgba<u8>, Vec<u8>>, buffer_b: &ImageBuffer<Rgba<u8>, Vec<u8>>, blend_mode: i32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = buffer_a.dimensions();
    
    ImageBuffer::from_fn(width, height, |x, y| {
        let pixel_a = buffer_a.get_pixel(x, y);
        let pixel_b = buffer_b.get_pixel(x, y);
        
        let alpha_a = pixel_a[3] as f32 / 255.0;
        let alpha_b = pixel_b[3] as f32 / 255.0;
        
        let new_alpha = alpha_a * (1.0 - alpha_b) + alpha_b;
        
        let mut blended_pixel = [0u8; 4];
        
        for i in 0..3 {
            let channel_a = pixel_a[i] as f32 / 255.0;
            let channel_b = pixel_b[i] as f32 / 255.0;
            
            match blend_mode {
                // Blend
                0 => {
                    blended_pixel[i] = ((channel_a * (1.0 - alpha_b) + channel_b * alpha_b) * 255.0).min(255.0) as u8;
                },
                // Multiply                
                1 => {
                    let blended_channel = channel_a * (1.0 - alpha_b) + channel_a * channel_b * alpha_b; // Interpolate based on alpha_b
                    blended_pixel[i] = (blended_channel * 255.0).min(255.0) as u8;
                },
                //
                2 => {
                    blended_pixel[i] = ((channel_a * alpha_a + channel_b * alpha_b) * 255.0).min(255.0) as u8;
                },
                _ => {
                    panic!("Invalid blend mode");
                }
            }
        }
        
        blended_pixel[3] = (new_alpha * 255.0).min(255.0) as u8;
        
        Rgba(blended_pixel)
    })
}

pub fn save_image_to_desktop(buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>, filename: &str, suffix: &str){
    let desktop_path = dirs::desktop_dir();
    match desktop_path {
        Some(path) => {            
            let full_path = path.join(format!("{}_{}.png", filename, suffix));
            println!("Desktop path: {}", full_path.display());
            match buffer.save(full_path) {
                Ok(_) => {
                    println!("Image saved");
                },
                Err(e) => {
                    println!("Couldn't save image: {}", e);
                }
            }
                
        },
        None => {
            println!("Couldn't find desktop path");
        }
    }
    
}

// Function to decide if erosion should happen based on the pixels and the erosion mode
fn should_erode(center: Rgba<u8>, neighbor: Rgba<u8>, talus_angle: f32, erosion_mode: i32) -> bool {
    for channel in 0..3 {
        match erosion_mode {
            0 => { return false; },
            1 => {
                if neighbor[channel] < center[channel] {
                    return true;
                }
            },
            2 => {
                let delta = (center[channel] as f32 - neighbor[channel] as f32) / 255.0;
                if delta > talus_angle {
                    return true;
                }
            },
            _ => { panic!("Invalid erosion mode"); }
        }
    }
    false
}

pub fn thermal_erosion(
    heightmap: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    colormap: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    iterations: usize,
    talus_angle: f32,
    erosion_mode: i32,
) {
    let (width, height) = heightmap.dimensions();

    for _ in 0..iterations {
        let temp_heightmap = heightmap.clone(); // Temporary heightmap to store updates
        let temp_colormap = colormap.clone(); // Temporary colormap to store updates

        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let center_pixel = temp_heightmap.get_pixel(x, y);
                let mut changed = false;

                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let neighbor_pixel = temp_heightmap.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32);
                        let neighbor_color = temp_colormap.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32);

                        if should_erode(*center_pixel, *neighbor_pixel, talus_angle, erosion_mode) {
                            // Update both heightmap and colormap
                            heightmap.put_pixel(x, y, *neighbor_pixel);
                            colormap.put_pixel(x, y, *neighbor_color);
                            changed = true;
                            break;
                        }
                    }
                    if changed {
                        break;
                    }
                }
            }
        }
    }
}



pub fn clamp_image_buffer(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, min: u8, max: u8) {
    let (width, height) = img.dimensions();

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel_mut(x, y);
            for channel in 0..3 {
                pixel[channel] = pixel[channel].min(max).max(min);
            }
        }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t) * a as f32 + t * b as f32) as u8
}

pub fn colorize_buffer(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut colorized_img = img.clone();
    
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let luminance = pixel.to_luma()[0] as f32 / 255.0;
            
            // Calculate indices and interpolation factor
            let t = luminance * (COLORS.len() as f32 - 1.0);
            let index1 = t.floor() as usize;
            let index2 = (index1 + 1).min(COLORS.len() - 1);
            let factor = t - index1 as f32;
            
            let color1 = COLORS[index1];
            let color2 = COLORS[index2];
            
            // Interpolate between the two colors
            let mut new_color = [0u8; 4];
            for i in 0..4 {
                new_color[i] = lerp(color1[i], color2[i], factor);
            }
            
            colorized_img.put_pixel(x, y, Rgba(new_color));
        }
    }
    
    colorized_img
}

pub fn simulate_river_flow(
    heightmap: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    colormap: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    rain_iterations: usize,
    erosion_factor: i16,
    num_rivers: usize,
    fixed_seed: u64
) -> Result<(), String> {
    let (width, height) = heightmap.dimensions();
    
    // Use a seeded RNG for consistent river origins
    let mut rng = StdRng::seed_from_u64(fixed_seed);

    let mut heightmap_i16: Vec<Vec<i16>> = vec![vec![0; height as usize]; width as usize];
    for x in 0..width {
        for y in 0..height {
            heightmap_i16[x as usize][y as usize] = heightmap.get_pixel(x, y)[1] as i16;
        }
    }

    for _ in 0..num_rivers {
        let mut x = rng.gen_range(1..width - 1);
        let mut y = rng.gen_range(1..height - 1);

        for _ in 0..rain_iterations {
            if x > 0 && x < width - 1 && y > 0 && y < height - 1 {
                let center_height = heightmap_i16[x as usize][y as usize];
                let mut min_height = center_height;
                let mut min_x = x;
                let mut min_y = y;

                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let neighbor_x = (x as i32 + dx) as usize;
                        let neighbor_y = (y as i32 + dy) as usize;

                        if neighbor_x >= width as usize || neighbor_y >= height as usize {
                            continue;
                        }

                        let neighbor_height = heightmap_i16[neighbor_x][neighbor_y];
                        if neighbor_height < min_height {
                            min_height = neighbor_height;
                            min_x = neighbor_x as u32;
                            min_y = neighbor_y as u32;
                        }
                    }
                }

                if min_height < center_height {
                    let new_height = min_height.saturating_sub(erosion_factor);
                    let new_height_u8 = std::cmp::max(0, std::cmp::min(new_height, 255)) as u8;
                    heightmap.put_pixel(min_x, min_y, Rgba([new_height_u8, new_height_u8, new_height_u8, 255]));
                    colormap.put_pixel(min_x, min_y, COLORS[0]);
                    x = min_x;
                    y = min_y;
                }
            }
        }
    }

    Ok(())
}


pub fn scale_image(buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, target_size: (u32, u32), scale_method: FilterType) -> Result<(), Box<dyn Error>> {
    let (target_width, target_height) = target_size;

    if target_width == 0 || target_height == 0 {
        return Err("Target size should be greater than zero".into());
    }

    let scaled_image = image::imageops::resize(buffer, target_width, target_height, scale_method);
    *buffer = scaled_image;

    Ok(())
}
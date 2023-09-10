extern crate image;
extern crate dirs;
use image::imageops::FilterType::Lanczos3;
use image::{ImageBuffer, Rgba, Pixel, imageops::FilterType};
use noise::{NoiseFn, Perlin};
use std::sync::{Arc, Mutex};
use std::thread;
use std::error::Error;
use slint::{slint, Model, VecModel,SharedPixelBuffer,Rgba8Pixel};
use rand::Rng;

const IMAGE_SIZE: u32 = 256;
const EXPORT_IMAGE_SIZE: (u32,u32) = (4096,4096);
const COLORS: [Rgba<u8>; 10] = [
    Rgba([0, 0, 200, 255]),   // Blue for water
    Rgba([0, 200, 255, 255]),   // Blue for water
    Rgba([0, 128, 0, 255]),   // Green for grasslands
    Rgba([60, 160, 0, 255]),   // Green for grasslands
    Rgba([100, 140, 0, 255]),   // Green for grasslands
    Rgba([139, 69, 19, 255]), // Brown for hills
    Rgba([139, 100, 60, 255]), // Brown for hills
    Rgba([105, 105, 105, 255]), // Dark Gray for lower mountains
    Rgba([192, 192, 192, 255]), // Light Gray for higher mountains
    Rgba([255, 255, 255, 255])  // White for mountain tips
];

slint! {
    import { Button , VerticalBox, Slider, HorizontalBox, CheckBox, TextEdit, ComboBox} from "std-widgets.slint";

    export struct LayerParams {
        scale: float,
        offset_x: float,
        offset_y: float,
        seed: float,
        opacity: float,
        blend_mode: int,
    }
    
    export component App inherits Window {
        title: "Perlin Noise Generator";
        icon: @image-url("images/icon.png");
        min-width: 800px;
        
        callback ui_changed;
        callback export_btn_clicked <=> btn.clicked;
        callback add_layer_btn_clicked <=> add_layer_btn.clicked;
        callback remove_layer_btn_clicked <=> remove_layer_btn.clicked;
        
        out property <float> scale <=> scl.value;
        out property <float> offset_x <=> ofx.value;
        out property <float> offset_y <=> ofy.value;
        out property <float> seed <=> sd.value;
        in-out property <image> image <=> img.source;
        in-out property <image> colormap <=> colormap.source;
        out property <[LayerParams]> layers: [];
        out property <string> filename <=> filename.text;

        out property <int> erosion_mode <=> erosion_mode.current-index;
        out property <float> erosion_iterations <=> erosion_iterations.value;
        out property <float> talus_angle <=> talus_angle.value;

        out property <bool> flatten_enabled <=> flatten_enabled.checked;
        out property <float> ground_level <=> ground_level.value;

        out property <bool> calculate_rivers <=> river_enabled.checked;
        out property <float> river_iterations <=> river_iterations.value;
        out property <float> erosion_factor <=> erosion_factor.value;
        out property <float> river_amount <=> river_amount.value;

        Rectangle {
            background: #292929;
            border-color: #161616;
            border-width: 5px;
            border-radius: 10px;
            HorizontalBox {
                VerticalBox {
                    Rectangle {
                        background: #161616;
                        border-radius: 10px;
                        border-width: 2px;
                        max-height: 150px;
                        VerticalBox {
                            spacing: 0px;
                            max-height: 150px;       
                            HorizontalBox {
                                Text {text: "Scale";}
                                scl:=Slider {value: 0.05;minimum: 0.001;maximum: 0.1; changed => {
                                    //scl_label.text = scl.value;
                                    root.ui_changed();
                                }}
                                //scl_label:=Text{ text: scl.value;}
                            }
                            HorizontalBox {
                                Text {text: "Offset X";}
                                ofx:=Slider {value: 0;minimum: 0.0;maximum: 256; changed => {
                                    //ofx_label.text = ofx.value;
                                    root.ui_changed();
                                }}
                                //ofx_label:=Text{ text: ofx.value;}
                            }
                            HorizontalBox {
                                Text {text: "Offset Y";}
                                ofy:=Slider {value: 0;minimum: 0.0;maximum: 256; changed => {
                                    //ofy_label.text = ofy.value;
                                    root.ui_changed();
                                }}
                                //ofy_label:=Text{ text: ofy.value;}
                            }
                            HorizontalBox {
                                Text {text: "Seed";}
                                sd:=Slider {value: 1;minimum: 1;maximum: 5000; changed => {
                                    //sd_label.text = sd.value;
                                    root.ui_changed();
                                }}
                                //sd_label:=Text{ text: sd.value;}
                            }                
                        }            
                    }
                    HorizontalBox {
                        add_layer_btn:=Button {height: 25px; text: "Add Layer";}
                        remove_layer_btn:=Button {height: 25px; text: "Remove Layer";}
                    }
                    for layer[i] in root.layers: Rectangle {
                        background: #161616;
                        border-radius: 10px;
                        border-width: 2px;
                        max-height: 150px;
                        VerticalBox {
                            spacing: -15px;
                            max-height: 150px;
                            Text {text: "Layer " + i; height: 25px;}     
                            HorizontalBox {
                                Text {text: "Scale";}
                                layer_scl:=Slider {value: 0.05;minimum: 0.001;maximum: 0.1; changed => {
                                    layer.scale = self.value;
                                    //layer_scl_label.text = layer.scale;
                                    root.ui_changed();
                                }}
                                //layer_scl_label:=Text{ text: layer_scl.value;}
                            }
                            HorizontalBox {
                                Text {text: "Offset X";}
                                layer_ofx:=Slider {value: 0;minimum: 0.0;maximum: 256; changed => {
                                    layer.offset-x = self.value;
                                    //layer_ofx_label.text = layer_ofx.value;
                                    root.ui_changed();
                                }}
                                //layer_ofx_label:=Text{ text: layer_ofx.value;}
                            }
                            HorizontalBox {
                                Text {text: "Offset Y";}
                                layer_ofy:=Slider {value: 0;minimum: 0.0;maximum: 256; changed => {
                                    layer.offset-y = self.value;
                                    //layer_ofy_label.text = layer_ofy.value;
                                    root.ui_changed();
                                }}
                                //layer_ofy_label:=Text{ text: layer_ofy.value;}
                            }
                            HorizontalBox {
                                Text {text: "Seed";}
                                layer_sd:=Slider {value: 1;minimum: 1;maximum: 5000; changed => {
                                    layer.seed = self.value;
                                    //layer_sd_label.text = layer_sd.value;
                                    root.ui_changed();
                                }}
                                //layer_sd_label:=Text{ text: layer_sd.value;}
                            }
                            HorizontalBox {
                                Text {text: "Opacity";}
                                layer_ops:=Slider {value: 1;minimum: 0.0;maximum: 1.0; changed => {
                                    layer.opacity = self.value;
                                    //layer_ops_label.text = layer_ops.value;
                                    root.ui_changed();
                                }}
                                //layer_ops_label:=Text{ text: layer_ops.value;}
                            }
                            HorizontalBox {
                                Text {text: "Blend Mode";}
                                layer_mul:=ComboBox {
                                    model: ["Blend", "Multiply", "Screen"];
                                    current-index: 0;
                                    selected => {
                                        layer.blend_mode = self.current-index;
                                    }
                                }
                            }             
                        }
                    }
                }
                VerticalBox {
                    Rectangle {
                        background: #161616;
                        border-radius: 10px;
                        VerticalBox {
                            Text {text: "Thermal Erosion"; height: 25px;}
                            erosion_mode:=ComboBox {
                                model: ["None", "Standard", "With Talus"];
                                current-index: 0;
                                height: 25px;
                                selected => {
                                    root.ui_changed();
                                }
                            }
                            HorizontalBox {
                                Text {text: "Iterations"; vertical-alignment: center;}
                                erosion_iterations:=Slider {value: 5;minimum: 5;maximum: 100; height: 25px; changed => {
                                    root.ui_changed();
                                }}
                            }
                            HorizontalBox {
                                Text {text: "Talus Angle"; vertical-alignment: center;}
                                talus_angle:=Slider {value: 0.01;minimum: 0.0;maximum: 0.1; height: 25px; changed => {
                                    root.ui_changed();
                                }}
                            }
                        }
                    }
                    Rectangle {
                        background: #161616;
                        border-radius: 10px;
                        VerticalBox {                            
                            Text {text: "River Flow"; height: 25px;}
                            HorizontalBox {
                                Text {text: "Calculate Rivers"; vertical-alignment: center;}
                                river_enabled:=CheckBox {enabled: erosion-mode.current-index != 0 ;checked: false; toggled => {
                                    root.ui_changed();
                                }}
                            }
                            HorizontalBox {
                                Text {text: "Iterations"; vertical-alignment: center;}
                                river_iterations:=Slider {enabled: erosion-mode.current-index != 0 ;value: 1;minimum: 1;maximum: 256; height: 25px; changed => {
                                    root.ui_changed();
                                }}
                            }
                            HorizontalBox {
                                Text {text: "Erosion Factor"; vertical-alignment: center;}
                                erosion_factor:=Slider {enabled: erosion-mode.current-index != 0 ;value: 1;minimum: 1;maximum: 100; height: 25px; changed => {
                                    root.ui_changed();
                                }}
                            }
                            HorizontalBox {
                                Text {text: "River Amount"; vertical-alignment: center;}
                                river_amount:=Slider {enabled: erosion-mode.current-index != 0 ;value: 1;minimum: 1;maximum: 100; height: 25px; changed => {
                                    root.ui_changed();
                                }}
                            }
                        }
                    }
                    Rectangle {
                        background: #161616;
                        border-radius: 10px;
                        VerticalBox {
                            Text {text: "Flatten Ground"; height: 25px;}
                            HorizontalBox {
                                Text {text: "Flatten Enabled"; vertical-alignment: center; height: 25px;}
                                flatten_enabled:=CheckBox {checked: false; height: 25px; toggled => {
                                    root.ui_changed();
                                }}
                            }
                            HorizontalBox {
                                Text {text: "Ground Level"; vertical-alignment: center; height: 25px;}
                                ground_level:=Slider {height: 25px;value: 0.5;minimum: 0.0;maximum: 255.0; changed => {
                                    root.ui_changed();
                                }}
                            }
                        }
                    }                   
                                        
                }
                VerticalBox {
                    img:=Image {source: @image-url("images/reload_icon.png");min-width: 256px;min-height: 256px;}
                    colormap:=Image {source: @image-url("images/reload_icon.png");min-width: 256px;min-height: 256px;}
                    HorizontalBox {
                        Text {
                            text: "Filename";
                            vertical-alignment: center;
                            height: 25px;
                        }
                        Rectangle {
                            background: #161616;
                            border-radius: 5px;
                            filename:=TextInput {
                                single-line: true;
                                text: "noise";
                                width: 100px;
                                height: 30px;
                                vertical-alignment: center;
                            }
                        }
                        
                    }
                    
                    btn:=Button {height: 30px; text: "Export Texture";}
                    Text {
                        text: "Exported textures will be saved on your Desktop";
                        color: #8a8a8a;
                        font-size: 10px;
                        font-italic: true;
                    }
                }
            }
        }
          
    }      
        
}

struct Layers {
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    seed: u32,
    opacity: f64,
    blend_mode: i32,
}

fn main() {
    let app: App = App::new().expect("Failed to create App");
    let app_weak: slint::Weak<App> = app.as_weak();
    let app_add_weak = app_weak.clone();
    let app_remove_weak = app_weak.clone();
    let app_export_weak = app_weak.clone();    
    let main_buffer: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>> = Arc::new(Mutex::new(ImageBuffer::new(IMAGE_SIZE, IMAGE_SIZE)));
    let main_color_buffer: Arc<Mutex<ImageBuffer<Rgba<u8>, Vec<u8>>>> = Arc::new(Mutex::new(ImageBuffer::new(IMAGE_SIZE, IMAGE_SIZE)));
    let export_main_buffer = Arc::clone(&main_buffer);
    let export_main_color_buffer = Arc::clone(&main_color_buffer);

    app.on_ui_changed({
        let main_buffer = Arc::clone(&main_buffer);
        let main_color_buffer = Arc::clone(&main_color_buffer);
        move || {
            let clicked_handle = app_weak.upgrade().unwrap();
            let main_buffer = Arc::clone(&main_buffer);
            let main_color_buffer = Arc::clone(&main_color_buffer);
            let scale = clicked_handle.get_scale() as f64;
            let offset_x = clicked_handle.get_offset_x() as f64;
            let offset_y = clicked_handle.get_offset_y() as f64;
            let seed = clicked_handle.get_seed() as u32;
            let model_rc = clicked_handle.get_layers();
            let layer_parms = model_rc.as_any().downcast_ref::<VecModel<LayerParams>>().unwrap();
            let erosion_mode = clicked_handle.get_erosion_mode() as i32;
            let erosion_iterations = clicked_handle.get_erosion_iterations() as usize;
            let talus_angle = clicked_handle.get_talus_angle() as f32;
            let flatten_enabled = clicked_handle.get_flatten_enabled() as bool;
            let ground_level = clicked_handle.get_ground_level() as u8;
            let calculate_rivers = clicked_handle.get_calculate_rivers() as bool;
            let river_iterations = clicked_handle.get_river_iterations() as usize;
            let erosion_factor = clicked_handle.get_erosion_factor() as i16;
            let river_amount = clicked_handle.get_river_amount() as usize;

            let mut layers: Vec<Layers> = Vec::new();
            for layer in layer_parms.iter() {
                layers.push(Layers {
                    scale: layer.scale as f64,
                    offset_x: layer.offset_x as f64,
                    offset_y: layer.offset_y as f64,
                    seed: layer.seed as u32,
                    opacity: layer.opacity as f64,
                    blend_mode: layer.blend_mode as i32,
                });
            }

            let handle = clicked_handle.as_weak();
            thread::spawn(move || {                
                let locked_buffer_result = main_buffer.lock();
                let locked_color_buffer_result = main_color_buffer.lock();
                match (locked_buffer_result, locked_color_buffer_result) {
                    (Ok(mut locked_buffer), Ok(mut locked_color_buffer)) => {
                        let mut buffer = generate_perlin_noise_buffer(IMAGE_SIZE,IMAGE_SIZE,offset_x,offset_y,scale,1.0,seed);
                        for layer in layers {
                            let layer_buffer = generate_perlin_noise_buffer(IMAGE_SIZE,IMAGE_SIZE,layer.offset_x as f64,layer.offset_y as f64,layer.scale as f64,layer.opacity as f64,layer.seed as u32);
                            buffer = blend_buffers(&buffer,&layer_buffer,layer.blend_mode);
                        }
                        if flatten_enabled {
                            clamp_image_buffer(& mut buffer, ground_level, 255);
                        }
                        let mut colored_buffer = colorize_buffer(&buffer);
                        if erosion_mode != 0 {
                            thermal_erosion(&mut buffer, &mut colored_buffer,  erosion_iterations, talus_angle, erosion_mode);
                        }
                        if calculate_rivers {
                            match simulate_river_flow(&mut buffer, &mut colored_buffer, river_iterations, erosion_factor, river_amount){
                                Ok(_) => {},
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            }
                        }
                        *locked_buffer = buffer.clone();
                        *locked_color_buffer = colored_buffer.clone();
                        let pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&buffer.into_raw().as_slice(), IMAGE_SIZE, IMAGE_SIZE); 
                        let colored_pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&colored_buffer.into_raw().as_slice(), IMAGE_SIZE, IMAGE_SIZE);
                        let weak_copy = handle.clone();
                        match slint::invoke_from_event_loop(move || {
                            let img = slint::Image::from_rgba8(pixel_buffer);
                            let colormap = slint::Image::from_rgba8(colored_pixel_buffer);
                            let weak = weak_copy.upgrade().unwrap();
                            weak.set_image(img);
                            weak.set_colormap(colormap);
                        }){
                            Ok(_) => {},
                            Err(e) => {
                                println!("Error: {}", e);
                            }
                        };
                    }
                    (Err(e), _) => {
                        println!("Error in Height Buffer: {}", e);
                    },
                    (_, Err(e)) => {
                        println!("Error in Color Buffer: {}", e);
                    }
                }
            });
        }
    });

    app.on_add_layer_btn_clicked(move ||{
        let clicked_handle = app_add_weak.upgrade().unwrap();
        let model_rc = clicked_handle.get_layers();
        let layers = model_rc.as_any().downcast_ref::<VecModel<LayerParams>>().unwrap();
        layers.push(LayerParams {
            scale: 0.05,
            offset_x: 0.0,
            offset_y: 0.0,
            seed: 1.0,
            opacity: 1.0,
            blend_mode: 0,
        });
    });

    app.on_remove_layer_btn_clicked(move ||{
        let clicked_handle = app_remove_weak.upgrade().unwrap();
        let model_rc = clicked_handle.get_layers();
        let layers = model_rc.as_any().downcast_ref::<VecModel<LayerParams>>().unwrap();
        if layers.iter().count() > 0 {
            layers.remove(layers.iter().count() - 1);
        }
    });

    app.on_export_btn_clicked(move || {
        let clicked_handle = app_export_weak.upgrade().unwrap();
        let locked_buffer = export_main_buffer.lock().unwrap();
        let locked_color_buffer = export_main_color_buffer.lock().unwrap();
        let filename = clicked_handle.get_filename();
        let mut buffer = locked_buffer.clone();
        let mut color_buffer = locked_color_buffer.clone();
        match scale_image(&mut buffer, EXPORT_IMAGE_SIZE, Lanczos3){
            Ok(_) => {},
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        match scale_image(&mut color_buffer, EXPORT_IMAGE_SIZE, Lanczos3){
            Ok(_) => {},
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        save_image_to_desktop(&buffer, filename.as_str(),"height");
        save_image_to_desktop(&color_buffer, filename.as_str(), "color");
    });
    app.run().unwrap();
}

fn generate_perlin_noise_buffer(width: u32, height: u32, offset_x: f64, offset_y: f64, scale: f64, opacity: f64, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let perlin = Perlin::new(seed);

    return ImageBuffer::from_fn(width, height, |x, y| {
        let x = (x as f64 + offset_x) * scale;
        let y = (y as f64 + offset_y) * scale;
        let noise_val = perlin.get([x, y]) * 0.5 + 0.5;
        let color = (noise_val * 255.0) as u8;
        Rgba([color, color, color,(opacity * 255.0) as u8])
    });    
}

fn blend_buffers(buffer_a: &ImageBuffer<Rgba<u8>, Vec<u8>>, buffer_b: &ImageBuffer<Rgba<u8>, Vec<u8>>, blend_mode: i32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
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

fn save_image_to_desktop(buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>, filename: &str, suffix: &str){
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

fn thermal_erosion(
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

                        if should_erode(center_pixel.clone(), neighbor_pixel.clone(), talus_angle, erosion_mode) {
                            // Update both heightmap and colormap
                            heightmap.put_pixel(x, y, neighbor_pixel.clone());
                            colormap.put_pixel(x, y, neighbor_color.clone());
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



fn clamp_image_buffer(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, min: u8, max: u8) {
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

fn colorize_buffer(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
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

fn simulate_river_flow(
    heightmap: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    colormap: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    rain_iterations: usize,
    erosion_factor: i16,
    num_rivers: usize,
) -> Result<(), String> {
    let (width, height) = heightmap.dimensions();
    let mut rng = rand::thread_rng();
    
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
                    //heightmap_i16[min_x as usize][min_y as usize] = new_height;

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


fn scale_image(buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, target_size: (u32, u32), scale_method: FilterType) -> Result<(), Box<dyn Error>> {
    let (target_width, target_height) = target_size;

    if target_width == 0 || target_height == 0 {
        return Err("Target size should be greater than zero".into());
    }

    let scaled_image = image::imageops::resize(buffer, target_width, target_height, scale_method);
    *buffer = scaled_image;

    Ok(())
}
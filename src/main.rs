mod heightmap_gen;

extern crate image;
extern crate dirs;
extern crate serde;
extern crate serde_json;

use image::{ImageBuffer, Rgba, imageops::FilterType};
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs;
use std::io::Read;
use slint::{slint, Model, VecModel,SharedPixelBuffer,Rgba8Pixel};
use serde_derive::{Serialize, Deserialize};


use heightmap_gen::heightmap::{generate_perlin_noise_buffer, blend_buffers, colorize_buffer, clamp_image_buffer, thermal_erosion, simulate_river_flow, scale_image, save_image_to_desktop};
use heightmap_gen::constants::IMAGE_SIZE;



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
        title: "Heightmap Generator";
        icon: @image-url("images/icon.png");
        min-width: 850px;
        background: #161616;
        
        callback ui_changed;
        callback load_btn_clicked <=> load_btn.clicked;
        callback export_btn_clicked <=> btn.clicked;
        callback add_layer_btn_clicked <=> add_layer_btn.clicked;
        callback remove_layer_btn_clicked <=> remove_layer_btn.clicked;
        
        in-out property <float> scale <=> scl.value;
        in-out property <float> offset_x <=> ofx.value;
        in-out property <float> offset_y <=> ofy.value;
        in-out property <float> seed <=> sd.value;
        in-out property <image> image <=> img.source;
        in-out property <image> colormap <=> colormap.source;
        in-out property <[LayerParams]> layers: [];

        in-out property <string> filename <=> filename.text;
        in-out property <int> export_scale <=> export_scale.current-index;
        in-out property <int> scale_type <=> export_filter.current-index;

        in-out property <int> erosion_mode <=> erosion_mode.current-index;
        in-out property <float> erosion_iterations <=> erosion_iterations.value;
        in-out property <float> talus_angle <=> talus_angle.value;

        in-out property <bool> flatten_enabled <=> flatten_enabled.checked;
        in-out property <float> ground_level <=> ground_level.value;

        in-out property <bool> calculate_rivers <=> river_enabled.checked;
        in-out property <float> river_iterations <=> river_iterations.value;
        in-out property <float> erosion_factor <=> erosion_factor.value;
        in-out property <float> river_amount <=> river_amount.value;
        in-out property <float> river_seed <=> river_seed.value;

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
                            HorizontalBox {
                                Text {text: "Seed"; vertical-alignment: center;}
                                river_seed:=Slider {enabled: erosion-mode.current-index != 0 ;value: 1;minimum: 1;maximum: 5000; height: 25px; changed => {
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
                            width: 160px;
                            filename:=TextInput {
                                single-line: true;
                                text: "noise";
                                height: 30px;
                                width: 150px;
                                vertical-alignment: center;
                                horizontal-alignment: left;
                            }
                        }
                        
                    }
                    HorizontalBox {
                        VerticalBox {
                            Text {text: "Scale"; vertical-alignment: center;}
                            export_scale:=ComboBox{
                                model: ["256","515","1024","2048","4096"];
                                current-index: 0;
                                height: 25px;
                            }
                        }
                        VerticalBox {
                            Text {text: "Filter"; vertical-alignment: center;}
                            export_filter:=ComboBox{
                                model: ["Nearest","Triangle","CatmullRom","Gaussian","Lanczos3"];
                                current-index: 4;
                                height: 25px;
                            }
                        }
                    }
                    HorizontalBox {
                        load_btn:=Button {height: 30px; text: "Try Load Texture";}
                        btn:=Button {height: 30px; text: "Export Texture";}
                    }
                    
                    Text {
                        text: "Exported textures will be saved or loaded to/from your Desktop";
                        color: #8a8a8a;
                        font-size: 10px;
                        font-italic: true;
                    }
                }
            }
        }
          
    }      
        
}

#[derive(Serialize, Deserialize)]
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
    let app_load_weak = app_weak.clone();
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
            let erosion_mode = clicked_handle.get_erosion_mode();
            let erosion_iterations = clicked_handle.get_erosion_iterations() as usize;
            let talus_angle = clicked_handle.get_talus_angle();
            let flatten_enabled = clicked_handle.get_flatten_enabled();
            let ground_level = clicked_handle.get_ground_level() as u8;
            let calculate_rivers = clicked_handle.get_calculate_rivers();
            let river_iterations = clicked_handle.get_river_iterations() as usize;
            let erosion_factor = clicked_handle.get_erosion_factor() as i16;
            let river_amount = clicked_handle.get_river_amount() as usize;
            let river_seed = clicked_handle.get_river_seed() as u64;

            let mut layers: Vec<Layers> = Vec::new();
            for layer in layer_parms.iter() {
                layers.push(Layers {
                    scale: layer.scale as f64,
                    offset_x: layer.offset_x as f64,
                    offset_y: layer.offset_y as f64,
                    seed: layer.seed as u32,
                    opacity: layer.opacity as f64,
                    blend_mode: layer.blend_mode,
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
                            let layer_buffer = generate_perlin_noise_buffer(IMAGE_SIZE,IMAGE_SIZE,layer.offset_x,layer.offset_y,layer.scale,layer.opacity,layer.seed);
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
                            match simulate_river_flow(&mut buffer, &mut colored_buffer, river_iterations, erosion_factor, river_amount,river_seed){
                                Ok(_) => {},
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            }
                        }
                        *locked_buffer = buffer.clone();
                        *locked_color_buffer = colored_buffer.clone();
                        let pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(buffer.into_raw().as_slice(), IMAGE_SIZE, IMAGE_SIZE); 
                        let colored_pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(colored_buffer.into_raw().as_slice(), IMAGE_SIZE, IMAGE_SIZE);
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

    app.on_load_btn_clicked(move ||{
        deserialize_tool(&app_load_weak);
    });

    app.on_export_btn_clicked(move || {
        let clicked_handle = app_export_weak.upgrade().unwrap();
        let locked_buffer = export_main_buffer.lock().unwrap();
        let locked_color_buffer = export_main_color_buffer.lock().unwrap();        
        let mut buffer = locked_buffer.clone();
        let mut color_buffer = locked_color_buffer.clone();

        let filename = clicked_handle.get_filename();
        let export_scale = clicked_handle.get_export_scale() as u32;
        let export_filter = clicked_handle.get_scale_type() as u32;

        let image_size = match export_scale {
            0 => {IMAGE_SIZE},
            1 => {IMAGE_SIZE * 2},
            2 => {IMAGE_SIZE * 4},
            3 => {IMAGE_SIZE * 8},
            4 => {IMAGE_SIZE * 16},
            _ => {IMAGE_SIZE}
        };

        let image_filter = match export_filter {
            0 => {FilterType::Nearest},
            1 => {FilterType::Triangle},
            2 => {FilterType::CatmullRom},
            3 => {FilterType::Gaussian},
            4 => {FilterType::Lanczos3},
            _ => {FilterType::Lanczos3}
        };

        match scale_image(&mut buffer, (image_size,image_size), image_filter){
            Ok(_) => {},
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        match scale_image(&mut color_buffer, (image_size,image_size), image_filter){
            Ok(_) => {},
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        serialize_tool(&app_export_weak);
        save_image_to_desktop(&buffer, filename.as_str(),"height");
        save_image_to_desktop(&color_buffer, filename.as_str(), "color");
    });
    app.run().unwrap();
}



#[derive(Serialize, Deserialize)]
struct SerializedTool{
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    seed: u32,
    layers: String,
    erosion_mode: i32,
    erosion_iterations: usize,
    talus_angle: f32,
    flatten_enabled: bool,
    ground_level: u8,
    calculate_rivers: bool,
    river_iterations: usize,
    erosion_factor: i16,
    river_amount: usize,
    river_seed: u64,
    filename: String,
    export_scale: u32,
    export_filter: u32,
}

fn deserialize_tool(weak: &slint::Weak<App>){
    let handle = weak.upgrade().unwrap();
    let file_name = handle.get_filename();
    let desktop_path = dirs::desktop_dir();
    match desktop_path {
        Some(path) =>{
            let file_path = path.join(format!("{}_config.json",file_name));
            let mut file = match fs::File::open(file_path) {
                Ok(file) => file,
                Err(e) => {
                    println!("Couldn't open file: {}", e);
                    return;
                }
            };
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {},
                Err(e) => {
                    println!("Couldn't read file: {}", e);
                    return;
                }
            }
            let serialized_tool: SerializedTool = match serde_json::from_str(&contents) {
                Ok(tool) => tool,
                Err(e) => {
                    println!("Couldn't deserialize tool: {}", e);
                    return;
                }
            };
            handle.set_scale(serialized_tool.scale as f32);
            handle.set_offset_x(serialized_tool.offset_x as f32);
            handle.set_offset_y(serialized_tool.offset_y as f32);
            handle.set_seed(serialized_tool.seed as f32);
            let layers: Vec<Layers> = match serde_json::from_str(&serialized_tool.layers) {
                Ok(layers) => layers,
                Err(e) => {
                    println!("Couldn't deserialize layers: {}", e);
                    return;
                }
            };
            let model_rc = handle.get_layers();
            let layer_parms = model_rc.as_any().downcast_ref::<VecModel<LayerParams>>().unwrap();
            while layer_parms.iter().count() > 0 {
                layer_parms.remove(0);
            }
            for layer in layers {
                layer_parms.push(LayerParams {
                    scale: layer.scale as f32,
                    offset_x: layer.offset_x as f32,
                    offset_y: layer.offset_y as f32,
                    seed: layer.seed as f32,
                    opacity: layer.opacity as f32,
                    blend_mode: layer.blend_mode,
                });
            }
            handle.set_erosion_mode(serialized_tool.erosion_mode as i32);
            handle.set_erosion_iterations(serialized_tool.erosion_iterations as f32);
            handle.set_talus_angle(serialized_tool.talus_angle as f32);
            handle.set_flatten_enabled(serialized_tool.flatten_enabled as bool);
            handle.set_ground_level(serialized_tool.ground_level as f32);
            handle.set_calculate_rivers(serialized_tool.calculate_rivers as bool);
            handle.set_river_iterations(serialized_tool.river_iterations as f32);
            handle.set_erosion_factor(serialized_tool.erosion_factor as f32);
            handle.set_river_amount(serialized_tool.river_amount as f32);
            handle.set_river_seed(serialized_tool.river_seed as f32);
            handle.set_filename(slint::SharedString::from(serialized_tool.filename));
            handle.set_export_scale(serialized_tool.export_scale as i32);
            handle.set_scale_type(serialized_tool.export_filter as i32);            
        },
        None => {
            println!("Couldn't find desktop path");
        }     
    }
}

fn serialize_tool(weak: &slint::Weak<App>){
    let handle = weak.upgrade().unwrap();
    let scale = handle.get_scale() as f64;
    let offset_x = handle.get_offset_x() as f64;
    let offset_y = handle.get_offset_y() as f64;
    let seed = handle.get_seed() as u32;
    let model_rc = handle.get_layers();
    let layer_parms = model_rc.as_any().downcast_ref::<VecModel<LayerParams>>().unwrap();
    let erosion_mode = handle.get_erosion_mode();
    let erosion_iterations = handle.get_erosion_iterations() as usize;
    let talus_angle = handle.get_talus_angle();
    let flatten_enabled = handle.get_flatten_enabled();
    let ground_level = handle.get_ground_level() as u8;
    let calculate_rivers = handle.get_calculate_rivers();
    let river_iterations = handle.get_river_iterations() as usize;
    let erosion_factor = handle.get_erosion_factor() as i16;
    let river_amount = handle.get_river_amount() as usize;
    let river_seed = handle.get_river_seed() as u64;
    let file_name = handle.get_filename();
    let export_scale = handle.get_export_scale() as u32;
    let export_filter = handle.get_scale_type() as u32;
    let mut layers: Vec<Layers> = Vec::new();
    for layer in layer_parms.iter() {
        layers.push(Layers {
            scale: layer.scale as f64,
            offset_x: layer.offset_x as f64,
            offset_y: layer.offset_y as f64,
            seed: layer.seed as u32,
            opacity: layer.opacity as f64,
            blend_mode: layer.blend_mode,
        });
    }
    let serialized_tool = SerializedTool {
        scale,
        offset_x,
        offset_y,
        seed,
        layers: serde_json::to_string(&layers).unwrap(),
        erosion_mode,
        erosion_iterations,
        talus_angle,
        flatten_enabled,
        ground_level,
        calculate_rivers,
        river_iterations,
        erosion_factor,
        river_amount,
        river_seed,
        filename: file_name.to_string(),
        export_scale,
        export_filter,
    };

    let serialized_tool_json = serde_json::to_string_pretty(&serialized_tool).unwrap();
    let desktop_path = dirs::desktop_dir();
    match desktop_path {
        Some(path) => {            
            let full_path = path.join(format!("{}_config.json",file_name));
            println!("Desktop path: {}", full_path.display());
            match fs::write(full_path, serialized_tool_json) {
                Ok(_) => {
                    println!("Tool saved");
                },
                Err(e) => {
                    println!("Couldn't save tool: {}", e);
                }
            }
                
        },
        None => {
            println!("Couldn't find desktop path");
        }
    }    
}
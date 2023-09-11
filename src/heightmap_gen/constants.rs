use image::Rgba;

pub const IMAGE_SIZE: u32 = 256;
pub const COLORS: [Rgba<u8>; 10] = [
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
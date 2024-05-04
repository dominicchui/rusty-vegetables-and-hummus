use crate::{constants, ecology::Ecosystem, render::EcosystemRenderable};
use image::io::Reader as ImageReader;

pub fn import_height_map(path: &str) -> EcosystemRenderable {
    println!("Reading height map at {path}");
    // read png image as height map
    let img = ImageReader::open(path).unwrap().decode().unwrap();
    let rgb8_vec = img.into_rgb8();

    // create ecosystem terrain based on the height map
    let mut heights = [0.0; constants::AREA_SIDE_LENGTH * constants::AREA_SIDE_LENGTH];
    // input is a u8, so a scaling factor of 0.1 means max height is 25.6m
    let height_scaling_factor = 1.0; 
    for (i, pixel) in rgb8_vec.pixels().enumerate() {
        let height = pixel.0[0] as f32 * height_scaling_factor;
        heights[i] = height;
    }
    let ecosystem = Ecosystem::init_with_heights(heights);

    EcosystemRenderable::init(ecosystem)
}


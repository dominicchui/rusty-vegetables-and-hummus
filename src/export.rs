use crate::{constants, ecology::{CellIndex, Ecosystem}, render::EcosystemRenderable};

/// process:
/// generate height map and density maps for all layers
/// in blender, blend colors together, add textures, instantiate geometry

pub(crate) fn export_maps(ecosystem: &Ecosystem, time_step: u32) {
    export_height_map(ecosystem, time_step);
    export_color_map(ecosystem, time_step);
}


pub(crate) fn export_height_map(ecosystem: &Ecosystem, time_step: u32) {
    // get date and time for file path

    let now = chrono::Local::now();
    let today = now.date_naive().format("%Y-%m-%d").to_string();
    // println!("today {today}");
    let time = now.time().format("%H-%M-%S").to_string();
    // println!("time {time}");
    let path = "../output/".to_owned() + &today + "-" + &time + "-terrain-" + &time_step.to_string() +".png";
    println!("{path}");

    let buf = build_height_map(ecosystem);
    println!("{buf:?}");
    image::save_buffer(path, &buf, constants::AREA_SIDE_LENGTH as u32, constants::AREA_SIDE_LENGTH as u32, image::ColorType::Rgb8).unwrap();
}

pub(crate) fn build_height_map(ecosystem: &Ecosystem) -> [u8; constants::NUM_CELLS * 3] {
    let mut heights = [0.0; constants::NUM_CELLS];
    let mut min_height = f32::MAX;
    let mut max_height = f32::MIN;
    for (i, row) in ecosystem.cells.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            let flat_index = i + j * constants::AREA_SIDE_LENGTH;
            let height = cell.get_height();
            heights[flat_index] = height;
            if height > max_height { max_height = height; }
            if height < min_height { min_height = height; }
        }
    }
    // normalize heights to fit within 256 values
    let norm_factor = 256.0 / (max_height - min_height);
    heights = heights.map(|v| (v-min_height) * norm_factor);

    // convert to greyscale rgb
    let mut buffer = [0; constants::NUM_CELLS * 3];
    for (i, height) in heights.iter().enumerate() {
        let height = *height as u8;
        buffer[i*3] = height;
        buffer[i*3+1] = height;
        buffer[i*3+2] = height;
    }
    buffer
}

pub(crate) fn export_color_map(ecosystem: &Ecosystem, time_step: u32) {
    // get date and time for file path

    let now = chrono::Local::now();
    let today = now.date_naive().format("%Y-%m-%d").to_string();
    // println!("today {today}");
    let time = now.time().format("%H-%M-%S").to_string();
    // println!("time {time}");
    let path = "./../output/".to_owned() + &today + "-" + &time + "-color" + &time_step.to_string() + ".png";
    println!("{path}");

    let buf = build_color_map(ecosystem);
    println!("{buf:?}");
    image::save_buffer(path, &buf, constants::AREA_SIDE_LENGTH as u32, constants::AREA_SIDE_LENGTH as u32, image::ColorType::Rgb8).unwrap();
}

pub(crate) fn build_color_map(ecosystem: &Ecosystem) -> [u8; constants::NUM_CELLS * 3] {
    let mut buffer = [0; constants::NUM_CELLS * 3];
    for i in 0..constants::AREA_SIDE_LENGTH {
        for j in 0..constants::AREA_SIDE_LENGTH {
            let flat_index = i + j * constants::AREA_SIDE_LENGTH;
            let color = EcosystemRenderable::get_color(ecosystem, CellIndex::new(i, j));
            buffer[flat_index*3] = (color[0] * 255.0) as u8;
            buffer[flat_index*3 + 1] = (color[1] * 255.0) as u8;
            buffer[flat_index*3 + 2] = (color[2] * 255.0) as u8;
        }
    }
    buffer
}
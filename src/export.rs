use crate::{
    constants, ecology::{CellIndex, Ecosystem}, render::EcosystemRenderable
};

/// process:
/// generate height map and density maps for all layers
/// in blender, blend colors together, add textures, instantiate geometry

pub(crate) fn export_maps(ecosystem: &Ecosystem, time_step: u32, path: &str) {
    export_height_map(ecosystem, time_step, path);
    export_color_map(ecosystem, time_step, path);
    // todo make more efficient
    export_hypsometric_color_map(build_height_map(ecosystem), time_step, path);
    export_vegetation_map(ecosystem, time_step, path);
}

pub(crate) fn export_height_map(ecosystem: &Ecosystem, time_step: u32, path: &str) {
    let path = format!("{path}/{}-terrain.png", time_step);
    println!("{path}");

    let buf = build_height_map(ecosystem);
    image::save_buffer(
        path,
        &buf,
        constants::AREA_SIDE_LENGTH as u32,
        constants::AREA_SIDE_LENGTH as u32,
        image::ColorType::Rgb8,
    )
    .unwrap();
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
            if height > max_height {
                max_height = height;
            }
            if height < min_height {
                min_height = height;
            }
        }
    }
    // normalize heights to fit within 256 values
    let norm_factor = 256.0 / (max_height - min_height);
    heights = heights.map(|v| (v - min_height) * norm_factor);

    // convert to greyscale rgb
    let mut buffer = [0; constants::NUM_CELLS * 3];
    for (i, height) in heights.iter().enumerate() {
        let height = *height as u8;
        buffer[i * 3] = height;
        buffer[i * 3 + 1] = height;
        buffer[i * 3 + 2] = height;
    }
    buffer
}

pub(crate) fn export_color_map(ecosystem: &Ecosystem, time_step: u32, path: &str) {
    let path = format!("{path}/{}-color.png", time_step);
    println!("{path}");

    let buf = build_color_map(ecosystem);
    image::save_buffer(
        path,
        &buf,
        constants::AREA_SIDE_LENGTH as u32,
        constants::AREA_SIDE_LENGTH as u32,
        image::ColorType::Rgb8,
    )
    .unwrap();
}

pub(crate) fn build_color_map(ecosystem: &Ecosystem) -> [u8; constants::NUM_CELLS * 3] {
    let mut buffer = [0; constants::NUM_CELLS * 3];
    for i in 0..constants::AREA_SIDE_LENGTH {
        for j in 0..constants::AREA_SIDE_LENGTH {
            let flat_index = i + j * constants::AREA_SIDE_LENGTH;
            let color = EcosystemRenderable::get_color(ecosystem, CellIndex::new(i, j));
            buffer[flat_index * 3] = (color[0] * 255.0) as u8;
            buffer[flat_index * 3 + 1] = (color[1] * 255.0) as u8;
            buffer[flat_index * 3 + 2] = (color[2] * 255.0) as u8;
        }
    }
    buffer
}

pub(crate) fn export_hypsometric_color_map(
    height_map: [u8; constants::NUM_CELLS * 3],
    time_step: u32,
    path: &str,
) {
    let path = format!("{path}/{}-hypsometric.png", time_step);
    println!("{path}");

    let buf = build_hypsometrically_tinted_map(height_map);
    image::save_buffer(
        path,
        &buf,
        constants::AREA_SIDE_LENGTH as u32,
        constants::AREA_SIDE_LENGTH as u32,
        image::ColorType::Rgb8,
    )
    .unwrap();
}

pub(crate) fn build_hypsometrically_tinted_map(
    height_map: [u8; constants::NUM_CELLS * 3],
) -> [u8; constants::NUM_CELLS * 3] {
    let mut buffer = [0; constants::NUM_CELLS * 3];
    for i in (0..height_map.len()).step_by(3) {
        let height = height_map[i] as f32;
        let color = EcosystemRenderable::get_hypsometric_color_helper(height, false);
        buffer[i] = (color[0] * 255.0) as u8;
        buffer[i + 1] = (color[1] * 255.0) as u8;
        buffer[i + 2] = (color[2] * 255.0) as u8;
    }
    buffer
}

pub(crate) fn export_vegetation_map(ecosystem: &Ecosystem, time_step: u32, path: &str) {
    let path = format!("{path}/{}-vegetation.png", time_step);
    println!("{path}");

    let buf = build_vegetation_map(ecosystem);
    image::save_buffer(
        path,
        &buf,
        constants::AREA_SIDE_LENGTH as u32,
        constants::AREA_SIDE_LENGTH as u32,
        image::ColorType::Rgb8,
    )
    .unwrap();
}

pub(crate) fn build_vegetation_map(ecosystem: &Ecosystem) -> [u8; constants::NUM_CELLS * 3] {
    // r channel is for trees
    // g channel is for bushes
    let mut buffer = [0; constants::NUM_CELLS * 3];

    // for starters, use average height as density proxy
    for i in 0..constants::AREA_SIDE_LENGTH {
        for j in 0..constants::AREA_SIDE_LENGTH {
            let index = CellIndex::new(i, j);
            let flat_index = i + j * constants::AREA_SIDE_LENGTH;
            let trees_color = if let Some(trees) = ecosystem[index].trees.as_ref() {
                let avg_height = trees.plant_height_sum / trees.number_of_plants as f32;
                (avg_height * 8.0) as u8
            } else {
                0
            };
            let bushes_color = if let Some(bushes) = ecosystem[index].bushes.as_ref() {
                let avg_height = bushes.plant_height_sum / bushes.number_of_plants as f32;
                (avg_height * 60.0) as u8
            } else {
                0
            };
            buffer[flat_index * 3] = trees_color;
            buffer[flat_index * 3 + 1] = bushes_color;
            buffer[flat_index * 3 + 2] = 0;
        }
    }

    buffer
}

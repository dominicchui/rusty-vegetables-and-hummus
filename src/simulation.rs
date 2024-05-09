use gl::types::GLuint;
use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::{
    constants,
    ecology::{CellIndex, Ecosystem},
    events::Events,
    import::import_height_map,
    render::{ColorMode, EcosystemRenderable},
};

pub struct Simulation {
    pub ecosystem: EcosystemRenderable,
}

impl Simulation {
    pub fn init() -> Self {
        let ecosystem = Ecosystem::init_standard();
        Simulation {
            ecosystem: EcosystemRenderable::init(ecosystem),
        }
    }

    pub fn init_with_height_map(path: &str) -> Self {
        Simulation {
            ecosystem: import_height_map(path),
        }
    }

    pub fn draw(&mut self, program_id: GLuint, render_mode: gl::types::GLuint) {
        self.ecosystem.draw(program_id, render_mode);
    }

    pub fn take_time_step(&mut self, color_mode: &ColorMode) {
        // iterate over all cells
        let num_cells = constants::AREA_SIDE_LENGTH * constants::AREA_SIDE_LENGTH;

        let mut vec: Vec<usize> = (0..num_cells).collect();
        vec.shuffle(&mut thread_rng());

        for i in vec {
            // apply random event
            let mut events = [
                // Events::Lightning,
                // Events::ThermalStress,
                // Events::SandSlide,
                // Events::RockSlide,
                // Events::HumusSlide,
                Events::VegetationTrees,
                Events::VegetationBushes,
                Events::VegetationGrasses,
                Events::Rainfall,
            ];
            events.shuffle(&mut thread_rng());
            // println!("Events {events:?}");

            let index = CellIndex::get_from_flat_index(i);
            for event in events {
                Events::apply_event(event, &mut self.ecosystem.ecosystem, index);
            }
            // let cell = &self.ecosystem.ecosystem[index];
            // humus_heights.push(cell.get_humus_height());
            // println!("{index} sunlight {:?}", cell.hours_of_sunlight);
            // println!("height {}", cell.get_height());
        }

        // println!("humus heights {humus_heights:?}");
        // let index = CellIndex::new(50, 50);
        // let cell = &self.ecosystem.ecosystem[index];
        // println!("rocks_height {}", cell.get_rock_height());
        // println!("humus_height {}", cell.get_humus_height());

        self.ecosystem.update_vertices(color_mode);
    }

    pub fn change_color_mode(&mut self, color_mode: &ColorMode) {
        self.ecosystem.update_vertices(color_mode);
    }
}

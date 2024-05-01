use gl::types::GLuint;
use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::{constants, ecology::CellIndex, events::Events, render::EcosystemRenderable};

pub struct Simulation {
    pub ecosystem: EcosystemRenderable,
}

impl Simulation {
    pub fn init() -> Self {
        Simulation {
            ecosystem: EcosystemRenderable::init(),
        }
    }

    pub fn draw(&mut self, program_id: GLuint, render_mode: gl::types::GLuint) {
        self.ecosystem.draw(program_id, render_mode);
    }

    pub fn take_time_step(&mut self) {
        // update sunlight computations
        // self.ecosystem.ecosystem.recompute_sunlight();

        // iterate over all cells
        let num_cells = constants::AREA_SIDE_LENGTH * constants::AREA_SIDE_LENGTH;

        let mut vec: Vec<usize> = (0..num_cells).collect();
        vec.shuffle(&mut thread_rng());

        for i in vec {
            // apply random event
            let mut events = [
                Events::Lightning,
                Events::ThermalStress,
                Events::SandSlide,
                Events::RockSlide,
                Events::HumusSlide,
                Events::VegetationTrees,
                Events::VegetationBushes,
                Events::VegetationGrasses,
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
        // let index = CellIndex::new(2, 2);
        // let cell = &self.ecosystem.ecosystem[index];
        // let rocks_height = cell.get_rock_height();
        // println!("rocks_height {rocks_height}");

        self.ecosystem.update_vertices();
    }
}

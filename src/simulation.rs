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
        // iterate over all cells
        let num_cells = constants::AREA_SIDE_LENGTH * constants::AREA_SIDE_LENGTH;

        let mut vec: Vec<usize> = (0..num_cells).collect();
        vec.shuffle(&mut thread_rng());

        // let i = vec[0];
        // let index = CellIndex::get_from_flat_index(i);
        // println!("index {index:?}");
        // Events::apply_and_propagate_lightning_event(&mut self.ecosystem.ecosystem, index);
        let mut total_height = 0.0;
        for i in 0..num_cells {
            let index = CellIndex::get_from_flat_index(i);
            total_height += self.ecosystem.ecosystem[index].get_height();
        }
        // println!("average height {}", total_height / num_cells as f32);

        for i in vec {
            // apply random event
            // just lightning for now
            let index = CellIndex::get_from_flat_index(i);
            // Events::apply_event(Events::Lightning, &mut self.ecosystem.ecosystem, index);
            Events::apply_event(Events::SandSlide, &mut self.ecosystem.ecosystem, index);
            Events::apply_event(Events::RockSlide, &mut self.ecosystem.ecosystem, index);
            Events::apply_event(Events::HumusSlide, &mut self.ecosystem.ecosystem, index);
        }

        self.ecosystem.update_vertices();
    }
}

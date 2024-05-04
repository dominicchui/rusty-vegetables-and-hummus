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
                Events::Rainfall,
            ];
            events.shuffle(&mut thread_rng());
            // println!("Events {events:?}");

            let index = CellIndex::get_from_flat_index(i);
            for event in events {
                Events::apply_event(event, &mut self.ecosystem.ecosystem, index);
            }
        }
        self.ecosystem.update_vertices();
    }
}

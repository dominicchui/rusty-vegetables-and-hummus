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
        // let mut total_height = 0.0;
        // for i in 0..num_cells {
        //     let index = CellIndex::get_from_flat_index(i);
        //     total_height += self.ecosystem.ecosystem[index].get_height();
        // }
        // println!("average height {}", total_height / num_cells as f32);

        for i in vec {
            // apply random event (todo randomize)
            let index = CellIndex::get_from_flat_index(i);
            // Events::apply_event(Events::Lightning, &mut self.ecosystem.ecosystem, index);
            // Events::apply_event(Events::ThermalStress, &mut self.ecosystem.ecosystem, index);
            // Events::apply_event(Events::SandSlide, &mut self.ecosystem.ecosystem, index);
            // Events::apply_event(Events::RockSlide, &mut self.ecosystem.ecosystem, index);
            // Events::apply_event(Events::HumusSlide, &mut self.ecosystem.ecosystem, index);
            Events::apply_event(Events::Rainfall, &mut self.ecosystem.ecosystem, index);
        }

        // let select_cell = vec.get(0);
        // let us: usize;

        // match select_cell {
        //     Some(x) => us = *x,
        //     None => us = 0
        // }
   
        // Events::apply_event(Events::Rainfall, &mut self.ecosystem.ecosystem, CellIndex::get_from_flat_index(us));

        let index = CellIndex::new(2, 2);
        let cell = &self.ecosystem.ecosystem[index];
        let rocks_height = cell.get_height_of_rock();
        println!("rocks_height {rocks_height}");

        self.ecosystem.update_vertices();
    }
}

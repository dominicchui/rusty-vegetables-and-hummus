use std::collections::HashMap;

use super::Events;
use crate::{
    constants,
    ecology::{Cell, CellIndex, Ecosystem, Neighbors},
};

use rand::{distributions::Distribution, Rng};
use rand::distributions::WeightedIndex;

impl Events {
    pub(crate) fn apply_rainfall_event(ecosystem: &mut Ecosystem, index: CellIndex) -> Option<(Events, CellIndex)> {
        let water_level: f32 = 0.001*ecosystem[index].get_height();

        //TODO: Account for plants intercepting rainfall

        Self::runoff(ecosystem, index, water_level, 0);

        None
    }

    fn runoff(ecosystem: &mut Ecosystem, index: CellIndex, water_level: f32, steps: usize) -> () {
        let neighbors: [Option<CellIndex>; 8] = Cell::get_neighbors(&index).as_array();
        const NUM_NEIGHBORS: usize = 8;

        let mut slopes: Vec<f32> = Vec::new();
        let mut existing_neighbors: Vec<CellIndex> = Vec::new();

        for i in 0..NUM_NEIGHBORS {
            let neighbor_option: Option<CellIndex> = neighbors[i];
            let neighbor: CellIndex;

            match neighbor_option {
                Some(x) => neighbor = x,
                None => continue
            }

            let slope: f32 = ecosystem.get_slope_between_points(index, neighbor);

            if (slope > 0.0) {
                slopes.push(slope);
                existing_neighbors.push(neighbor);
            }
        }

        //Decide which cell the water will flow to
        let chosen_slope: f32;
        let next_cell_index: CellIndex;
        
        if slopes.len() != 0 {
            let dist = WeightedIndex::new(&slopes).unwrap();
            let mut rng = rand::thread_rng();

            let choice: usize = dist.sample(&mut rng);

            chosen_slope = slopes[choice];
            next_cell_index = existing_neighbors[choice];

            //Soil absorption

            let cur_cell = &mut ecosystem[index];

            println!("Moved water from {index} to {next_cell_index}");

            //ecosystem[next_cell].soil_moisture += chosen_slope;
            
            //Erosion

            let sediment_capacity: f32 = constants::KC*water_level; //CS
            let h_amt = cur_cell.get_height_of_humus();
            let r_amt = cur_cell.get_height_of_rock();
            let s_amt = cur_cell.get_height_of_sand();

            let cur_cell_sediment: f32 = h_amt+r_amt+s_amt; //SV

            let percent_humus: f32 = h_amt/cur_cell_sediment;
            let percent_rock: f32 = r_amt/cur_cell_sediment;
            let percent_sand: f32 = s_amt/cur_cell_sediment;

            if (cur_cell_sediment >= sediment_capacity) {
                //Equation 2 unneeded

                //Equation 3
                let cur_cell_sediment_change = cur_cell_sediment-((1.0-constants::KD)*(cur_cell_sediment-sediment_capacity));

                cur_cell.remove_humus(cur_cell_sediment_change*percent_humus);
                cur_cell.remove_rocks(cur_cell_sediment_change*percent_rock);
                cur_cell.remove_sand(cur_cell_sediment_change*percent_sand);

                let next_cell = &mut ecosystem[next_cell_index];

                //Equation 1
                next_cell.add_humus(sediment_capacity*percent_humus);
                next_cell.add_rocks(sediment_capacity*percent_rock);
                next_cell.add_sand(sediment_capacity*percent_sand);
            } else {
                //Amount eroded
                let mut eroded = constants::KS*(sediment_capacity-cur_cell_sediment);

                println!("Eroded {eroded} of bedrock");

                if (eroded > cur_cell.get_height_of_bedrock()) {
                    eroded = cur_cell.get_height_of_bedrock();
                }

                //Equation 2
                cur_cell.remove_bedrock(eroded);

                //Equation 3
                cur_cell.remove_humus(h_amt);
                cur_cell.remove_rocks(r_amt);
                cur_cell.remove_sand(s_amt); 

                let next_cell = &mut ecosystem[next_cell_index];

                //Equation 1
                next_cell.add_rocks(r_amt + eroded);
            }

            if (steps < 100) {
                Self::runoff(ecosystem, next_cell_index, water_level, steps + 1);
            } else {
                println!("Finished");
            }
        }
    }
}
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
        let water_level: f32 = 0.0001*ecosystem[index].get_height();

        //TODO: Account for plants intercepting rainfall

        Self::runoff(ecosystem, index, water_level, [0.0, 0.0, 0.0], 0);

        None
    }

    fn runoff(ecosystem: &mut Ecosystem, index: CellIndex, water_level: f32, lifted_material: [f32; 3], steps: usize) -> () {
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

        let cur_cell = &mut ecosystem[index];

        if slopes.len() != 0 {

            //Decide which cell the water will flow to
            let chosen_slope: f32;
            let next_cell_index: CellIndex;

            let dist = WeightedIndex::new(&slopes).unwrap();
            let mut rng = rand::thread_rng();

            let choice: usize = dist.sample(&mut rng);

            chosen_slope = slopes[choice];
            next_cell_index = existing_neighbors[choice];
                
            //Erosion

            let mut lifted = lifted_material; //**SUM OF THIS** is STV

            if (chosen_slope > 0.2) { //LIFT HAPPENS
                
                let sediment_capacity: f32 = constants::KC*water_level; //CS

                let remaining_capacity = sediment_capacity-(lifted[0]+lifted[1]+lifted[2]);

                let h_amt = cur_cell.get_humus_height();
                let r_amt = cur_cell.get_rock_height();
                let s_amt = cur_cell.get_sand_height();

                let cur_cell_sediment: f32 = h_amt+r_amt+s_amt;

                let percent_humus: f32 = h_amt/cur_cell_sediment;
                let percent_rock: f32 = r_amt/cur_cell_sediment;
                let percent_sand: f32 = s_amt/cur_cell_sediment;

                if cur_cell_sediment >= remaining_capacity && cur_cell_sediment != 0.0 { //SEDIMENT FILLS CAPACITY
                    cur_cell.remove_humus(remaining_capacity*percent_humus);
                    cur_cell.remove_rocks(remaining_capacity*percent_rock);
                    cur_cell.remove_sand(remaining_capacity*percent_sand);

                    lifted[0] += remaining_capacity*percent_humus;
                    lifted[1] += remaining_capacity*percent_rock;
                    lifted[2] += remaining_capacity*percent_sand;
                } else { //ERODE
                    //Equation 3: Pick up all sediment
                    cur_cell.remove_humus(h_amt);
                    cur_cell.remove_rocks(r_amt);
                    cur_cell.remove_sand(s_amt);

                    lifted[0] += h_amt;
                    lifted[1] += r_amt;
                    lifted[2] += s_amt;

                    //Now, erode an amount equal to K_s*(the difference between capacity and current amount held)

                    let mut eroded = constants::KS*(sediment_capacity-(lifted[0]+lifted[1]+lifted[2]));

                    if (eroded > cur_cell.get_bedrock_height()) {
                        eroded = cur_cell.get_bedrock_height();
                    }

                    //Equation 2
                    cur_cell.remove_bedrock(eroded);

                    //Equation 1
                    lifted[1] += eroded;
                }
            } else { //DEPOSIT
                let deposited_humus = constants::KD*lifted[0];
                let deposited_rock = constants::KD*lifted[1];
                let deposited_sand = constants::KD*lifted[2];

                cur_cell.add_humus(deposited_humus);
                cur_cell.add_rocks(deposited_rock);
                cur_cell.add_sand(deposited_sand);

                lifted[0] -= deposited_humus;
                lifted[1] -= deposited_rock;
                lifted[2] -= deposited_sand;
            }

            if (steps < 1000) {
                let h = cur_cell.get_height();

                Self::runoff(ecosystem, next_cell_index, water_level, lifted, steps + 1);
            } else {
                println!("1k steps");
            }
        } else {
            cur_cell.add_humus(lifted_material[0]);
            cur_cell.add_rocks(lifted_material[1]);
            cur_cell.add_sand(lifted_material[2]);
        }
    }
}
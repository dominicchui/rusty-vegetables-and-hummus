// LIGHTNING
// based on ~10 lightning strikes per km per year
// https://www.sciencedirect.com/science/article/pii/S0169555X13003929
const DESIRED_MAX_STRIKES: f32 = 20.0; // strikes per squar kilometer
const MAX_LIGHTNING_PROBABILITY: f32 =
    constants::AREA * DESIRED_MAX_STRIKES / constants::NUM_CELLS as f32;
const LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME: f32 = 4.0; // m^3

use super::Events;
use crate::{
    constants,
    ecology::{Cell, CellIndex, Ecosystem},
};
use rand::Rng;

impl Events {
    pub(crate) fn apply_lightning_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let strike_probability = Self::compute_lightning_damage_probability(ecosystem, index);
        Self::apply_lightning_event_helper(ecosystem, index, strike_probability)
    }

    fn apply_lightning_event_helper(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
        strike_probability: f32,
    ) -> Option<(Events, CellIndex)> {
        let mut rng = rand::thread_rng();
        let rand: f32 = rng.gen();
        if rand < strike_probability {
            // println!("Lightning at {index}");
            let cell = &mut ecosystem[index];

            // kill all vegetation in the cell
            Self::kill_trees(cell);
            Self::kill_bushes(cell);
            Self::kill_grasses(cell);

            // destroy some bedrock and scatter as rocks and sand to nearby cells
            let lost_height = LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME
                / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);
            cell.remove_bedrock(lost_height);

            // simplifying assumption 1: half of the volume becomes rock and the other half sand
            // simplifying assumption 2: distribute volume evenly to 8 neighbors and cell (instead of being based on slope and relative elevation)
            let neighbors = Cell::get_neighbors(&index);
            let num_affected_cells = neighbors.len() + 1;
            let volume_per_cell = LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME / num_affected_cells as f32;
            let height_per_cell =
                volume_per_cell / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);

            // add to cell
            cell.add_rocks(height_per_cell / 2.0);
            cell.add_sand(height_per_cell / 2.0);

            // add to neighbors
            for index in neighbors.as_array().into_iter().flatten() {
                let neighbor = &mut ecosystem[index];
                neighbor.add_rocks(height_per_cell / 2.0);
                neighbor.add_sand(height_per_cell / 2.0);
            }
        }

        // does not propagate
        None
    }

    fn compute_lightning_damage_probability(ecosystem: &Ecosystem, index: CellIndex) -> f32 {
        //l(p)=k_L min(1,e^(k_lc * (∇E(p)−k_ls))
        // k_L is maximum probability
        // k_lc is scaling factor
        // k_ls is minimum curvature required
        let curvature = ecosystem.estimate_curvature(index);
        // println!("index {index}, curvature {curvature}");

        let scaling_factor = 1.0;
        let min_curve = 4.0;
        let exp = scaling_factor * ((-curvature) - min_curve);
        // println!("exp {exp}");
        MAX_LIGHTNING_PROBABILITY * f32::min(1.0, (std::f32::consts::E).powf(exp))
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;

    use crate::{
        constants,
        ecology::{Cell, CellIndex, Ecosystem, Trees},
        events::{lightning::LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME, Events},
    };

    #[test]
    fn test_lightning_event() {
        let index = CellIndex::new(2, 2);
        test_lightning_event_helper(index);
        let index = CellIndex::new(0, 0);
        test_lightning_event_helper(index);
        let index = CellIndex::new(2, 0);
        test_lightning_event_helper(index);
    }

    fn test_lightning_event_helper(index: CellIndex) {
        let mut ecosystem = Ecosystem::init();
        let trees = Trees {
            number_of_plants: 1,
            plant_height_sum: 30.0,
            plant_age_sum: 10.0,
        };
        let cell = &mut ecosystem[index];
        cell.trees = Some(trees);

        let result = Events::apply_lightning_event_helper(&mut ecosystem, index, 1.0);
        assert!(result.is_none());

        // verify trees are dead
        let cell = &ecosystem[index];
        let trees = &cell.trees;
        assert!(trees.is_none());

        // assert bedrock is decreased
        let expected_height = constants::DEFAULT_BEDROCK_HEIGHT
            - LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME
                / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);
        let actual_height = cell.get_bedrock_height();
        assert_eq!(actual_height, expected_height,);

        // assert neighbors and self have increase in rocks and sand
        let neighbors = Cell::get_neighbors(&index);
        let num_neighbors = neighbors.len() + 1;
        let volume_per_cell = LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME / (num_neighbors + 1) as f32;
        let height_per_cell =
            volume_per_cell / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);

        assert!(
            approx_eq!(
                f32,
                cell.get_rock_height(),
                height_per_cell / 2.0,
                epsilon = 0.01
            ),
            "Expected {}, actual {}",
            height_per_cell / 2.0,
            cell.get_rock_height(),
        );

        assert!(
            approx_eq!(
                f32,
                cell.get_sand_height(),
                height_per_cell / 2.0,
                epsilon = 0.01
            ),
            "Expected {}, actual {}",
            height_per_cell / 2.0,
            cell.get_sand_height()
        );

        for neighbor_index in neighbors.as_array().into_iter().flatten() {
            let neighbor = &ecosystem[neighbor_index];
            assert!(
                approx_eq!(
                    f32,
                    neighbor.get_rock_height(),
                    height_per_cell / 2.0,
                    epsilon = 0.01
                ),
                "Expected {}, actual {}",
                height_per_cell / 2.0,
                neighbor.get_rock_height(),
            );
            assert!(
                approx_eq!(
                    f32,
                    cell.get_sand_height(),
                    height_per_cell / 2.0,
                    epsilon = 0.01
                ),
                "Expected {}, actual {}",
                height_per_cell / 2.0,
                cell.get_sand_height()
            );
        }
    }
}

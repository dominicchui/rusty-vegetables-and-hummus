use std::{collections::HashMap, f64::consts::E};

use nalgebra::Vector3;
use rand::Rng;

use crate::{
    constants::{self, CELL_SIDE_LENGTH},
    ecology::{Cell, CellIndex, Ecosystem},
};

// trait Event {
//     // performs and propagates the event until it is finished
//     fn apply_event(self, ecosystem: &mut Ecosystem, index: CellIndex);
// }

#[derive(PartialEq, Debug)]
pub(crate) enum Events {
    Rainfall,
    ThermalStress,
    Lightning,
    RockSlide,
    SandSlide,
    HumusSlide,
    Fire,
    Vegetation,
}

impl Events {
    pub fn apply_event(self, ecosystem: &mut Ecosystem, index: CellIndex) {
        let mut event_option = Some((self, index));
        while let Some((event, index)) = event_option {
            event_option = match event {
                Events::Rainfall => todo!(),
                Events::ThermalStress => todo!(),
                Events::Lightning => Self::apply_lightning_event(ecosystem, index),
                Events::RockSlide => todo!(),
                Events::SandSlide => Self::apply_sand_slide_event(ecosystem, index),
                Events::HumusSlide => todo!(),
                Events::Fire => todo!(),
                Events::Vegetation => todo!(),
            };
        }
    }

    pub(crate) fn apply_lightning_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let strike_probability = Self::compute_probability_of_lightning_damage(ecosystem, index);
        Self::apply_lightning_event_helper(ecosystem, index, strike_probability)
    }

    fn apply_lightning_event_helper(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
        strike_probability: f32,
    ) -> Option<(Events, CellIndex)> {
        let mut rng = rand::thread_rng();
        let rand: f32 = rng.gen();
        if rand > strike_probability {
            // no lightning strike
            return None;
        }

        let cell = &mut ecosystem[index];

        // kill all vegetation in the cell
        Self::kill_trees(cell);
        Self::kill_bushes(cell);
        Self::kill_grasses(cell);

        // destroy some bedrock and scatter as rocks and sand to nearby cells
        let bedrock = &mut cell.bedrock.as_mut().unwrap();
        let lost_height = constants::LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME
            / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);
        bedrock.height -= lost_height;

        // simplifying assumption 1: half of the volume becomes rock and the other half sand
        // simplifying assumption 2: distribute volume evenly to 8 neighbors and cell (instead of being based on slope and relative elevation)
        let neighbors = Cell::get_neighbors(&index);
        let num_affected_cells = neighbors.len() + 1;
        let volume_per_cell =
            constants::LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME / num_affected_cells as f32;
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

        // does not propagate
        None
    }

    fn compute_probability_of_lightning_damage(ecosystem: &Ecosystem, index: CellIndex) -> f32 {
        //l(p)=k_L min(1,e^(k_lc * (∇E(p)−k_ls))
        // k_L is maximum probability
        // k_lc is scaling factor
        // k_ls is minimum curvature required
        let curvature = ecosystem.estimate_curvature(index);
        println!("index {index}, curvature {curvature}");

        let max_prob = 1.0;
        let scaling_factor = 1.0;
        let min_curve = 4.0;
        let exp = scaling_factor * ((-curvature) - min_curve);
        println!("exp {exp}");
        let prob = max_prob * f32::min(1.0, (E as f32).powf(exp));
        println!("prob {prob}");

        prob
    }

    pub(crate) fn apply_sand_slide_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let mut critical_neighbors: HashMap<CellIndex, f32> = HashMap::new();
        let neighbors = Cell::get_neighbors(&index);
        for neighbor_index in neighbors.as_array().into_iter().flatten() {
            let slope = ecosystem.get_slope_between_points(index, neighbor_index);
            let angle = Ecosystem::get_angle(slope);
            if angle >= constants::CRITICAL_ANGLE_SAND {
                critical_neighbors.insert(neighbor_index, slope);
            }
        }
        // if current cell does not have a slope of at least the critical angle, no slide and no propagation
        if critical_neighbors.is_empty() {
            return None;
        } else {
            // else randomly select neighbor weighted by slope
            let mut neighbor_probabilities: HashMap<CellIndex, f32> = HashMap::new();
            let slope_sum: f32 = critical_neighbors.values().sum();
            for (neighbor, slope) in critical_neighbors {
                let prob = slope / slope_sum;
                neighbor_probabilities.insert(neighbor, prob);
            }
            let mut rng = rand::thread_rng();
            let mut rand: f32 = rng.gen();
            for (neighbor, prob) in neighbor_probabilities {
                rand -= prob;
                if rand < 0.0 {
                    // to propagate, reduce appropriate amount of sand and move it to neighbor
                    let sand_height =
                        Events::compute_sand_height_to_slide(ecosystem, index, neighbor);
                    // println!("Sand of height {sand_height} sliding from {index} to {neighbor}");
                    let cell = &mut ecosystem[index];
                    cell.remove_sand(sand_height);

                    let neighbor_cell = &mut ecosystem[neighbor];
                    neighbor_cell.add_sand(sand_height);

                    return Some((Events::SandSlide, neighbor));
                }
            }
        }

        None
    }
    fn compute_ideal_slide_height(
        pos_1: Vector3<f32>,
        pos_2: Vector3<f32>,
        critical_angle: f32,
    ) -> f32 {
        let critical_slope = f32::sin(critical_angle.to_radians());
        let k = (critical_slope
            * critical_slope
            * (f32::powf(pos_1.x - pos_2.x, 2.0) + f32::powf(pos_1.y - pos_2.y, 2.0)))
            / (1.0 - critical_slope * critical_slope);
        pos_2.z + f32::sqrt(k)
    }

    fn compute_sand_height_to_slide(
        ecosystem: &Ecosystem,
        origin: CellIndex,
        target: CellIndex,
    ) -> f32 {
        let cell = &ecosystem[origin];
        if let Some(sand) = &cell.sand {
            let sand_height = sand.height;
            let origin_pos = ecosystem.get_position_of_cell(&origin);
            let target_pos = ecosystem.get_position_of_cell(&target);
            let ideal_height = Events::compute_ideal_slide_height(
                origin_pos,
                target_pos,
                constants::CRITICAL_ANGLE_SAND,
            );

            let non_sand_height = cell.get_height() - sand_height;

            // simplifying assumption: half of the excess slides away
            if non_sand_height >= ideal_height {
                sand_height / 2.0
            } else {
                ((non_sand_height + sand_height) - ideal_height) / 2.0
            }
        } else {
            0.0
        }
    }
}

impl Events {
    // converts all trees in a cell into dead vegetation
    fn kill_trees(cell: &mut Cell) {
        if let Some(trees) = &mut cell.trees {
            let biomass = trees.estimate_biomass();
            trees.number_of_plants = 0;
            trees.plant_height_sum = 0.0;
            trees.plant_age_sum = 0.0;
            cell.add_dead_vegetation(biomass);
        }
    }

    // converts all bushes in a cell into dead vegetation
    fn kill_bushes(cell: &mut Cell) {
        if let Some(bushes) = &mut cell.bushes {
            let biomass = bushes.estimate_biomass();
            bushes.number_of_plants = 0;
            bushes.plant_height_sum = 0.0;
            bushes.plant_age_sum = 0.0;
            cell.add_dead_vegetation(biomass);
        }
    }

    // converts all grasses in a cell into dead vegetation
    fn kill_grasses(cell: &mut Cell) {
        if let Some(grasses) = &mut cell.grasses {
            let coverage_density = grasses.coverage_density;
            cell.add_dead_vegetation(
                coverage_density * CELL_SIDE_LENGTH * CELL_SIDE_LENGTH * constants::GRASS_DENSITY,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use nalgebra::Vector3;

    use crate::{
        constants,
        ecology::{Cell, CellIndex, CellLayer, Ecosystem, Trees},
        events::Events,
    };

    #[test]
    fn kill_trees() {
        let trees = Trees {
            number_of_plants: 1,
            plant_height_sum: 30.0,
            plant_age_sum: 10.0,
        };
        let mut cell = Cell {
            soil_moisture: 0.0,
            sunlight: 0.0,
            temperature: 0.0,
            bedrock: None,
            rock: None,
            sand: None,
            humus: None,
            trees: Some(trees),
            bushes: None,
            grasses: None,
            dead_vegetation: None,
        };
        let biomass = cell.estimate_tree_biomass();

        Events::kill_trees(&mut cell);

        let trees = &cell.trees;
        assert!(trees.is_some());
        let trees = trees.as_ref().unwrap();
        assert!(trees.number_of_plants == 0);
        assert!(trees.plant_age_sum == 0.0);
        assert!(trees.plant_height_sum == 0.0);

        let dead_vegetation = &cell.dead_vegetation;
        assert!(dead_vegetation.is_some());
        let dead_vegetation = &dead_vegetation.as_ref().unwrap();
        let expected = dead_vegetation.biomass;
        let actual = biomass;
        assert!(expected == actual, "Expected {expected}, actual {actual}");

        // add more trees and kill them
        let trees = &mut cell.trees.as_mut().unwrap();
        trees.number_of_plants = 5;
        trees.plant_height_sum = 150.0;
        let biomass_2 = cell.estimate_tree_biomass();

        Events::kill_trees(&mut cell);

        let trees = &mut cell.trees;
        assert!(trees.is_some());
        let trees = trees.as_ref().unwrap();
        assert!(trees.number_of_plants == 0);
        assert!(trees.plant_age_sum == 0.0);
        assert!(trees.plant_height_sum == 0.0);

        let dead_vegetation = cell.dead_vegetation;
        assert!(dead_vegetation.is_some());
        let dead_vegetation = dead_vegetation.unwrap();
        let expected = dead_vegetation.biomass;
        let actual = biomass + biomass_2;
        assert!(expected == actual, "Expected {expected}, actual {actual}");
    }

    #[test]
    fn test_lightning_event() {
        let index = CellIndex::new(5, 5);
        test_lightning_event_helper(index);
        let index = CellIndex::new(0, 0);
        test_lightning_event_helper(index);
        let index = CellIndex::new(95, 0);
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
        assert!(trees.is_some());
        let trees = trees.as_ref().unwrap();
        assert!(trees.number_of_plants == 0);
        assert!(trees.plant_age_sum == 0.0);
        assert!(trees.plant_height_sum == 0.0);

        // assert bedrock is decreased
        assert!(cell.bedrock.is_some());
        let expected_height = constants::DEFAULT_BEDROCK_HEIGHT
            - constants::LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME
                / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);
        let actual_height = cell.bedrock.as_ref().unwrap().height;
        assert!(
            actual_height == expected_height,
            "Expected {expected_height}, actual {actual_height}"
        );

        // assert neighbors and self have increase in rocks and sand
        let neighbors = Cell::get_neighbors(&index);
        let num_neighbors = neighbors.len() + 1;
        let volume_per_cell =
            constants::LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME / num_neighbors as f32;
        let height_per_cell =
            volume_per_cell / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);
        let rock_layer = &cell.rock;
        assert!(rock_layer.is_some());
        assert!(rock_layer.as_ref().unwrap().height == height_per_cell);
        let sand_layer = &cell.sand;
        assert!(sand_layer.is_some());
        assert!(sand_layer.as_ref().unwrap().height == height_per_cell);
        for neighbor_index in neighbors.as_array().into_iter().flatten() {
            let neighbor = &ecosystem[neighbor_index];
            let rock_layer = &neighbor.rock;
            assert!(rock_layer.is_some());
            assert!(rock_layer.as_ref().unwrap().height == height_per_cell);
            let sand_layer = &neighbor.sand;
            assert!(sand_layer.is_some());
            assert!(sand_layer.as_ref().unwrap().height == height_per_cell);
        }
    }

    #[test]
    fn test_compute_max_slide_height() {
        let pos_1 = Vector3::new(0.0, 0.0, 1.0);
        let pos_2 = Vector3::new(0.0, 1.0, 0.0);
        let critical_angle = 34.0;
        let new_height = Events::compute_ideal_slide_height(pos_1, pos_2, critical_angle);
        let expected = 0.676;
        assert!(
            approx_eq!(f32, new_height, expected, epsilon = 0.01),
            "Expected {expected}, actual {new_height}"
        );
    }

    #[test]
    fn test_apply_sand_slide_event() {
        let mut ecosystem = Ecosystem::init();
        let center = &mut ecosystem[CellIndex::new(3, 3)];
        let bedrock = &mut center.bedrock.as_mut().unwrap();
        bedrock.height = 0.0;
        center.add_sand(1.0);

        let up = &mut ecosystem[CellIndex::new(3, 2)];
        let bedrock = &mut up.bedrock.as_mut().unwrap();
        bedrock.height = 0.0;

        let propagation = Events::apply_sand_slide_event(&mut ecosystem, CellIndex::new(3, 3));

        assert!(propagation.is_some());
        let (event, index) = propagation.unwrap();
        assert_eq!(event, Events::SandSlide);
        assert_eq!(index, CellIndex::new(3, 2));

        let center = &mut ecosystem[CellIndex::new(3, 3)];
        let sand_height = center.sand.as_ref().unwrap().height;
        let expected = 0.838;
        assert!(
            approx_eq!(f32, sand_height, expected, epsilon = 0.01),
            "Expected {expected}, actual {sand_height}"
        );

        let up = &mut ecosystem[CellIndex::new(3, 2)];
        let sand_height = up.sand.as_ref().unwrap().height;
        let expected = 0.162;
        assert!(
            approx_eq!(f32, sand_height, expected, epsilon = 0.01),
            "Expected {expected}, actual {sand_height}"
        );
    }
}

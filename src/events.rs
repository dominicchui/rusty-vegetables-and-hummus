use rand::Rng;

use crate::{
    constants::{self, CELL_SIDE_LENGTH},
    ecology::{Cell, CellIndex, Ecosystem},
};

trait Event {
    // returns the event to propagate and next cell index to propagate to
    fn apply_event_and_propagate(
        &self,
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)>;
}

enum Events {
    Rainfall,
    ThermalStress,
    Lightning,
    RockSlide,
    SandSlide,
    HumusSlide,
    Fire,
    Vegetation,
}

impl Event for Events {
    fn apply_event_and_propagate(
        &self,
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        match self {
            Events::Rainfall => todo!(),
            Events::ThermalStress => todo!(),
            Events::Lightning => self.apply_and_propagate_lightning_event(ecosystem, index),
            Events::RockSlide => todo!(),
            Events::SandSlide => todo!(),
            Events::HumusSlide => todo!(),
            Events::Fire => todo!(),
            Events::Vegetation => todo!(),
        }
    }
}

impl Events {
    fn apply_and_propagate_lightning_event(
        &self,
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let strike_probability = Self::compute_probability_of_lightning_damage(ecosystem, &index);
        self.apply_and_propagate_lightning_event_helper(ecosystem, index, strike_probability)
    }

    fn apply_and_propagate_lightning_event_helper(
        &self,
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
        let bedrock = cell.get_bedrock_layer_mut().unwrap();
        bedrock.height -= constants::LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME
            / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);

        // simplifying assumption 1: half of the volume becomes rock and the other half sand
        // simplifying assumption 2: distribute volume evenly to 8 neighbors and self (instead of being based on slope and relative elevation)
        let neighbors = Cell::get_neighbors(&index);
        let num_neighbors = neighbors.len() + 1;
        let volume_per_cell =
            constants::LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME / num_neighbors as f32;
        let height_per_cell =
            volume_per_cell / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);

        // add to cell
        Self::add_rocks_and_sand(cell, height_per_cell);

        // add to neighbors
        for index in neighbors.as_array().into_iter().flatten() {
            // if let Some(index) = index {
            let neighbor = &mut ecosystem[index];
            Self::add_rocks_and_sand(neighbor, height_per_cell);
            // }
        }

        // does not propagate
        None
    }

    fn add_rocks_and_sand(cell: &mut Cell, height: f32) {
        let rock_layer = cell.get_rock_layer_mut();
        if let Some(rocks) = rock_layer {
            rocks.height += height;
        } else {
            cell.insert_rocks(height);
        }
        let sand_layer = cell.get_sand_layer_mut();
        if let Some(sand) = sand_layer {
            sand.height += height;
        } else {
            cell.insert_sand(height);
        }
    }

    fn compute_probability_of_lightning_damage(ecosystem: &Ecosystem, index: &CellIndex) -> f32 {
        //l(p)=k_L min(1,e^(k_lc (∇E(p)−k_ls))
        // k_L is maximum probability
        // k_lc is scaling factor
        // k_ls is minimum curvature required
        0.001
    }
}

impl Events {
    // converts all trees in a cell into dead vegetation
    fn kill_trees(cell: &mut Cell) {
        if let Some(trees) = cell.get_trees_layer_mut() {
            let biomass = trees.estimate_biomass();
            trees.number_of_plants = 0;
            trees.plant_height_sum = 0.0;
            trees.plant_age_sum = 0.0;
            cell.add_dead_vegetation(biomass);
        }
    }

    // converts all bushes in a cell into dead vegetation
    fn kill_bushes(cell: &mut Cell) {
        if let Some(bushes) = cell.get_bushes_layer_mut() {
            let biomass = bushes.estimate_biomass();
            bushes.number_of_plants = 0;
            bushes.plant_height_sum = 0.0;
            bushes.plant_age_sum = 0.0;
            cell.add_dead_vegetation(biomass);
        }
    }

    // converts all grasses in a cell into dead vegetation
    fn kill_grasses(cell: &mut Cell) {
        if let Some(grasses) = cell.get_grasses_layer_mut() {
            let coverage_density = grasses.coverage_density;
            cell.add_dead_vegetation(
                coverage_density * CELL_SIDE_LENGTH * CELL_SIDE_LENGTH * constants::GRASS_DENSITY,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        constants,
        ecology::{Cell, CellIndex, CellLayer, Ecosystem, Trees},
        events::Events,
    };

    #[test]
    fn kill_trees() {
        let layer = CellLayer::Trees(Trees {
            number_of_plants: 1,
            plant_height_sum: 30.0,
            plant_age_sum: 10.0,
        });
        let mut cell = Cell {
            layers: vec![layer],
            soil_moisture: 0.0,
            sunlight: 0.0,
            temperature: 0.0,
        };
        let biomass = cell.estimate_tree_biomass();

        Events::kill_trees(&mut cell);

        let trees = cell.get_trees_layer();
        assert!(trees.is_some());
        let trees = trees.unwrap();
        assert!(trees.number_of_plants == 0);
        assert!(trees.plant_age_sum == 0.0);
        assert!(trees.plant_height_sum == 0.0);

        let dead_vegetation = cell.get_dead_vegetation_layer();
        assert!(dead_vegetation.is_some());
        let dead_vegetation = dead_vegetation.unwrap();
        assert!(dead_vegetation.biomass == biomass);

        // add more trees and kill them
        let trees = cell.get_trees_layer_mut().unwrap();
        trees.number_of_plants = 5;
        trees.plant_height_sum = 150.0;
        let biomass_2 = cell.estimate_tree_biomass();

        Events::kill_trees(&mut cell);

        let trees = cell.get_trees_layer();
        assert!(trees.is_some());
        let trees = trees.unwrap();
        assert!(trees.number_of_plants == 0);
        assert!(trees.plant_age_sum == 0.0);
        assert!(trees.plant_height_sum == 0.0);

        let dead_vegetation = cell.get_dead_vegetation_layer();
        assert!(dead_vegetation.is_some());
        let dead_vegetation = dead_vegetation.unwrap();
        assert!(dead_vegetation.biomass == biomass + biomass_2);
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
        let trees = CellLayer::Trees(Trees {
            number_of_plants: 1,
            plant_height_sum: 30.0,
            plant_age_sum: 10.0,
        });
        let cell = &mut ecosystem[index];
        cell.layers.push(trees);

        let result = Events::Lightning.apply_and_propagate_lightning_event_helper(
            &mut ecosystem,
            index,
            1.0,
        );
        assert!(result.is_none());

        // verify trees are dead
        let cell = &ecosystem[index];
        let trees = cell.get_trees_layer();
        assert!(trees.is_some());
        let trees = trees.unwrap();
        assert!(trees.number_of_plants == 0);
        assert!(trees.plant_age_sum == 0.0);
        assert!(trees.plant_height_sum == 0.0);

        // assert bedrock is decreased
        assert!(cell.get_bedrock_layer().is_some());
        let expected_height = constants::DEFAULT_BEDROCK_HEIGHT
            - constants::LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME
                / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH);
        let actual_height = cell.get_bedrock_layer().unwrap().height;
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
        let rock_layer = cell.get_rock_layer();
        assert!(rock_layer.is_some());
        assert!(rock_layer.unwrap().height == height_per_cell);
        let sand_layer = cell.get_sand_layer();
        assert!(sand_layer.is_some());
        assert!(sand_layer.unwrap().height == height_per_cell);
        for neighbor_index in neighbors.as_array().into_iter().flatten() {
            let neighbor = &ecosystem[neighbor_index];
            let rock_layer = neighbor.get_rock_layer();
            assert!(rock_layer.is_some());
            assert!(rock_layer.unwrap().height == height_per_cell);
            let sand_layer = neighbor.get_sand_layer();
            assert!(sand_layer.is_some());
            assert!(sand_layer.unwrap().height == height_per_cell);
        }
    }
}

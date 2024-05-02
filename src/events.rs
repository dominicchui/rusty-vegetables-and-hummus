mod humus_slide;
mod lightning;
mod rock_slide;
mod sand_slide;
mod thermal_stress;
mod rainfall;

use nalgebra::Vector3;

use crate::{
    constants::{self, CELL_SIDE_LENGTH},
    ecology::{Cell, CellIndex, Ecosystem},
};

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
    // performs and propagates the event until it is finished
    pub fn apply_event(self, ecosystem: &mut Ecosystem, index: CellIndex) {
        let mut event_option = Some((self, index));
        while let Some((event, index)) = event_option {
            event_option = match event {
                Events::Rainfall => Self::apply_rainfall_event(ecosystem, index),
                Events::ThermalStress => Self::apply_thermal_stress_event(ecosystem, index),
                Events::Lightning => Self::apply_lightning_event(ecosystem, index),
                Events::RockSlide => Self::apply_rock_slide_event(ecosystem, index),
                Events::SandSlide => Self::apply_sand_slide_event(ecosystem, index),
                Events::HumusSlide => Self::apply_humus_slide_event(ecosystem, index),
                Events::Fire => todo!(),
                Events::Vegetation => todo!(),
            };
        }
    }

    // given the critical angle, compute the ideal height of material to slide from pos_1 to pos_2
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

    // converts all trees in a cell into dead vegetation
    fn kill_trees(cell: &mut Cell) {
        if let Some(trees) = &mut cell.trees {
            let biomass = trees.estimate_biomass();
            trees.number_of_plants = 0;
            trees.plant_height_sum = 0.0;
            trees.plant_age_sum = 0.0;
            cell.add_dead_vegetation(biomass);
            cell.trees = None;
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
            cell.bushes = None;
        }
    }

    // converts all grasses in a cell into dead vegetation
    fn kill_grasses(cell: &mut Cell) {
        if let Some(grasses) = &mut cell.grasses {
            let coverage_density = grasses.coverage_density;
            cell.add_dead_vegetation(
                coverage_density * CELL_SIDE_LENGTH * CELL_SIDE_LENGTH * constants::GRASS_DENSITY,
            );
            cell.grasses = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use nalgebra::Vector3;

    use crate::{
        constants,
        ecology::{Cell, CellIndex, Ecosystem, Trees},
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
        assert!(trees.is_none());

        let dead_vegetation = &cell.dead_vegetation;
        assert!(dead_vegetation.is_some());
        let dead_vegetation = &dead_vegetation.as_ref().unwrap();
        let expected = dead_vegetation.biomass;
        let actual = biomass;
        assert!(expected == actual, "Expected {expected}, actual {actual}");

        // add more trees and kill them
        let trees = Trees {
            number_of_plants: 5,
            plant_height_sum: 150.0,
            plant_age_sum: 10.0,
        };
        cell.trees = Some(trees);
        let biomass_2 = cell.estimate_tree_biomass();

        Events::kill_trees(&mut cell);

        let trees = &mut cell.trees;
        assert!(trees.is_none());

        let dead_vegetation = cell.dead_vegetation;
        assert!(dead_vegetation.is_some());
        let dead_vegetation = dead_vegetation.unwrap();
        let expected = dead_vegetation.biomass;
        let actual = biomass + biomass_2;
        assert!(expected == actual, "Expected {expected}, actual {actual}");
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
}

// a constant to control the probability of a thermal stress event
// higher is more likely
const FRACTURE_CONSTANT: f32 = 1.0;
// how much sand and humus dampen the probability of a thermal stress event
const GRANULAR_DAMPENING_CONSTANT: f32 = 1.0;
// how much vegetation density dampens the probability of a thermal stress event
const VEGETATION_DAMPENING_CONSTANT: f32 = 1.0;
// amount of bedrock fractured into rock per successful event
const BEDROCK_FRACTURE_HEIGHT: f32 = 1.0;

use rand::Rng;

use super::Events;
use crate::{
    constants,
    ecology::{Cell, CellIndex, Ecosystem},
};

impl Events {
    pub(crate) fn apply_thermal_stress_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let fracture_probability = Self::compute_thermal_fracture_probability(ecosystem, index);
        println!("fracture_probability {fracture_probability}");
        let mut rng = rand::thread_rng();
        let rand: f32 = rng.gen();

        if rand < fracture_probability {
            // fracture some bedrock and convert to rocks
            let cell = &mut ecosystem[index];
            cell.remove_bedrock(BEDROCK_FRACTURE_HEIGHT);
            cell.add_rocks(BEDROCK_FRACTURE_HEIGHT);
        }

        None
    }

    fn compute_thermal_fracture_probability(ecosystem: &Ecosystem, index: CellIndex) -> f32 {
        // simplifying assumption: day/night temperature difference is 10°C (todo improve based on elevation and illumination)
        let delta_t = 10.0;

        // probability bedrock B will fracture into rocks R
        // dampen Δt with vegetation density V(p), and sand + humus height G(p)
        // k, kG, and kV are constants
        // s(p) is maximum local slope
        // f(p) = k * ∆T * s(p) / (1 + kG * G(p) + kV * V(p))

        let mut max_slope = 0.0;
        let neighbors = Cell::get_neighbors(&index);
        for neighbor_index in neighbors.as_array().into_iter().flatten() {
            let slope = f32::abs(ecosystem.get_slope_between_points(index, neighbor_index));
            if slope > max_slope {
                max_slope = slope;
            }
        }
        let cell = &ecosystem[index];
        let vegetation_density = cell.estimate_vegetation_density();
        let granular_height = cell.get_height_of_sand() + cell.get_height_of_humus();
        println!("vegetation_density {vegetation_density}");
        println!("granular_height {granular_height}");
        FRACTURE_CONSTANT * delta_t * max_slope
            / (1.0
                + GRANULAR_DAMPENING_CONSTANT * granular_height
                + VEGETATION_DAMPENING_CONSTANT * vegetation_density)
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use nalgebra::Vector3;

    use crate::{
        constants,
        ecology::{Cell, CellIndex, Ecosystem, Trees},
        events::{thermal_stress::GRANULAR_DAMPENING_CONSTANT, Events},
    };

    #[test]
    fn test_compute_thermal_fracture_probability() {
        // flat terrain should have 0 probability
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(2, 2);
        let prob = Events::compute_thermal_fracture_probability(&ecosystem, index);
        assert_eq!(prob, 0.0);

        // slightly raise the cell to create a hill and a slope
        let cell = &mut ecosystem[index];
        cell.bedrock.as_mut().unwrap().height = 101.0;

        let prob = Events::compute_thermal_fracture_probability(&ecosystem, index);
        let expected = 7.07;
        assert!(
            approx_eq!(f32, prob, expected, epsilon = 0.01),
            "Expected {expected}, actual {prob}"
        );

        // set the hill to be a neighboring cell instead
        let cell = &mut ecosystem[index];
        cell.bedrock.as_mut().unwrap().height = 100.0;

        let cell = &mut ecosystem[CellIndex::new(2, 1)];
        cell.bedrock.as_mut().unwrap().height = 101.0;

        let prob = Events::compute_thermal_fracture_probability(&ecosystem, index);
        let expected = 7.07;
        assert!(
            approx_eq!(f32, prob, expected, epsilon = 0.01),
            "Expected {expected}, actual {prob}"
        );

        // add some sand and humus
        let cell = &mut ecosystem[CellIndex::new(2, 2)];
        cell.bedrock.as_mut().unwrap().height = 98.0;
        cell.add_sand(1.0);
        cell.add_humus(1.0);

        let prob = Events::compute_thermal_fracture_probability(&ecosystem, index);
        let expected = 7.07 / (1.0 + GRANULAR_DAMPENING_CONSTANT * 2.0);
        assert!(
            approx_eq!(f32, prob, expected, epsilon = 0.01),
            "Expected {expected}, actual {prob}"
        );
    }
}

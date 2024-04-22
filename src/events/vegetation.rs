use itertools::Itertools;

use super::Events;
use crate::{
    constants,
    ecology::{Bushes, Cell, CellIndex, Ecosystem, Grasses, Trees},
};

// viability constants for vegetation
trait HasViability {
    // temperature in celsius
    const TEMPERATURE_LIMIT_MIN: f32;
    const TEMPERATURE_LIMIT_MAX: f32;
    const TEMPERATURE_IDEAL_MIN: f32;
    const TEMPERATURE_IDEAL_MAX: f32;
    // % soil moisture, which is the % by weight or volume of soil
    // e.g. 10% moisture means 10% volume (or weight) of soil is water
    const MOISTURE_LIMIT_MIN: f32;
    const MOISTURE_LIMIT_MAX: f32;
    const MOISTURE_IDEAL_MIN: f32;
    const MOISTURE_IDEAL_MAX: f32;
    // hours of daily sunlight
    const ILLUMINATION_LIMIT_MIN: f32;
    const ILLUMINATION_LIMIT_MAX: f32;
    const ILLUMINATION_IDEAL_MIN: f32;
    const ILLUMINATION_IDEAL_MAX: f32;
}

impl HasViability for Trees {
    // source: https://www.picturethisai.com/care/temperature/Acer_rubrum.html
    const TEMPERATURE_LIMIT_MIN: f32 = -10.0;
    const TEMPERATURE_IDEAL_MIN: f32 = 0.0;
    const TEMPERATURE_IDEAL_MAX: f32 = 35.0;
    const TEMPERATURE_LIMIT_MAX: f32 = 38.0;

    // sources:
    // https://www.acurite.com/blog/soil-moisture-guide-for-plants-and-vegetables.html
    // https://www.nature.com/articles/s41598-021-01804-3#Sec2
    // https://www.srs.fs.usda.gov/pubs/misc/ag_654/volume_2/acer/rubrum.htm
    const MOISTURE_LIMIT_MIN: f32 = 0.1;
    const MOISTURE_IDEAL_MIN: f32 = 0.2;
    const MOISTURE_IDEAL_MAX: f32 = 0.4;
    const MOISTURE_LIMIT_MAX: f32 = 0.8;

    // very rough estimates since numbers are hard to find
    const ILLUMINATION_LIMIT_MIN: f32 = 4.0;
    const ILLUMINATION_IDEAL_MIN: f32 = 6.0;
    const ILLUMINATION_IDEAL_MAX: f32 = 8.0;
    const ILLUMINATION_LIMIT_MAX: f32 = 12.0;
}

impl HasViability for Bushes {
    const TEMPERATURE_LIMIT_MIN: f32 = 0.0;
    const TEMPERATURE_IDEAL_MIN: f32 = 0.0;
    const TEMPERATURE_IDEAL_MAX: f32 = 0.0;
    const TEMPERATURE_LIMIT_MAX: f32 = 0.0;

    // sources:
    // https://www.acurite.com/blog/soil-moisture-guide-for-plants-and-vegetables.html
    const MOISTURE_LIMIT_MIN: f32 = 0.0;
    const MOISTURE_IDEAL_MIN: f32 = 40.0;
    const MOISTURE_IDEAL_MAX: f32 = 60.0;
    const MOISTURE_LIMIT_MAX: f32 = 0.0;

    const ILLUMINATION_LIMIT_MIN: f32 = 0.0;
    const ILLUMINATION_IDEAL_MIN: f32 = 0.0;
    const ILLUMINATION_IDEAL_MAX: f32 = 0.0;
    const ILLUMINATION_LIMIT_MAX: f32 = 0.0;
}

impl HasViability for Grasses {
    const TEMPERATURE_LIMIT_MIN: f32 = 0.0;
    const TEMPERATURE_IDEAL_MAX: f32 = 0.0;
    const TEMPERATURE_LIMIT_MAX: f32 = 0.0;
    const TEMPERATURE_IDEAL_MIN: f32 = 0.0;

    const MOISTURE_LIMIT_MIN: f32 = 0.0;
    const MOISTURE_IDEAL_MIN: f32 = 0.0;
    const MOISTURE_IDEAL_MAX: f32 = 0.0;
    const MOISTURE_LIMIT_MAX: f32 = 0.0;

    const ILLUMINATION_LIMIT_MIN: f32 = 0.0;
    const ILLUMINATION_IDEAL_MIN: f32 = 0.0;
    const ILLUMINATION_IDEAL_MAX: f32 = 0.0;
    const ILLUMINATION_LIMIT_MAX: f32 = 0.0;
}

impl Events {
    pub(crate) fn apply_vegetation_trees_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let cell = &ecosystem[index];
        if let Some(trees) = &cell.trees {
            let (vigor, stress) = Self::compute_vigor_and_stress(ecosystem, index, trees);
        }

        // does not propagate
        None
    }

    // returns tuple of vigor and stress
    // vigor is average viability during growing season (T > 5°C)
    // stress is average of 4 worst negative viabilities
    fn compute_vigor_and_stress<T: HasViability>(
        ecosystem: &Ecosystem,
        index: CellIndex,
        vegetation: &T,
    ) -> (f32, f32) {
        let mut viabilities = [0.0; 12];
        let mut growing_viabilities = vec![];
        for (i, value) in viabilities.iter_mut().enumerate() {
            let viability = Self::compute_viability(ecosystem, index, vegetation, i);
            *value = viability;
            if constants::AVERAGE_MONTHLY_TEMPERATURES[i] > 5.0 {
                growing_viabilities.push(viability);
            }
        }

        // vigor is average viability during growing season (T > 5°C)
        let vigor = growing_viabilities.into_iter().sum::<f32>() / 12.0;

        // stress is average of 4 worst negative viabilities
        let mut negative_viabilities = viabilities.into_iter().filter(|v| *v < 0.0).collect_vec();
        negative_viabilities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let stress =
            negative_viabilities.iter().take(4).sum::<f32>() / negative_viabilities.len() as f32;
        (vigor, stress)
    }

    // returns viability for a given plant for a given month
    fn compute_viability<T: HasViability>(
        ecosystem: &Ecosystem,
        index: CellIndex,
        vegetation: &T,
        month: usize,
    ) -> f32 {
        // determines viability from piecewise function evaluating all three of temperature, moisture, and sunlight
        let temperature_viability =
            Self::compute_temperature_viability(ecosystem, index, vegetation, month);
        let moisture_viability =
            Self::compute_moisture_viability(ecosystem, index, vegetation, month);
        let illumination_viability =
            Self::compute_illumination_viability(ecosystem, index, vegetation, month);

        // viability is lowest of the the sub-values (Leibig’s law of the minimum)
        f32::min(
            temperature_viability,
            f32::min(moisture_viability, illumination_viability),
        )
    }

    fn compute_temperature_viability<T: HasViability>(
        ecosystem: &Ecosystem,
        index: CellIndex,
        _: &T,
        month: usize,
    ) -> f32 {
        let cell = &ecosystem[index];
        let temperature = cell.get_monthly_temperature(month);
        match temperature {
            temperature if temperature < T::TEMPERATURE_LIMIT_MIN => -1.0,
            temperature if temperature < T::TEMPERATURE_IDEAL_MIN => {
                (temperature - T::TEMPERATURE_LIMIT_MIN)
                    / (T::TEMPERATURE_IDEAL_MIN - T::TEMPERATURE_LIMIT_MIN)
            }
            temperature if temperature <= T::TEMPERATURE_IDEAL_MAX => 1.0,
            _ => -1.0,
        }
    }

    fn compute_moisture_viability<T: HasViability>(
        ecosystem: &Ecosystem,
        index: CellIndex,
        _: &T,
        month: usize,
    ) -> f32 {
        let cell = &ecosystem[index];
        // convert moisture in terms of volume to % by volume
        let moisture_volume = cell.get_monthly_soil_moisture(month); // in L
        println!("moisture_volume {moisture_volume}");
        // bedrock, rock, sand, and humus can all hold water, but make simplifying assumption that all water makes it to humus layer
        // so each cell is 10x10xheight m, where height is height of humus
        // 1 cubic meter = 1000 liters
        let height = cell.get_humus_height();
        println!("height {height}");
        let cell_volume =
            constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH * height * 1000.0; // in L
        println!("cell_volume {cell_volume}");
        let moisture = moisture_volume / cell_volume;
        println!("moisture {moisture}");

        match moisture {
            moisture if moisture < T::MOISTURE_LIMIT_MIN => -1.0,
            moisture if moisture < T::MOISTURE_IDEAL_MIN => {
                (moisture - T::MOISTURE_LIMIT_MIN) / (T::MOISTURE_IDEAL_MIN - T::MOISTURE_LIMIT_MIN)
            }
            moisture if moisture <= T::MOISTURE_IDEAL_MAX => 1.0,
            moisture if moisture <= T::MOISTURE_LIMIT_MAX => {
                (moisture - T::MOISTURE_IDEAL_MAX) / (T::MOISTURE_LIMIT_MAX - T::MOISTURE_IDEAL_MAX)
            }
            _ => -1.0,
        }
    }

    fn compute_illumination_viability<T: HasViability>(
        ecosystem: &Ecosystem,
        index: CellIndex,
        _: &T,
        month: usize,
    ) -> f32 {
        let illumination = ecosystem.estimate_illumination(&index, month);
        match illumination {
            illumination if illumination < T::ILLUMINATION_LIMIT_MIN => -1.0,
            illumination if illumination < T::ILLUMINATION_IDEAL_MIN => {
                (illumination - T::ILLUMINATION_LIMIT_MIN)
                    / (T::ILLUMINATION_IDEAL_MIN - T::ILLUMINATION_LIMIT_MIN)
            }
            illumination if illumination <= T::ILLUMINATION_IDEAL_MAX => 1.0,
            _ => -1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;

    use crate::{
        constants,
        ecology::{Bushes, Cell, CellIndex, Ecosystem, Grasses, Trees},
        events::Events,
    };

    #[test]
    fn test_tree_compute_viability() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(2, 2);
        let trees = Trees {
            number_of_plants: 1,
            plant_height_sum: 10.0,
            plant_age_sum: 10.0,
        };
        let cell = &mut ecosystem[index];
        cell.trees = Some(trees.clone());
        // 50 cm of humus/soil
        cell.remove_bedrock(0.5);
        cell.add_humus(0.5);

        // January
        let temperature_viability =
            Events::compute_temperature_viability(&ecosystem, index, &trees, 0);
        assert_eq!(temperature_viability, 0.735);
        let moisture_viability = Events::compute_moisture_viability(&ecosystem, index, &trees, 0);
        assert_eq!(moisture_viability, -1.0);
        let illumination_viability =
            Events::compute_illumination_viability(&ecosystem, index, &trees, 0);
        assert_eq!(illumination_viability, 1.0);

        // viability is min of the sub-components
        let viability = Events::compute_viability(&ecosystem, index, &trees, 0);
        assert_eq!(viability, -1.0);

        // boost moisture content to within ideal range
        let cell = &mut ecosystem[index];
        cell.soil_moisture = 1.8E5;
        let moisture_viability = Events::compute_moisture_viability(&ecosystem, index, &trees, 0);
        assert_eq!(moisture_viability, 1.0);
        let viability = Events::compute_viability(&ecosystem, index, &trees, 0);
        assert_eq!(viability, 0.735);

        // remove some humus, which will boost soil moisture
        let cell = &mut ecosystem[index];
        cell.add_bedrock(0.2);
        cell.remove_humus(0.2);
        let moisture_viability = Events::compute_moisture_viability(&ecosystem, index, &trees, 0);
        let expected = 0.25;
        assert!(
            approx_eq!(f32, moisture_viability, expected, epsilon = 0.01),
            "Expected {expected}, actual {moisture_viability}"
        );
        let viability = Events::compute_viability(&ecosystem, index, &trees, 0);
        assert!(
            approx_eq!(f32, viability, expected, epsilon = 0.01),
            "Expected {expected}, actual {viability}"
        );

        // boost moisture content to above max limit
        let cell = &mut ecosystem[index];
        cell.soil_moisture = 3E5;
        let moisture_viability = Events::compute_moisture_viability(&ecosystem, index, &trees, 0);
        assert_eq!(moisture_viability, -1.0);
        let viability = Events::compute_viability(&ecosystem, index, &trees, 0);
        assert_eq!(viability, -1.0);
    }
}

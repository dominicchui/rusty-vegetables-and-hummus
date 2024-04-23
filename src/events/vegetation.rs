use itertools::Itertools;
use rand::Rng;

use super::Events;
use crate::{
    constants,
    ecology::{Bushes, Cell, CellIndex, Ecosystem, Grasses, Trees},
};

// % of dead vegetation that is converted to humus while the rest rots away (disappears)
const DEAD_VEGETATION_TO_HUMUS_RATE: f32 = 0.3;
// https://link.springer.com/referenceworkentry/10.1007/978-1-4020-3995-9_406
const HUMUS_DENSITY: f32 = 1500.0; // in kg per cubic meter

// viability constants for vegetation
trait Vegetation {
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

    // number of new plants per square meter per year
    const ESTABLISHMENT_RATE: f32;
    // impact of density on seedling count
    const SEEDLING_DENSITY_CONSTANT: f32;
    // impact of vigor on seedlign count
    const SEEDLING_VIGOR_CONSTANT: f32;
    // meter per plant per year
    const GROWTH_RATE: f32;
    const LIFE_EXPECTANCY: f32;
    // impact of stress on number of plants
    const STRESS_DEATH_CONSTANT: f32;
    // impact of age on number of plants
    const SENESCENCE_DEATH_CONSTANT: f32;
}

impl Vegetation for Trees {
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
    const ILLUMINATION_IDEAL_MAX: f32 = 10.0;
    const ILLUMINATION_LIMIT_MAX: f32 = 14.0;

    const ESTABLISHMENT_RATE: f32 = 0.24;
    const SEEDLING_DENSITY_CONSTANT: f32 = 0.05;
    const SEEDLING_VIGOR_CONSTANT: f32 = 0.5;
    const GROWTH_RATE: f32 = 0.3;
    const LIFE_EXPECTANCY: f32 = 80.0;
    const STRESS_DEATH_CONSTANT: f32 = 1.0;
    const SENESCENCE_DEATH_CONSTANT: f32 = 0.05;
}

impl Vegetation for Bushes {
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

    const ESTABLISHMENT_RATE: f32 = 0.0;
    const SEEDLING_DENSITY_CONSTANT: f32 = 0.0;
    const SEEDLING_VIGOR_CONSTANT: f32 = 0.0;
    const GROWTH_RATE: f32 = 0.0;
    const LIFE_EXPECTANCY: f32 = 0.0;
    const STRESS_DEATH_CONSTANT: f32 = 0.0;
    const SENESCENCE_DEATH_CONSTANT: f32 = 0.0;
}

impl Vegetation for Grasses {
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

    const ESTABLISHMENT_RATE: f32 = 0.0;
    const SEEDLING_DENSITY_CONSTANT: f32 = 0.0;
    const SEEDLING_VIGOR_CONSTANT: f32 = 0.0;
    const GROWTH_RATE: f32 = 0.0;
    const LIFE_EXPECTANCY: f32 = 0.0;
    const STRESS_DEATH_CONSTANT: f32 = 0.0;
    const SENESCENCE_DEATH_CONSTANT: f32 = 0.0;
}

trait Individualized {
    // get density
}

impl Events {
    pub(crate) fn apply_vegetation_trees_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let cell = &ecosystem[index];
        // println!("\nindex {index} trees {:?}", cell.trees);
        let mut new_dead_biomass = 0.0;
        let mut new_trees_option: Option<Trees> = None;

        let mut new_trees = if let Some(trees) = &cell.trees {
            trees.clone()
        } else {
            // no trees so potentially germinate some
            Trees::init()
        };

        let (vigor, stress) = Self::compute_vigor_and_stress(ecosystem, index, &new_trees);

        // Germination
        let mut density = Cell::estimate_tree_density(&new_trees);
        // println!("vigor {vigor}, stress {stress}, density {density}");
        if stress == 0.0 && density < 1.0 {
            // convert establishment rate from plants per square meter to plants per cell
            let mut seedling_count = (Trees::ESTABLISHMENT_RATE
                * constants::CELL_SIDE_LENGTH
                * constants::CELL_SIDE_LENGTH)
                * (Trees::SEEDLING_DENSITY_CONSTANT * (1.0 - density))
                * Trees::SEEDLING_VIGOR_CONSTANT
                * vigor;
            // println!("seedling_count {seedling_count}");
            // if seedling count is < 0, use it as probability of new seedling
            if seedling_count > 0.0 && seedling_count < 1.0 {
                let mut rng = rand::thread_rng();
                let rand: f32 = rng.gen();
                if rand < seedling_count {
                    seedling_count = 1.0;
                }
            }
            new_trees.number_of_plants += seedling_count as u32;
        }

        // need trees from here on
        if new_trees.number_of_plants > 0 {
            // Growth
            new_trees.plant_height_sum += new_trees.number_of_plants as f32 * Trees::GROWTH_RATE;
            new_trees.plant_age_sum += new_trees.number_of_plants as f32;

            // Death
            let pre_death_count = new_trees.number_of_plants;

            // overpopulation
            while density > 1.0 && new_trees.number_of_plants > 1 {
                let average_plant_height =
                    new_trees.plant_height_sum / new_trees.number_of_plants as f32;
                let average_plant_age = new_trees.plant_age_sum / new_trees.number_of_plants as f32;
                new_trees.number_of_plants -= 1;
                new_trees.plant_height_sum -= average_plant_height;
                new_trees.plant_age_sum -= average_plant_age;
                density = Cell::estimate_tree_density(&new_trees);
            }

            // stress
            new_trees.number_of_plants -= (stress * Trees::STRESS_DEATH_CONSTANT) as u32;

            // old age
            let average_age = new_trees.plant_age_sum / new_trees.number_of_plants as f32;
            let num_dead_from_old_age = if average_age > Trees::LIFE_EXPECTANCY {
                f32::ceil(
                    (1.0 - Trees::SENESCENCE_DEATH_CONSTANT) * new_trees.number_of_plants as f32,
                ) as u32
            } else {
                0
            };
            let average_plant_height =
                new_trees.plant_height_sum / new_trees.number_of_plants as f32;
            let average_plant_age = new_trees.plant_age_sum / new_trees.number_of_plants as f32;
            new_trees.number_of_plants -= num_dead_from_old_age;
            new_trees.plant_height_sum -= num_dead_from_old_age as f32 * average_plant_height;
            new_trees.plant_age_sum -= num_dead_from_old_age as f32 * average_plant_age;

            // create temporary new plant struct to calculate biomass
            let dead_trees = Trees {
                number_of_plants: pre_death_count - new_trees.number_of_plants,
                plant_height_sum: new_trees.plant_height_sum,
                plant_age_sum: 0.0, // age doesn't matter
            };

            // conversion to dead vegetation
            new_dead_biomass += dead_trees.estimate_biomass();

            // println!("new trees {new_trees:?}");

            // handle mutability restrictions
            if new_trees.number_of_plants > 0 {
                new_trees_option = Some(new_trees);
            }
        }

        let cell = &mut ecosystem[index];
        cell.trees = new_trees_option;

        // convert dead vegetation (from last year) to humus
        let new_humus = Self::convert_dead_vegetation_to_humus(cell.get_dead_vegetation_biomass());
        cell.remove_all_dead_vegetation();
        cell.add_humus(new_humus);

        // add new dead biomass to dead vegetation
        cell.add_dead_vegetation(new_dead_biomass);

        // does not propagate
        None
    }

    // given an amount of biomass, determine the height of humus to be produced
    fn convert_dead_vegetation_to_humus(biomass: f32) -> f32 {
        let converted_biomass = 0.3 * biomass;
        converted_biomass
            / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH * HUMUS_DENSITY)
    }

    // returns tuple of vigor and stress
    // vigor is average viability during growing season (T > 5°C)
    // stress is average of 4 worst negative viabilities
    fn compute_vigor_and_stress<T: Vegetation>(
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
        let num_months = growing_viabilities.len();
        let vigor = growing_viabilities.into_iter().sum::<f32>() / num_months as f32;

        // stress is average of 4 worst negative viabilities
        let mut negative_viabilities = viabilities.into_iter().filter(|v| *v < 0.0).collect_vec();
        negative_viabilities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let stress = if !negative_viabilities.is_empty() {
            negative_viabilities.iter().take(4).sum::<f32>() / negative_viabilities.len() as f32
        } else {
            0.0
        };
        (vigor, stress)
    }

    // returns viability for a given plant for a given month
    fn compute_viability<T: Vegetation>(
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
        // println!("temperature_viability {temperature_viability}");
        // println!("moisture_viability {moisture_viability}");
        // println!("illumination_viability {illumination_viability}");

        // viability is lowest of the the sub-values (Leibig’s law of the minimum)
        f32::min(
            temperature_viability,
            f32::min(moisture_viability, illumination_viability),
        )
    }

    fn compute_temperature_viability<T: Vegetation>(
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
            temperature if temperature <= T::TEMPERATURE_LIMIT_MAX => {
                (temperature - T::TEMPERATURE_LIMIT_MAX)
                    / (T::TEMPERATURE_IDEAL_MAX - T::TEMPERATURE_LIMIT_MAX)
            }
            _ => -1.0,
        }
    }

    fn compute_moisture_viability<T: Vegetation>(
        ecosystem: &Ecosystem,
        index: CellIndex,
        _: &T,
        month: usize,
    ) -> f32 {
        let cell = &ecosystem[index];
        // convert moisture in terms of volume to % by volume
        let moisture_volume = cell.get_monthly_soil_moisture(month); // in L
                                                                     // println!("moisture_volume {moisture_volume}");
                                                                     // println!("cell moisture {}", cell.soil_moisture);
                                                                     // bedrock, rock, sand, and humus can all hold water, but make simplifying assumption that all water makes it to humus layer
                                                                     // so each cell is 10x10xheight m, where height is height of humus
                                                                     // 1 cubic meter = 1000 liters
        let height = cell.get_humus_height();
        // println!("height {height}");
        let cell_volume =
            constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH * height * 1000.0; // in L
                                                                                         // println!("cell_volume {cell_volume}");
        let moisture = if cell_volume == 0.0 {
            0.0
        } else {
            moisture_volume / cell_volume
        };
        // println!("moisture {moisture}");

        match moisture {
            moisture if moisture < T::MOISTURE_LIMIT_MIN => -1.0,
            moisture if moisture < T::MOISTURE_IDEAL_MIN => {
                (moisture - T::MOISTURE_LIMIT_MIN) / (T::MOISTURE_IDEAL_MIN - T::MOISTURE_LIMIT_MIN)
            }
            moisture if moisture <= T::MOISTURE_IDEAL_MAX => 1.0,
            moisture if moisture <= T::MOISTURE_LIMIT_MAX => {
                (moisture - T::MOISTURE_LIMIT_MAX) / (T::MOISTURE_IDEAL_MAX - T::MOISTURE_LIMIT_MAX)
            }
            _ => -1.0,
        }
    }

    fn compute_illumination_viability<T: Vegetation>(
        ecosystem: &Ecosystem,
        index: CellIndex,
        _: &T,
        month: usize,
    ) -> f32 {
        let illumination = ecosystem.estimate_illumination(&index, month);
        // println!("illumination {illumination}");
        match illumination {
            illumination if illumination < T::ILLUMINATION_LIMIT_MIN => -1.0,
            illumination if illumination < T::ILLUMINATION_IDEAL_MIN => {
                (illumination - T::ILLUMINATION_LIMIT_MIN)
                    / (T::ILLUMINATION_IDEAL_MIN - T::ILLUMINATION_LIMIT_MIN)
            }
            illumination if illumination <= T::ILLUMINATION_IDEAL_MAX => 1.0,
            illumination if illumination <= T::ILLUMINATION_LIMIT_MAX => {
                (illumination - T::ILLUMINATION_LIMIT_MAX)
                    / (T::ILLUMINATION_IDEAL_MAX - T::ILLUMINATION_LIMIT_MAX)
            }
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
        cell.soil_moisture = 0.0;

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
        let expected = 0.75;
        assert!(
            approx_eq!(f32, moisture_viability, expected, epsilon = 0.01),
            "Expected {expected}, actual {moisture_viability}"
        );
        let viability = Events::compute_viability(&ecosystem, index, &trees, 0);
        let expected = 0.735; // temperature limited
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

    #[test]
    fn test_tree_compute_vigor_and_stress() {
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
        cell.soil_moisture = 1.8E5;

        let mut viabilities = vec![];
        for i in 0..12 {
            let temperature_viability =
                Events::compute_temperature_viability(&ecosystem, index, &trees, i);
            let moisture_viability =
                Events::compute_moisture_viability(&ecosystem, index, &trees, i);
            let illumination_viability =
                Events::compute_illumination_viability(&ecosystem, index, &trees, i);
            let viability = Events::compute_viability(&ecosystem, index, &trees, i);
            viabilities.push(viability);
            println!("VIABILITY FOR MONTH {i}: {viability}");
            println!("temp : {temperature_viability}");
            println!("moisture : {moisture_viability}");
            println!("light : {illumination_viability}");
        }

        let (vigor, stress) = Events::compute_vigor_and_stress(&ecosystem, index, &trees);

        // months 3-11 have temperature > 5
        // AVERAGE_MONTHLY_TEMPERATURES = [-2.0, -0.8, 2.8, 8.8, 14.3, 19.2, 23.0, 22.3, 18.7, 12.5, 6.7, 1.5]
        let expected_vigor = viabilities[3..11].iter().sum::<f32>() / 8.0;
        assert_eq!(vigor, expected_vigor);
        // all monthly viabilities expected to be > 0
        assert_eq!(stress, 0.0);
    }

    #[test]
    fn test_apply_vegetation_trees_event() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(0, 0);

        // case 1: simple growth
        let trees = Trees {
            number_of_plants: 1,
            plant_height_sum: 10.0,
            plant_age_sum: 20.0,
        };
        let cell = &mut ecosystem[index];
        cell.trees = Some(trees);
        // 50 cm of humus/soil
        cell.remove_bedrock(0.5);
        cell.add_humus(0.5);
        cell.soil_moisture = 1.8E5;

        Events::apply_vegetation_trees_event(&mut ecosystem, index);

        let cell = &mut ecosystem[index];
        assert!(cell.trees.is_some());
        let new_trees = cell.trees.as_ref().unwrap();
        assert!(new_trees.number_of_plants >= 1);
        assert!(new_trees.plant_height_sum > 10.0);
        assert!(new_trees.plant_age_sum > 20.0);
        assert_eq!(cell.get_humus_height(), 0.5);
        assert_eq!(cell.get_dead_vegetation_biomass(), 0.0);

        // case 2: overpopulation
        let trees = Trees {
            number_of_plants: 5,
            plant_height_sum: 100.0,
            plant_age_sum: 100.0,
        };
        let cell = &mut ecosystem[index];
        cell.trees = Some(trees);

        Events::apply_vegetation_trees_event(&mut ecosystem, index);
        let cell = &mut ecosystem[index];
        assert!(cell.trees.is_some());
        let new_trees = cell.trees.as_ref().unwrap();
        assert!(new_trees.number_of_plants < 5);
        assert!(new_trees.plant_height_sum < 100.0);
        assert!(new_trees.plant_age_sum < 100.0);
        assert_eq!(cell.get_humus_height(), 0.5);
        assert!(cell.get_dead_vegetation_biomass() > 0.0);

        // let another year pass so dead trees get converted to humus
        Events::apply_vegetation_trees_event(&mut ecosystem, index);
        let cell = &mut ecosystem[index];
        assert!(cell.trees.is_some());
        assert!(cell.get_humus_height() > 0.5);
        assert_eq!(cell.get_dead_vegetation_biomass(), 0.0);

        Events::apply_vegetation_trees_event(&mut ecosystem, index);
        Events::apply_vegetation_trees_event(&mut ecosystem, index);
        Events::apply_vegetation_trees_event(&mut ecosystem, index);
    }
}

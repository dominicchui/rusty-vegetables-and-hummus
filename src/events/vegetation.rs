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

// how vigor and stress affects grass coverage
const GRASSES_VIGOR_GROWTH: f32 = 0.5;
const GRASSES_STRESS_DEATH: f32 = 0.1;

// viability constants for vegetation
pub(crate) trait Vegetation {
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

    // if cell contains this plant, return it, otherwise init an empty one
    fn clone_from_cell(cell: &Cell) -> Self;

    fn estimate_biomass(&self) -> f32;

    // returns how much of the illumination of the cell should be applied to this vegetation layer based on coverage from other vegetation
    // e.g. bushes and grasses will be partially shaded by trees
    fn get_illumination_coverage_constant(cell: &Cell) -> f32;
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

    fn clone_from_cell(cell: &Cell) -> Self {
        if let Some(trees) = &cell.trees {
            trees.clone()
        } else {
            Trees::init()
        }
    }

    fn estimate_biomass(&self) -> f32 {
        self.estimate_biomass()
    }

    // trees are not shaded by other vegetation
    fn get_illumination_coverage_constant(_: &Cell) -> f32 {
        1.0
    }
}

impl Vegetation for Bushes {
    const TEMPERATURE_LIMIT_MIN: f32 = -30.0;
    const TEMPERATURE_IDEAL_MIN: f32 = 4.0;
    const TEMPERATURE_IDEAL_MAX: f32 = 16.0;
    const TEMPERATURE_LIMIT_MAX: f32 = 30.0;

    // sources:
    // https://www.acurite.com/blog/soil-moisture-guide-for-plants-and-vegetables.html
    const MOISTURE_LIMIT_MIN: f32 = 0.2;
    const MOISTURE_IDEAL_MIN: f32 = 0.4;
    const MOISTURE_IDEAL_MAX: f32 = 0.6;
    const MOISTURE_LIMIT_MAX: f32 = 0.8;

    const ILLUMINATION_LIMIT_MIN: f32 = 2.0;
    const ILLUMINATION_IDEAL_MIN: f32 = 4.0;
    const ILLUMINATION_IDEAL_MAX: f32 = 6.0;
    const ILLUMINATION_LIMIT_MAX: f32 = 12.0;

    fn clone_from_cell(cell: &Cell) -> Self {
        if let Some(bushes) = &cell.bushes {
            bushes.clone()
        } else {
            Bushes::init()
        }
    }

    fn estimate_biomass(&self) -> f32 {
        self.estimate_biomass()
    }

    fn get_illumination_coverage_constant(cell: &Cell) -> f32 {
        if let Some(trees) = &cell.trees {
            let tree_density = Cell::estimate_tree_density(trees);
            // todo placeholder value
            tree_density * 0.5
        } else {
            1.0
        }
    }
}

impl Vegetation for Grasses {
    // based on switchgrass
    const TEMPERATURE_LIMIT_MIN: f32 = -5.0;
    const TEMPERATURE_IDEAL_MAX: f32 = 20.0;
    const TEMPERATURE_LIMIT_MAX: f32 = 30.0;
    const TEMPERATURE_IDEAL_MIN: f32 = 38.0;

    const MOISTURE_LIMIT_MIN: f32 = 0.2;
    const MOISTURE_IDEAL_MIN: f32 = 0.4;
    const MOISTURE_IDEAL_MAX: f32 = 0.6;
    const MOISTURE_LIMIT_MAX: f32 = 0.8;

    const ILLUMINATION_LIMIT_MIN: f32 = 4.0;
    const ILLUMINATION_IDEAL_MIN: f32 = 6.0;
    const ILLUMINATION_IDEAL_MAX: f32 = 8.0;
    const ILLUMINATION_LIMIT_MAX: f32 = 12.0;

    fn clone_from_cell(cell: &Cell) -> Self {
        if let Some(grasses) = &cell.grasses {
            grasses.clone()
        } else {
            Grasses::init()
        }
    }

    fn estimate_biomass(&self) -> f32 {
        self.estimate_biomass()
    }

    fn get_illumination_coverage_constant(cell: &Cell) -> f32 {
        let mut modifier = 1.0;
        if let Some(trees) = &cell.trees {
            let tree_density = Cell::estimate_tree_density(trees);
            // todo placeholder value
            modifier *= 0.5 * tree_density;
        }
        if let Some(bushes) = &cell.bushes {
            let bushes_density = Cell::estimate_bushes_density(bushes);
            // todo placeholder value
            modifier *= 0.5 * bushes_density;
        }

        modifier
    }
}

pub(crate) trait Individualized {
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

    fn init(number_of_plants: u32, plant_height_sum: f32, plant_age_sum: f32) -> Self;
    fn set_in_cell(self, cell: &mut Cell);
    fn estimate_density(&self) -> f32;
    fn get_number_of_plants(&self) -> u32;
    fn get_plant_height_sum(&self) -> f32;
    fn get_plant_age_sum(&self) -> f32;
    fn update_number_of_plants(&mut self, amount: i32);
    fn update_plant_height_sum(&mut self, amount: f32);
    fn update_plant_age_sum(&mut self, amount: f32);
    fn kill_plants(&mut self, amount: u32);
}

impl Individualized for Trees {
    const ESTABLISHMENT_RATE: f32 = 0.24;
    const SEEDLING_DENSITY_CONSTANT: f32 = 0.05;
    const SEEDLING_VIGOR_CONSTANT: f32 = 0.5;
    const GROWTH_RATE: f32 = 0.3;
    const LIFE_EXPECTANCY: f32 = 80.0;
    const STRESS_DEATH_CONSTANT: f32 = 1.0;
    const SENESCENCE_DEATH_CONSTANT: f32 = 0.05;

    fn init(number_of_plants: u32, plant_height_sum: f32, plant_age_sum: f32) -> Self {
        Trees {
            number_of_plants,
            plant_height_sum,
            plant_age_sum,
        }
    }

    fn set_in_cell(self, cell: &mut Cell) {
        if self.get_number_of_plants() > 0 {
            cell.trees = Some(self);
        } else {
            cell.trees = None;
        }
    }

    fn estimate_density(&self) -> f32 {
        Cell::estimate_tree_density(self)
    }

    fn get_number_of_plants(&self) -> u32 {
        self.number_of_plants
    }

    fn get_plant_height_sum(&self) -> f32 {
        self.plant_height_sum
    }

    fn get_plant_age_sum(&self) -> f32 {
        self.plant_age_sum
    }

    fn update_number_of_plants(&mut self, amount: i32) {
        if amount > 0 {
            self.number_of_plants += amount as u32;
        } else {
            self.number_of_plants -= (-amount) as u32;
        }
    }

    fn update_plant_height_sum(&mut self, amount: f32) {
        self.plant_height_sum += amount;
    }

    fn update_plant_age_sum(&mut self, amount: f32) {
        self.plant_age_sum += amount;
    }

    fn kill_plants(&mut self, amount: u32) {
        let average_plant_height = self.get_plant_height_sum() / self.get_number_of_plants() as f32;
        let average_plant_age = self.get_plant_age_sum() / self.get_number_of_plants() as f32;
        self.update_number_of_plants(-(amount as i32));
        self.update_plant_height_sum(-(amount as f32) * average_plant_height);
        self.update_plant_age_sum(-(amount as f32) * average_plant_age);
    }
}

impl Individualized for Bushes {
    const ESTABLISHMENT_RATE: f32 = 0.24;
    const SEEDLING_DENSITY_CONSTANT: f32 = 0.05;
    const SEEDLING_VIGOR_CONSTANT: f32 = 0.5;
    const GROWTH_RATE: f32 = 0.2;
    const LIFE_EXPECTANCY: f32 = 20.0;
    const STRESS_DEATH_CONSTANT: f32 = 1.0;
    const SENESCENCE_DEATH_CONSTANT: f32 = 0.05;

    fn init(number_of_plants: u32, plant_height_sum: f32, plant_age_sum: f32) -> Self {
        Bushes {
            number_of_plants,
            plant_height_sum,
            plant_age_sum,
        }
    }

    fn set_in_cell(self, cell: &mut Cell) {
        if self.get_number_of_plants() > 0 {
            cell.bushes = Some(self);
        } else {
            cell.bushes = None;
        }
    }

    fn estimate_density(&self) -> f32 {
        Cell::estimate_bushes_density(self)
    }

    fn get_number_of_plants(&self) -> u32 {
        self.number_of_plants
    }

    fn get_plant_height_sum(&self) -> f32 {
        self.plant_height_sum
    }

    fn get_plant_age_sum(&self) -> f32 {
        self.plant_age_sum
    }

    fn update_number_of_plants(&mut self, amount: i32) {
        if amount > 0 {
            self.number_of_plants += amount as u32;
        } else {
            self.number_of_plants -= (-amount) as u32;
        }
    }

    fn update_plant_height_sum(&mut self, amount: f32) {
        self.plant_height_sum += amount;
    }

    fn update_plant_age_sum(&mut self, amount: f32) {
        self.plant_age_sum += amount;
    }

    fn kill_plants(&mut self, amount: u32) {
        let average_plant_height = self.get_plant_height_sum() / self.get_number_of_plants() as f32;
        let average_plant_age = self.get_plant_age_sum() / self.get_number_of_plants() as f32;
        self.update_number_of_plants(-(amount as i32));
        self.update_plant_height_sum(-(amount as f32) * average_plant_height);
        self.update_plant_age_sum(-(amount as f32) * average_plant_age);
    }
}

impl Events {
    pub(crate) fn apply_trees_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let cell = &ecosystem[index];
        let trees = Trees::clone_from_cell(cell);
        Self::apply_individualized_vegetation_event(ecosystem, index, trees)
    }

    pub(crate) fn apply_bushes_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let cell = &ecosystem[index];
        let bushes = Bushes::clone_from_cell(cell);
        Self::apply_individualized_vegetation_event(ecosystem, index, bushes)
    }

    pub(crate) fn apply_grasses_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        // treat grasses as a collective over the entire cell
        let cell = &ecosystem[index];
        let grasses = Grasses::clone_from_cell(cell);
        let (vigor, stress) = Self::compute_vigor_and_stress(ecosystem, index, &grasses);
        // directly modify coverage based on vigor and stress
        let mut new_coverage = grasses.coverage_density;
        if stress < 0.0 {
            let death_coverage = (-stress) * GRASSES_STRESS_DEATH;
            new_coverage += death_coverage;

            // convert to dead_vegetation
            let dead_biomass = Grasses::estimate_biomass_for_coverage_density(death_coverage);
            assert!(dead_biomass > 0.0, "{dead_biomass}");
            let cell = &mut ecosystem[index];
            cell.add_dead_vegetation(dead_biomass);
        } else if vigor > 0.0 {
            // growth only if no stress
            new_coverage += vigor * GRASSES_VIGOR_GROWTH;
        }

        // handle overpopulation
        if new_coverage > 1.0 {
            let death_coverage = new_coverage - 1.0;
            new_coverage = 1.0;

            // convert to dead_vegetation
            let dead_biomass = Grasses::estimate_biomass_for_coverage_density(death_coverage);
            assert!(dead_biomass > 0.0, "{dead_biomass}");
            let cell = &mut ecosystem[index];
            cell.add_dead_vegetation(dead_biomass);
        }

        let new_grasses = if new_coverage > 0.0 {
            Some(Grasses {
                coverage_density: new_coverage,
            })
        } else {
            None
        };
        let cell = &mut ecosystem[index];
        cell.grasses = new_grasses;

        None
    }

    pub(crate) fn apply_individualized_vegetation_event<
        T: Vegetation + Individualized + std::fmt::Debug,
    >(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
        mut vegetation: T,
    ) -> Option<(Events, CellIndex)> {
        let mut new_dead_biomass = 0.0;

        let (vigor, stress) = Self::compute_vigor_and_stress(ecosystem, index, &vegetation);

        // Germination
        let mut density = vegetation.estimate_density();
        // println!("vigor {vigor}, stress {stress}, density {density}");
        if stress == 0.0 && density < 1.0 {
            // convert establishment rate from plants per square meter to plants per cell
            let mut seedling_count =
                (T::ESTABLISHMENT_RATE * constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH)
                    * (T::SEEDLING_DENSITY_CONSTANT * (1.0 - density))
                    * T::SEEDLING_VIGOR_CONSTANT
                    * vigor;
            // if seedling count is < 0, use it as probability of new seedling
            if seedling_count > 0.0 && seedling_count < 1.0 {
                let mut rng = rand::thread_rng();
                let rand: f32 = rng.gen();
                if rand < seedling_count {
                    seedling_count = 1.0;
                }
            }
            vegetation.update_number_of_plants(seedling_count as i32);
        }
        // println!("Vegetation initial {vegetation:?}");

        // need non-zero vegetation from here on
        if vegetation.get_number_of_plants() > 0 {
            // Growth
            vegetation
                .update_plant_height_sum(vegetation.get_number_of_plants() as f32 * T::GROWTH_RATE);
            vegetation.update_plant_age_sum(vegetation.get_number_of_plants() as f32);

            // Death from three factors
            let pre_death_count = vegetation.get_number_of_plants();
            let pre_death_average_height =
                vegetation.get_plant_height_sum() / pre_death_count as f32;

            // 1) overpopulation
            while density > 1.0 && vegetation.get_number_of_plants() > 1 {
                vegetation.kill_plants(1);
                density = vegetation.estimate_density();
            }
            let overpopulation_deaths = pre_death_count - vegetation.get_number_of_plants();
            // println!("overpopulation_deaths {overpopulation_deaths}");

            // 2) stress (non-positive real number)
            let stress_deaths = ((-stress) * T::STRESS_DEATH_CONSTANT) as u32;
            // println!("stress_deaths {stress_deaths}");
            vegetation.kill_plants(stress_deaths);

            // 3) old age
            let average_age =
                vegetation.get_plant_age_sum() / vegetation.get_number_of_plants() as f32;
            let old_age_deaths = if average_age > T::LIFE_EXPECTANCY {
                f32::ceil(
                    (1.0 - T::SENESCENCE_DEATH_CONSTANT) * vegetation.get_number_of_plants() as f32,
                ) as u32
            } else {
                0
            };
            // println!("old_age_deaths {old_age_deaths}");
            vegetation.kill_plants(old_age_deaths);

            // create temporary new plant struct to calculate biomass
            let total_dead = pre_death_count - vegetation.get_number_of_plants();
            let dead_vegetation = T::init(
                total_dead,
                total_dead as f32 * pre_death_average_height,
                0.0,
            );
            // println!("dead_vegetation {dead_vegetation:?}");

            // conversion to dead vegetation
            new_dead_biomass += dead_vegetation.estimate_biomass();
            // println!("new_dead_biomass {new_dead_biomass}");
        }
        // println!("Vegetation end {vegetation:?}");

        let cell = &mut ecosystem[index];
        vegetation.set_in_cell(cell);
        // println!("Cell {cell:?}");

        // convert dead vegetation (from last year) to humus
        let new_humus = Self::convert_dead_vegetation_to_humus(cell.get_dead_vegetation_biomass());
        cell.remove_all_dead_vegetation();
        assert!(new_humus >= 0.0, "{new_humus}");
        cell.add_humus(new_humus);

        // add new dead biomass to dead vegetation
        assert!(
            new_dead_biomass >= 0.0,
            "new_dead_biomass {new_dead_biomass}"
        );
        cell.add_dead_vegetation(new_dead_biomass);

        // does not propagate
        None
    }

    // given an amount of biomass, determine the height of humus to be produced
    fn convert_dead_vegetation_to_humus(biomass: f32) -> f32 {
        let converted_biomass = DEAD_VEGETATION_TO_HUMUS_RATE * biomass;
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
        // println!("type {}", std::any::type_name::<T>());
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
        let cell = &ecosystem[index];
        let modifier = T::get_illumination_coverage_constant(cell);
        // println!("modifier {modifier}");
        let illumination = ecosystem.estimate_illumination(&index, month) * modifier;
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
            // let temperature_viability =
            //     Events::compute_temperature_viability(&ecosystem, index, &trees, i);
            // let moisture_viability =
            //     Events::compute_moisture_viability(&ecosystem, index, &trees, i);
            // let illumination_viability =
            //     Events::compute_illumination_viability(&ecosystem, index, &trees, i);
            let viability = Events::compute_viability(&ecosystem, index, &trees, i);
            viabilities.push(viability);
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
    fn test_apply_trees_event() {
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

        Events::apply_trees_event(&mut ecosystem, index);

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

        Events::apply_trees_event(&mut ecosystem, index);
        let cell = &mut ecosystem[index];
        assert!(cell.trees.is_some());
        let new_trees = cell.trees.as_ref().unwrap();
        assert!(new_trees.number_of_plants < 5);
        assert!(new_trees.plant_height_sum < 100.0);
        assert!(new_trees.plant_age_sum < 100.0);
        assert_eq!(cell.get_humus_height(), 0.5);
        assert!(cell.get_dead_vegetation_biomass() > 0.0);

        // let another year pass so dead trees get converted to humus
        Events::apply_trees_event(&mut ecosystem, index);
        let cell = &mut ecosystem[index];
        assert!(cell.trees.is_some());
        assert!(cell.get_humus_height() > 0.5);
        assert_eq!(cell.get_dead_vegetation_biomass(), 0.0);
    }

    #[test]
    fn test_apply_bushes_event() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(0, 0);

        // case 1: simple growth
        let bushes = Bushes {
            number_of_plants: 1,
            plant_height_sum: 2.0,
            plant_age_sum: 10.0,
        };
        let cell = &mut ecosystem[index];
        cell.bushes = Some(bushes);
        // 50 cm of humus/soil
        cell.remove_bedrock(0.5);
        cell.add_humus(0.5);
        cell.soil_moisture = 1.8E5;

        Events::apply_bushes_event(&mut ecosystem, index);

        let cell = &mut ecosystem[index];
        assert!(cell.bushes.is_some());
        let new_bushes = cell.bushes.as_ref().unwrap();
        assert!(new_bushes.number_of_plants >= 1);
        assert!(new_bushes.plant_height_sum > 2.0);
        assert!(new_bushes.plant_age_sum > 10.0);
        assert_eq!(cell.get_humus_height(), 0.5);
        assert_eq!(cell.get_dead_vegetation_biomass(), 0.0);

        // case 2: overpopulation
        let bushes = Bushes {
            number_of_plants: 100,
            plant_height_sum: 200.0,
            plant_age_sum: 1000.0,
        };
        let cell = &mut ecosystem[index];
        cell.bushes = Some(bushes);

        Events::apply_bushes_event(&mut ecosystem, index);
        let cell = &mut ecosystem[index];
        assert!(cell.bushes.is_some());
        let new_bushes = cell.bushes.as_ref().unwrap();
        assert!(new_bushes.number_of_plants < 100);
        assert!(new_bushes.plant_height_sum < 200.0);
        assert!(new_bushes.plant_age_sum < 1000.0);
        assert_eq!(cell.get_humus_height(), 0.5);
        assert!(cell.get_dead_vegetation_biomass() > 0.0);

        // let another year pass so dead bushes get converted to humus
        Events::apply_bushes_event(&mut ecosystem, index);
        let cell = &mut ecosystem[index];
        assert!(cell.bushes.is_some());
        assert!(cell.get_humus_height() > 0.5);
        assert_eq!(cell.get_dead_vegetation_biomass(), 0.0);
    }

    #[test]
    fn test_apply_grasses_event() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(0, 0);

        // case 1: simple growth
        let grasses = Grasses {
            coverage_density: 0.0,
        };
        let cell = &mut ecosystem[index];
        cell.grasses = Some(grasses);
        // 50 cm of humus/soil
        cell.remove_bedrock(0.5);
        cell.add_humus(0.5);
        cell.soil_moisture = 1.8E5;

        Events::apply_grasses_event(&mut ecosystem, index);

        let cell = &mut ecosystem[index];
        assert!(cell.grasses.is_some());
        let new_grasses = cell.grasses.as_ref().unwrap();
        assert!(new_grasses.coverage_density > 0.0);
        assert_eq!(cell.get_humus_height(), 0.5);
        assert_eq!(cell.get_dead_vegetation_biomass(), 0.0);

        // case 2: overpopulation
        let grasses = Grasses {
            coverage_density: 1.5,
        };
        let cell = &mut ecosystem[index];
        cell.grasses = Some(grasses);

        Events::apply_grasses_event(&mut ecosystem, index);

        let cell = &mut ecosystem[index];
        assert!(cell.grasses.is_some());
        let new_grasses = cell.grasses.as_ref().unwrap();
        assert!(new_grasses.coverage_density <= 1.0);
        assert_eq!(cell.get_humus_height(), 0.5);
        assert!(cell.get_dead_vegetation_biomass() > 0.0);
    }
}

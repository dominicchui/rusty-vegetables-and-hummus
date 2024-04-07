use crate::constants;
use std::{f32::consts::E, ops::{Index, IndexMut}};

pub struct Ecosystem {
    // Array of structs
    cells: Vec<Vec<Cell>>,
    // latitude, wind direction and strength, etc.
}

pub(crate) struct CellIndex {
    x: usize,
    y: usize,
}

impl Index<CellIndex> for Ecosystem {
    type Output = Cell;
    fn index(&self, index: CellIndex) -> &Self::Output {
        &self.cells[index.x][index.y]
    }
}
impl IndexMut<CellIndex> for Ecosystem {
    fn index_mut(&mut self, index: CellIndex) -> &mut Self::Output {
        &mut self.cells[index.x][index.y]
    }
}

pub(crate) struct Cell {
    pub(crate) layers: Vec<CellLayer>,
    pub(crate) soil_moisture: f32,
    pub(crate) sunlight: f32,
    pub(crate) temperature: f32,
}

pub(crate) enum CellLayer {
    Bedrock(Bedrock),
    Rock(Rock),
    Sand(Sand),
    Humus(Humus),
    Trees(Trees),
    Bushes(Bushes),
    Grasses(Grasses),
    DeadVegetation(DeadVegetation),
}

struct Bedrock {
    height: f32,
}

struct Rock {
    height: f32,
}

struct Sand {
    height: f32,
}

struct Humus {
    height: f32,
}


pub(crate) struct Trees {
    pub(crate) number_of_plants: u32,
    // height ∝ diameter ^ (2/3)
    pub(crate) plant_height_sum: f32,
    pub(crate) plant_age_sum: f32,
}

struct Bushes {
    number_of_plants: u32,
    plant_height_sum: f32,
    plant_age_sum: f32,
}

struct Grasses {
    density: f32,
}

pub(crate) struct DeadVegetation {
    pub(crate) biomass: f32, // in kg
}

// Maybe this should be a static of some sort? It captures the nature of a given type of plant that holds for all types
struct Plant {
    name: String,
    establishment_rate: f32, // saplings per area per year
    growth_rate: f32,        // growth in height per tree per year
    life_expectancy: f32,
    temperature_e_min: f32,
    temperature_e_max: f32,
    temperature_i_min: f32,
    temperature_i_max: f32,
    // etc...
}

impl Cell {

    pub fn get_trees_layer(&self) -> Option<&Trees> {
        for layer in &self.layers {
            if let CellLayer::Trees(trees) = layer {
                return Some(trees);
            }
        }
        None
    }

    pub fn get_trees_layer_mut(&mut self) -> Option<&mut Trees> {
        for layer in &mut self.layers {
            if let CellLayer::Trees(trees) = layer {
                return Some(trees);
            }
        }
        None
    }

    pub fn get_dead_vegetation_layer(&self) -> Option<&DeadVegetation> {
        for layer in &self.layers {
            if let CellLayer::DeadVegetation(dead_vegetation) = layer {
                return Some(dead_vegetation);
            }
        }
        None
    }

    pub fn get_dead_vegetation_layer_mut(&mut self) -> Option<&mut DeadVegetation> {
        for layer in &mut self.layers {
            if let CellLayer::DeadVegetation(dead_vegetation) = layer {
                return Some(dead_vegetation);
            }
        }
        None
    }

    pub fn add_dead_vegetation(&mut self, biomass: f32) {
        if let Some(dead_vegetation) = self.get_dead_vegetation_layer_mut() {
            dead_vegetation.biomass += biomass;
        } else {
            // add new layer
            let dead_vegetation = DeadVegetation { biomass };
            let layer = CellLayer::DeadVegetation(dead_vegetation);
            self.layers.push(layer);
        }
    }

    pub fn get_tree_biomass(&self) -> f32 {
        let mut biomass = 0.0;
        // assume max one tree layer
        if let Some(trees) = self.get_trees_layer() {
            biomass += trees.get_tree_biomass();
        }
        biomass
    }
}

impl Trees {
    pub fn get_tree_biomass(&self) -> f32 {
        // based on biomass equation for red maples
        // source: https://academic.oup.com/forestry/article/87/1/129/602137#9934369
        // biomass in kg = e ^ (-2.0470 + 2.3852*ln(diameter in cm))
        let average_height = self.plant_height_sum / self.number_of_plants as f32;
        let average_diameter = Trees::estimate_diameter_from_height(average_height);
        let average_biomass = f32::powf(E, -2.0470 + 2.3852 * f32::ln(average_diameter));
        average_biomass * self.number_of_plants as f32
    }

    pub fn estimate_diameter_from_height(height: f32) -> f32 {
        // based on red maples
        // source: https://www.ccsenet.org/journal/index.php/jps/article/view/69956
        // log(height in m) = 0.6 * log(diameter in cm) - 0.4
        f32::powf(10.0, (f32::log10(height) - 0.6) / 0.4)
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;

    use crate::ecology::{Cell, CellLayer, Trees};
    #[test]
    fn get_tree_biovolume() {
        let layer = CellLayer::Trees(Trees {
            number_of_plants: 1,
            plant_height_sum: 10.0,
            plant_age_sum: 10.0
        });
        let mut cell = Cell {
            layers: vec![layer],
            soil_moisture: 0.0,
            sunlight: 0.0,
            temperature: 0.0,
        };
        let volume = cell.get_tree_biomass();
        let expected = 31.3472318147;
        assert!(
            approx_eq!(f32,
            volume, expected, epsilon = 0.001),
            "Expected volume {expected}, actual volume {volume}"
        );

        if let Some(trees) = cell.get_trees_layer_mut() {
            trees.number_of_plants = 5;
            trees.plant_height_sum = 50.0;
        }
        let volume = cell.get_tree_biomass();
        let expected = 156.7361590735;
        assert!(
            approx_eq!(f32,
                volume, expected, epsilon = 0.001),
            "Expected volume {expected}, actual volume {volume}"
        );
    }

    #[test]
    fn estimate_diameter_from_height() {
        let estimate = Trees::estimate_diameter_from_height(10.0);
        let expected = 10.0;
        assert!(approx_eq!(f32, expected, estimate, epsilon = 0.00001), "Expected {expected}; actual {estimate}");
    }
}

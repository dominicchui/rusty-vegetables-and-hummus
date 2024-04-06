use crate::constants;
use std::ops::{Index, IndexMut};

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
    // pub(crate) side_length: f32,
}

pub(crate) struct CellLayer {
    pub(crate) layer_type: CellLayerKind,
}

pub(crate) enum CellLayerKind {
    Bedrock(Bedrock),
    GranularMaterial(GranularMaterial),
    Vegetation(Vegetation),
    DeadVegetation(DeadVegetation),
}

struct GranularMaterial {
    layer_type: GranularMaterialKind,
    height: f32,
    critical_angle: f32,
}

enum GranularMaterialKind {
    Rock(Rock),
    Sand(Sand),
    Humus(Humus),
}

struct Bedrock {
    height: f32,
}

struct Rock {}

struct Sand {}

struct Humus {}

pub(crate) struct Vegetation {
    pub(crate) vegetation_kind: VegetationKind,
}

pub(crate) enum VegetationKind {
    Trees(Trees),
    Bushes(Bushes),
    Grasses(Grasses),
}

pub(crate) struct Trees {
    pub(crate) number_of_plants: u32,
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
    pub(crate) volume: f32,
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

// // list of all possible base layers
// enum BaseLayerKind {
//     Bedrock(Bedrock),
//     Rock(Rock),
//     Sand(Sand),
//     Humus(Humus),
//     Trees(Trees),
//     Bushes(Bushes),
//     Grasses(Grasses),
// }

impl Cell {
    pub fn get_trees_layer(&self) -> Option<&Trees> {
        for layer in &self.layers {
            if let CellLayerKind::Vegetation(vegetation) = &layer.layer_type {
                if let VegetationKind::Trees(trees) = &vegetation.vegetation_kind {
                    return Some(trees);
                }
            }
        }
        None
    }

    pub fn get_trees_layer_mut(&mut self) -> Option<&mut Trees> {
        for layer in &mut self.layers {
            if let CellLayerKind::Vegetation(vegetation) = &mut layer.layer_type {
                if let VegetationKind::Trees(trees) = &mut vegetation.vegetation_kind {
                    return Some(trees);
                }
            }
        }
        None
    }

    pub fn get_dead_vegetation_layer(&self) -> Option<&DeadVegetation> {
        for layer in &self.layers {
            if let CellLayerKind::DeadVegetation(dead_vegetation) = &layer.layer_type {
                return Some(dead_vegetation);
            }
        }
        None
    }

    pub fn get_dead_vegetation_layer_mut(&mut self) -> Option<&mut DeadVegetation> {
        for layer in &mut self.layers {
            if let CellLayerKind::DeadVegetation(dead_vegetation) = &mut layer.layer_type {
                return Some(dead_vegetation);
            }
        }
        None
    }

    pub fn add_dead_vegetation(&mut self, volume: f32) {
        if let Some(dead_vegetation) = self.get_dead_vegetation_layer_mut() {
            dead_vegetation.volume += volume;
        } else {
            // add new layer
            let dead_vegetation = DeadVegetation { volume };
            let layer = CellLayer {
                layer_type: CellLayerKind::DeadVegetation(dead_vegetation),
            };
            self.layers.push(layer);
        }
    }

    // "an average green wood weight (m in kg / m3), ... bio-volume (b), ... average plant height (h): b = 0.52h^(2.55) / m"
    pub fn get_tree_biovolume(&self) -> f32 {
        let mut biovolume = 0.0;
        // assume max one tree layer
        if let Some(trees) = self.get_trees_layer() {
            biovolume += trees.get_tree_biovolume();
        }
        biovolume
    }
}

impl Trees {
    pub fn get_tree_biovolume(&self) -> f32 {
        self.number_of_plants as f32 * (0.52 * f32::powf(self.plant_height_sum / self.number_of_plants as f32, 2.55) / constants::AVERAGE_GREEN_WOOD_WEIGHT)
    }
}

#[cfg(test)]
mod tests {
    use crate::ecology::{Cell, CellLayer, CellLayerKind, Trees, Vegetation, VegetationKind};
    #[test]
    fn get_tree_biovolume() {
        let layer = CellLayer {
            layer_type: CellLayerKind::Vegetation(Vegetation {
                vegetation_kind: VegetationKind::Trees(Trees {
                    number_of_plants: 1,
                    plant_height_sum: 30.0,
                    plant_age_sum: 10.0,
                }),
            }),
        };
        let mut cell = Cell {
            layers: vec![layer],
            soil_moisture: 0.0,
            sunlight: 0.0,
            temperature: 0.0,
        };
        let volume = cell.get_tree_biovolume();
        let expected = 3.0385225;
        assert!(
            volume == expected,
            "Expected volume {expected}, actual volume {volume}"
        );

        if let Some(trees) = cell.get_trees_layer_mut() {
            trees.number_of_plants = 5;
            trees.plant_height_sum = 150.0;
        }
        let volume = cell.get_tree_biovolume();
        let expected = 15.1926125;
        assert!(
            volume == expected,
            "Expected volume {expected}, actual volume {volume}"
        );
    }
}

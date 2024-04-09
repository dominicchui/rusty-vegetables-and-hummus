use itertools::Itertools;
use nalgebra::Vector3;

use crate::constants;
use std::{
    f32::consts::E,
    ops::{Index, IndexMut},
};

pub struct Ecosystem {
    // Array of structs
    cells: Vec<Vec<Cell>>,
    // latitude, wind direction and strength, etc.
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) struct CellIndex {
    x: usize,
    y: usize,
}

impl CellIndex {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
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

#[derive(Clone)]
pub(crate) struct Cell {
    pub(crate) layers: Vec<CellLayer>,
    pub(crate) soil_moisture: f32,
    pub(crate) sunlight: f32,
    pub(crate) temperature: f32,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub(crate) struct Bedrock {
    pub(crate) height: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Rock {
    pub(crate) height: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Sand {
    pub(crate) height: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Humus {
    pub(crate) height: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Trees {
    pub(crate) number_of_plants: u32,
    // height ∝ diameter ^ (2/3) apparently
    pub(crate) plant_height_sum: f32,
    pub(crate) plant_age_sum: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Bushes {
    pub(crate) number_of_plants: u32,
    pub(crate) plant_height_sum: f32,
    pub(crate) plant_age_sum: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Grasses {
    pub(crate) coverage_density: f32,
}

#[derive(Clone, Copy)]
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

impl Ecosystem {
    pub fn init() -> Self {
        let bedrock = CellLayer::Bedrock(Bedrock {
            height: constants::DEFAULT_BEDROCK_HEIGHT,
        });
        Ecosystem {
            cells: vec![
                vec![
                    Cell {
                        layers: vec![bedrock],
                        soil_moisture: 0.0,
                        sunlight: 0.0,
                        temperature: 0.0
                    };
                    100
                ];
                100
            ],
        }
    }

    pub(crate) fn get_normal(&self, index: CellIndex) -> Vector3<f32> {
        // normal of a vertex is the normalized sum of the normals of the adjacent faces
        // cells are vertices and the triangles formed between the cell and its 4 adjacent cells are faces

        // get neighbors
        let neighbors = Cell::get_neighbors(&index);
        // get normals of these triangles
        // triangles/faces are center-up-left, center-left-down, center-right-up, center-down-right (ccw winding)
        let mut face_normals = vec![];
        if let Some(up) = neighbors.north {
            if let Some(left) = neighbors.west {
                let up_left_normal = Cell::get_normal_of_triangle(self, index, up, left);
                face_normals.push(up_left_normal);
            }
        }
        if let Some(left) = neighbors.west {
            if let Some(down) = neighbors.south {
                let left_down_normal = Cell::get_normal_of_triangle(self, index, left, down);
                face_normals.push(left_down_normal);
            }
        }
        if let Some(right) = neighbors.east {
            if let Some(up) = neighbors.north {
                let right_up_normal = Cell::get_normal_of_triangle(self, index, right, up);
                face_normals.push(right_up_normal);
            }
        }
        if let Some(down) = neighbors.south {
            if let Some(right) = neighbors.east {
                let down_right_normal = Cell::get_normal_of_triangle(self, index, down, right);
                face_normals.push(down_right_normal);
            }
        }
        println!("face normals {:?}", face_normals);
        // average face normals
        let normal_sum: Vector3<f32> = face_normals.iter().sum();
        normal_sum.normalize()
    }

    pub(crate) fn estimate_curvature(self, index: CellIndex) -> f32 {
        let mut curvatures = vec![];
        // get neighbors
        let neighbors = Cell::get_neighbors(&index);

        // get curvature along each edge
        if let Some(up) = neighbors.north {
            curvatures.push(self.estimate_curvature_between_points(index, up));
        }
        if let Some(down) = neighbors.south {
            curvatures.push(self.estimate_curvature_between_points(index, down));
        }
        if let Some(left) = neighbors.west {
            curvatures.push(self.estimate_curvature_between_points(index, left));
        }
        if let Some(right) = neighbors.east {
            curvatures.push(self.estimate_curvature_between_points(index, right));
        }

        // take geometric mean
        println!("curvatures: {curvatures:?}",);
        let sum = curvatures.iter().sum::<f32>();
        println!("sum {sum}");
        sum / curvatures.len() as f32
    }

    fn estimate_curvature_between_points(&self, i1: CellIndex, i2: CellIndex) -> f32 {
        let n1 = self.get_normal(i1);
        let n2 = self.get_normal(i2);
        let p1 = self.get_position_of_cell(&i1);
        let p2 = self.get_position_of_cell(&i2);

        println!("normals {n1}, {n2}");
        println!("positions {p1}, {p2}");
        let num = (n2 - n1).dot(&(p2 - p1));
        let denom = (f32::powf((p2 - p1).norm(), 2.0));
        println!("num {num}, denom {denom}");
        num / denom
        // (n2 - n1).dot(&(p2-p1)) / (f32::powf((p2 - p1).norm(),2.0))
    }

    fn get_position_of_cell(&self, index: &CellIndex) -> Vector3<f32> {
        let cell = &self[*index];
        let height = cell.get_height();
        Vector3::new(index.x as f32, index.y as f32, height)
    }
}

pub(crate) struct Neighbors {
    northwest: Option<CellIndex>,
    north: Option<CellIndex>,
    northeast: Option<CellIndex>,
    west: Option<CellIndex>,
    east: Option<CellIndex>,
    southwest: Option<CellIndex>,
    south: Option<CellIndex>,
    southeast: Option<CellIndex>,
}

impl Neighbors {
    pub fn init() -> Self {
        Neighbors {
            north: None,
            south: None,
            west: None,
            east: None,
            northwest: None,
            northeast: None,
            southwest: None,
            southeast: None,
        }
    }

    pub fn as_array(&self) -> [Option<CellIndex>; 8] {
        [
            self.northwest,
            self.north,
            self.northeast,
            self.west,
            self.east,
            self.southwest,
            self.south,
            self.southeast,
        ]
    }

    // returns number of Some fields
    pub fn len(&self) -> usize {
        self.as_array().iter().filter(|n| n.is_some()).count()
    }
}
// }

impl Cell {
    pub(crate) fn get_neighbors(index: &CellIndex) -> Neighbors {
        let x = index.x;
        let y = index.y;

        let mut neighbors = Neighbors::init();

        if x > 0 {
            neighbors.west = Some(CellIndex { x: x - 1, y });
            if y > 0 {
                neighbors.northwest = Some(CellIndex { x: x - 1, y: y - 1 });
            }
            if y < constants::AREA_SIDE_LENGTH - 1 {
                neighbors.southwest = Some(CellIndex { x: x - 1, y: y + 1 })
            }
        }
        if x < constants::AREA_SIDE_LENGTH - 1 {
            neighbors.east = Some(CellIndex { x: x + 1, y });
            if y > 0 {
                neighbors.northeast = Some(CellIndex { x: x + 1, y: y - 1 });
            }
            if y < constants::AREA_SIDE_LENGTH - 1 {
                neighbors.southeast = Some(CellIndex { x: x + 1, y: y + 1 })
            }
        };
        if y > 0 {
            neighbors.north = Some(CellIndex { x, y: y - 1 });
        }
        if y < constants::AREA_SIDE_LENGTH - 1 {
            neighbors.south = Some(CellIndex { x, y: y + 1 });
        }

        neighbors
    }

    pub(crate) fn get_normal_of_triangle(
        ecosystem: &Ecosystem,
        i1: CellIndex,
        i2: CellIndex,
        i3: CellIndex,
    ) -> Vector3<f32> {
        let c1 = &ecosystem[i1];
        let c2 = &ecosystem[i2];
        let c3 = &ecosystem[i3];
        let a = Vector3::new(i1.x as f32, i1.y as f32, c1.get_height());
        let b = Vector3::new(i2.x as f32, i2.y as f32, c2.get_height());
        let c = Vector3::new(i3.x as f32, i3.y as f32, c3.get_height());

        let ba = b - a;
        let ca = c - a;

        let mut normal = ba.cross(&ca).normalize();
        if normal.z < 0.0 {
            normal.z = -normal.z;
        }
        normal
    }

    pub(crate) fn get_height(self: &Cell) -> f32 {
        self.layers.iter().map(CellLayer::get_height).sum()
    }

    pub(crate) fn get_bedrock_layer(&self) -> Option<&Bedrock> {
        for layer in &self.layers {
            if let CellLayer::Bedrock(bedrock) = layer {
                return Some(bedrock);
            }
        }
        None
    }

    pub(crate) fn get_bedrock_layer_mut(&mut self) -> Option<&mut Bedrock> {
        for layer in &mut self.layers {
            if let CellLayer::Bedrock(bedrock) = layer {
                return Some(bedrock);
            }
        }
        None
    }

    pub(crate) fn get_rock_layer(&self) -> Option<&Rock> {
        for layer in &self.layers {
            if let CellLayer::Rock(rock) = layer {
                return Some(rock);
            }
        }
        None
    }

    pub(crate) fn get_rock_layer_mut(&mut self) -> Option<&mut Rock> {
        for layer in &mut self.layers {
            if let CellLayer::Rock(rock) = layer {
                return Some(rock);
            }
        }
        None
    }

    pub(crate) fn insert_rocks(&mut self, height: f32) {
        // assume bedrock is always layer 0 so insert after it
        let rock = Rock { height };
        let layers = &mut self.layers;
        layers.insert(1, CellLayer::Rock(rock));
    }

    pub(crate) fn get_sand_layer(&self) -> Option<&Sand> {
        for layer in &self.layers {
            if let CellLayer::Sand(sand) = layer {
                return Some(sand);
            }
        }
        None
    }

    pub(crate) fn get_sand_layer_mut(&mut self) -> Option<&mut Sand> {
        for layer in &mut self.layers {
            if let CellLayer::Sand(sand) = layer {
                return Some(sand);
            }
        }
        None
    }

    pub(crate) fn insert_sand(&mut self, height: f32) {
        // assume bedrock is always layer 0 and insert after rocks
        let sand = Sand { height };
        let index = if self.get_rock_layer().is_some() {
            1
        } else {
            2
        };
        let layers = &mut self.layers;
        layers.insert(index, CellLayer::Sand(sand));
    }

    pub(crate) fn get_trees_layer(&self) -> Option<&Trees> {
        for layer in &self.layers {
            if let CellLayer::Trees(trees) = layer {
                return Some(trees);
            }
        }
        None
    }

    pub(crate) fn get_trees_layer_mut(&mut self) -> Option<&mut Trees> {
        for layer in &mut self.layers {
            if let CellLayer::Trees(trees) = layer {
                return Some(trees);
            }
        }
        None
    }

    pub(crate) fn get_bushes_layer(&self) -> Option<&Bushes> {
        for layer in &self.layers {
            if let CellLayer::Bushes(bushes) = layer {
                return Some(bushes);
            }
        }
        None
    }

    pub(crate) fn get_bushes_layer_mut(&mut self) -> Option<&mut Bushes> {
        for layer in &mut self.layers {
            if let CellLayer::Bushes(bushes) = layer {
                return Some(bushes);
            }
        }
        None
    }

    pub(crate) fn get_grass_layer(&self) -> Option<&Grasses> {
        for layer in &self.layers {
            if let CellLayer::Grasses(grasses) = layer {
                return Some(grasses);
            }
        }
        None
    }

    pub(crate) fn get_grasses_layer_mut(&mut self) -> Option<&mut Grasses> {
        for layer in &mut self.layers {
            if let CellLayer::Grasses(grasses) = layer {
                return Some(grasses);
            }
        }
        None
    }

    pub(crate) fn get_dead_vegetation_layer(&self) -> Option<&DeadVegetation> {
        for layer in &self.layers {
            if let CellLayer::DeadVegetation(dead_vegetation) = layer {
                return Some(dead_vegetation);
            }
        }
        None
    }

    pub(crate) fn get_dead_vegetation_layer_mut(&mut self) -> Option<&mut DeadVegetation> {
        for layer in &mut self.layers {
            if let CellLayer::DeadVegetation(dead_vegetation) = layer {
                return Some(dead_vegetation);
            }
        }
        None
    }

    pub(crate) fn add_dead_vegetation(&mut self, biomass: f32) {
        if let Some(dead_vegetation) = self.get_dead_vegetation_layer_mut() {
            dead_vegetation.biomass += biomass;
        } else {
            // add new layer
            let dead_vegetation = DeadVegetation { biomass };
            let layer = CellLayer::DeadVegetation(dead_vegetation);
            self.layers.push(layer);
        }
    }

    pub(crate) fn estimate_tree_biomass(&self) -> f32 {
        let mut biomass = 0.0;
        // assume max one tree layer
        if let Some(trees) = self.get_trees_layer() {
            biomass += trees.estimate_biomass();
        }
        biomass
    }

    pub(crate) fn estimate_bush_biomass(&self) -> f32 {
        let mut biomass = 0.0;
        // assume max one bush layer
        if let Some(bushes) = self.get_bushes_layer() {
            biomass += bushes.estimate_biomass();
        }
        biomass
    }
}

impl CellLayer {
    pub(crate) fn get_height(&self) -> f32 {
        match self {
            CellLayer::Bedrock(bedrock) => bedrock.height,
            CellLayer::Rock(rock) => rock.height,
            CellLayer::Sand(sand) => sand.height,
            CellLayer::Humus(humus) => humus.height,
            CellLayer::Trees(_)
            | CellLayer::Bushes(_)
            | CellLayer::Grasses(_)
            | CellLayer::DeadVegetation(_) => 0.0,
        }
    }
}

impl Trees {
    pub fn estimate_biomass(&self) -> f32 {
        // based on allometric equation for red maples
        // source: https://academic.oup.com/forestry/article/87/1/129/602137#9934369
        // ln(biomass in kg) = -2.0470 + 2.3852 * ln(diameter in cm)
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

impl Bushes {
    pub fn estimate_biomass(&self) -> f32 {
        // based on allometric equation for rhododendren mariesii
        // source: https://link.springer.com/article/10.1007/s11056-023-09963-z
        // ln(biomass in kg) = -2.635 + 3.614 * ln(height in m)
        let average_height = self.plant_height_sum / self.number_of_plants as f32;
        let average_biomass = f32::powf(E, -2.635 + 3.614 * f32::ln(average_height));
        average_biomass * self.number_of_plants as f32
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use nalgebra::Vector3;

    use super::{Bedrock, CellIndex, Ecosystem, Humus, Rock, Sand};
    use crate::{
        constants,
        ecology::{Bushes, Cell, CellLayer, Trees},
    };

    #[test]
    fn test_ecosystem_init() {
        let ecosystem = Ecosystem::init();
        let cells = &ecosystem.cells;
        assert!(cells.len() == constants::AREA_SIDE_LENGTH);
        for cell_row in cells {
            assert!(cell_row.len() == constants::AREA_SIDE_LENGTH);
        }

        for i in 0..constants::AREA_SIDE_LENGTH {
            for j in 0..constants::AREA_SIDE_LENGTH {
                let index = CellIndex::new(i, j);
                let cell = &ecosystem[index];
                assert!(cell.get_height() == constants::DEFAULT_BEDROCK_HEIGHT);
            }
        }
    }

    #[test]
    fn test_get_neighbors() {
        let index = CellIndex::new(5, 10);
        let neighbors = Cell::get_neighbors(&index);
        assert!(neighbors.west == Some(CellIndex::new(4, 10)));
        assert!(neighbors.north == Some(CellIndex::new(5, 9)));
        assert!(neighbors.south == Some(CellIndex::new(5, 11)));
        assert!(neighbors.east == Some(CellIndex::new(6, 10)));
        assert!(neighbors.northeast == Some(CellIndex::new(6, 9)));
        assert!(neighbors.southeast == Some(CellIndex::new(6, 11)));
        assert!(neighbors.northwest == Some(CellIndex::new(4, 9)));
        assert!(neighbors.southwest == Some(CellIndex::new(4, 11)));

        let index = CellIndex::new(0, 0);
        let neighbors = Cell::get_neighbors(&index);
        assert!(neighbors.south == Some(CellIndex::new(0, 1)));
        assert!(neighbors.east == Some(CellIndex::new(1, 0)));
        assert!(neighbors.north.is_none());
        assert!(neighbors.west.is_none());
        assert!(neighbors.northeast.is_none());
        assert!(neighbors.southeast == Some(CellIndex::new(1, 1)));
        assert!(neighbors.northwest.is_none());
        assert!(neighbors.southwest.is_none());

        let index = CellIndex::new(1, 99);
        let neighbors = Cell::get_neighbors(&index);
        println!("neighbors {:?}", neighbors.as_array());
        println!(
            "neighbors flattened {:?}",
            neighbors.as_array().into_iter().flatten()
        );
        assert!(neighbors.north == Some(CellIndex::new(1, 98)));
        assert!(neighbors.east == Some(CellIndex::new(2, 99)));
        assert!(neighbors.west == Some(CellIndex::new(0, 99)));
        assert!(neighbors.south.is_none());
        assert!(neighbors.northeast == Some(CellIndex::new(2, 98)));
        assert!(neighbors.southeast.is_none());
        assert!(neighbors.northwest == Some(CellIndex::new(1, 98)));
        assert!(neighbors.southwest.is_none());
    }

    #[test]
    fn test_get_height() {
        let bedrock = CellLayer::Bedrock(Bedrock { height: 100.0 });
        let rocks = CellLayer::Rock(Rock { height: 10.0 });
        let sand = CellLayer::Sand(Sand { height: 5.0 });
        let humus = CellLayer::Humus(Humus { height: 1.1 });
        let trees = CellLayer::Trees(Trees {
            number_of_plants: 1,
            plant_height_sum: 10.0,
            plant_age_sum: 10.0,
        });
        let cell = Cell {
            layers: vec![bedrock, rocks, sand, humus, trees],
            soil_moisture: 0.0,
            sunlight: 0.0,
            temperature: 0.0,
        };
        assert!(cell.get_height() == 116.1);
    }

    #[test]
    fn test_get_normal() {
        let mut ecosystem = Ecosystem::init();
        let normal = ecosystem.get_normal(CellIndex::new(5, 5));
        let unit_z = Vector3::z();
        assert!(normal == unit_z, "Expected {unit_z}, actual: {normal}");

        let up = &mut ecosystem[CellIndex::new(4, 5)];
        let bedrock = up.get_bedrock_layer_mut().unwrap();
        bedrock.height = 99.0;

        let down = &mut ecosystem[CellIndex::new(6, 5)];
        let bedrock = down.get_bedrock_layer_mut().unwrap();
        bedrock.height = 101.0;

        let normal = ecosystem.get_normal(CellIndex::new(5, 5));
        let expected = Vector3::new(f32::sqrt(0.5), 0.0, f32::sqrt(0.5));
        assert!(approx_eq!(f32, expected[0], normal[0], epsilon = 0.001));
        assert!(approx_eq!(f32, expected[1], normal[1], epsilon = 0.001));
        assert!(approx_eq!(f32, expected[2], normal[2], epsilon = 0.001));
    }

    #[test]
    fn test_estimate_curvature() {
        // curvature of sphere should be 1/r^2
        let mut ecosystem = Ecosystem::init();
        // let curvature = ecosystem.estimate_curvature(CellIndex::new(5,5));
        // let expected = 0.0;
        // assert!(curvature == expected, "Expected {expected}, actual {curvature}", );

        let neighbor_height = 100.0 + f32::sqrt(3.0) / 2.0;
        println!("neighbor_height {neighbor_height}");

        let center = &mut ecosystem[CellIndex::new(5, 5)];
        let bedrock = center.get_bedrock_layer_mut().unwrap();
        bedrock.height = 101.0;

        let up = &mut ecosystem[CellIndex::new(5, 4)];
        let bedrock = up.get_bedrock_layer_mut().unwrap();
        bedrock.height = neighbor_height;

        let down = &mut ecosystem[CellIndex::new(5, 6)];
        let bedrock = down.get_bedrock_layer_mut().unwrap();
        bedrock.height = neighbor_height;

        let left = &mut ecosystem[CellIndex::new(4, 5)];
        let bedrock = left.get_bedrock_layer_mut().unwrap();
        bedrock.height = neighbor_height;

        let right = &mut ecosystem[CellIndex::new(6, 5)];
        let bedrock = right.get_bedrock_layer_mut().unwrap();
        bedrock.height = neighbor_height;

        let curvature = ecosystem.estimate_curvature(CellIndex::new(5, 5));
        let expected = 0.25;
        assert!(
            curvature == expected,
            "Expected {expected}, actual {curvature}",
        );
    }

    #[test]
    fn test_estimate_tree_biomass() {
        let layer = CellLayer::Trees(Trees {
            number_of_plants: 1,
            plant_height_sum: 10.0,
            plant_age_sum: 10.0,
        });
        let mut cell = Cell {
            layers: vec![layer],
            soil_moisture: 0.0,
            sunlight: 0.0,
            temperature: 0.0,
        };
        let biomass = cell.estimate_tree_biomass();
        let expected = 31.3472318147;
        assert!(
            approx_eq!(f32, biomass, expected, epsilon = 0.001),
            "Expected biomass {expected}, actual biomass {biomass}"
        );

        if let Some(trees) = cell.get_trees_layer_mut() {
            trees.number_of_plants = 5;
            trees.plant_height_sum = 50.0;
        }
        let biomass = cell.estimate_tree_biomass();
        let expected = 156.7361590735;
        assert!(
            approx_eq!(f32, biomass, expected, epsilon = 0.001),
            "Expected volume {biomass}, actual volume {biomass}"
        );
    }

    #[test]
    fn test_estimate_diameter_from_height() {
        let estimate = Trees::estimate_diameter_from_height(10.0);
        let expected = 10.0;
        assert!(
            approx_eq!(f32, expected, estimate, epsilon = 0.00001),
            "Expected {expected}; actual {estimate}"
        );
    }

    #[test]
    fn test_estimate_bush_biomass() {
        let layer = CellLayer::Bushes(Bushes {
            number_of_plants: 1,
            plant_height_sum: 1.5,
            plant_age_sum: 1.0,
        });
        let mut cell = Cell {
            layers: vec![layer],
            soil_moisture: 0.0,
            sunlight: 0.0,
            temperature: 0.0,
        };
        let volume = cell.estimate_bush_biomass();
        let expected = 0.3104;
        assert!(
            approx_eq!(f32, volume, expected, epsilon = 0.001),
            "Expected volume {expected}, actual volume {volume}"
        );

        if let Some(trees) = cell.get_bushes_layer_mut() {
            trees.number_of_plants = 5;
            trees.plant_height_sum = 7.5;
        }
        let volume = cell.estimate_bush_biomass();
        let expected = 1.5523;
        assert!(
            approx_eq!(f32, volume, expected, epsilon = 0.001),
            "Expected volume {expected}, actual volume {volume}"
        );
    }
}

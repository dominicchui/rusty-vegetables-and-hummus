use itertools::Itertools;
use nalgebra::Vector3;

use crate::constants;
use std::{
    fmt,
    ops::{Index, IndexMut},
};

pub struct Ecosystem {
    // Array of structs
    pub(crate) cells: Vec<Vec<Cell>>,
    // latitude, wind direction and strength, etc.
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub(crate) struct CellIndex {
    x: usize,
    y: usize,
}

impl fmt::Display for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Debug for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl CellIndex {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn get_from_flat_index(i: usize) -> Self {
        let y = i / constants::AREA_SIDE_LENGTH;
        let x = i % constants::AREA_SIDE_LENGTH;
        CellIndex::new(x, y)
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
    bedrock: Option<Bedrock>,
    rock: Option<Rock>,
    sand: Option<Sand>,
    humus: Option<Humus>,
    pub(crate) trees: Option<Trees>,
    pub(crate) bushes: Option<Bushes>,
    pub(crate) grasses: Option<Grasses>,
    dead_vegetation: Option<DeadVegetation>,

    pub(crate) soil_moisture: f32,
    pub(crate) sunlight: f32,
}

#[derive(Clone)]
pub(crate) enum CellLayer {
    Bedrock(Option<Bedrock>),
    Rock(Option<Rock>),
    Sand(Option<Sand>),
    Humus(Option<Humus>),
    Trees(Option<Trees>),
    Bushes(Option<Bushes>),
    Grasses(Option<Grasses>),
    DeadVegetation(Option<DeadVegetation>),
}

// use the methods to access and modify height of these layers
#[derive(Clone)]
pub(crate) struct Bedrock {
    height: f32,
}

#[derive(Clone)]
pub(crate) struct Rock {
    height: f32,
}

#[derive(Clone)]
pub(crate) struct Sand {
    height: f32,
}

#[derive(Clone)]
pub(crate) struct Humus {
    height: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct Trees {
    pub(crate) number_of_plants: u32,
    // height ∝ diameter ^ (2/3) apparently
    pub(crate) plant_height_sum: f32,
    pub(crate) plant_age_sum: f32,
}

#[derive(Clone)]
pub(crate) struct Bushes {
    pub(crate) number_of_plants: u32,
    pub(crate) plant_height_sum: f32,
    pub(crate) plant_age_sum: f32,
}

#[derive(Clone)]
pub(crate) struct Grasses {
    pub(crate) coverage_density: f32,
}

#[derive(Clone)]
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
        Ecosystem {
            cells: vec![
                vec![
                    Cell {
                        soil_moisture: 0.0,
                        sunlight: 0.0,
                        bedrock: Some(Bedrock {
                            height: constants::DEFAULT_BEDROCK_HEIGHT,
                        }),
                        rock: None,
                        sand: None,
                        humus: None,
                        trees: None,
                        bushes: None,
                        grasses: None,
                        dead_vegetation: None,
                    };
                    constants::AREA_SIDE_LENGTH
                ];
                constants::AREA_SIDE_LENGTH
            ],
        }
    }

    pub fn init_test() -> Self {
        let mut ecosystem = Self::init();
        let neighbor_height = 101.0 + f32::sqrt(3.0) / 2.0;
        let c_i = 2;

        let trees = Trees {
            number_of_plants: 15,
            plant_height_sum: 150.0,
            plant_age_sum: 10.0,
        };

        let center = &mut ecosystem[CellIndex::new(c_i, c_i)];
        let bedrock = center.bedrock.as_mut().unwrap();
        bedrock.height = 103.0;
        // center.trees = Some(trees.clone());

        let up = &mut ecosystem[CellIndex::new(c_i, c_i - 1)];
        let bedrock = up.bedrock.as_mut().unwrap();
        bedrock.height = neighbor_height;
        // up.trees = Some(trees.clone());

        let down = &mut ecosystem[CellIndex::new(c_i, c_i + 1)];
        let bedrock = down.bedrock.as_mut().unwrap();
        bedrock.height = neighbor_height;
        // down.trees = Some(trees.clone());

        let left = &mut ecosystem[CellIndex::new(c_i - 1, c_i)];
        let bedrock = left.bedrock.as_mut().unwrap();
        bedrock.height = neighbor_height;
        left.trees = Some(trees.clone());

        let right = &mut ecosystem[CellIndex::new(c_i + 1, c_i)];
        let bedrock = right.bedrock.as_mut().unwrap();
        bedrock.height = neighbor_height;
        // right.trees = Some(trees.clone());

        let up_left = &mut ecosystem[CellIndex::new(c_i - 1, c_i - 1)];
        let bedrock = up_left.bedrock.as_mut().unwrap();
        bedrock.height = neighbor_height;
        up_left.trees = Some(trees.clone());

        let up_right = &mut ecosystem[CellIndex::new(c_i + 1, c_i - 1)];
        let bedrock = up_right.bedrock.as_mut().unwrap();
        bedrock.height = neighbor_height;
        // up_right.trees = Some(trees.clone());

        let down_left = &mut ecosystem[CellIndex::new(c_i - 1, c_i + 1)];
        let bedrock = down_left.bedrock.as_mut().unwrap();
        bedrock.height = neighbor_height;
        down_left.trees = Some(trees.clone());

        let down_right = &mut ecosystem[CellIndex::new(c_i + 1, c_i + 1)];
        let bedrock = down_right.bedrock.as_mut().unwrap();
        bedrock.height = neighbor_height;
        // down_right.trees = Some(trees.clone());

        ecosystem
    }

    pub fn init_piles() -> Self {
        let mut ecosystem = Self::init();

        let c_i = 3;
        let center = &mut ecosystem[CellIndex::new(c_i, c_i)];
        center.add_sand(1.0);

        let down = &mut ecosystem[CellIndex::new(c_i, c_i + 1)];
        down.add_sand(1.0);

        let right = &mut ecosystem[CellIndex::new(c_i + 1, c_i)];
        right.add_sand(1.0);

        let down_right = &mut ecosystem[CellIndex::new(c_i + 1, c_i + 1)];
        down_right.add_sand(3.0);

        let new_center = &mut ecosystem[CellIndex::new(c_i - 2, c_i)];
        new_center.add_rocks(1.0);

        let new_down = &mut ecosystem[CellIndex::new(c_i - 2, c_i + 1)];
        new_down.add_rocks(1.0);

        let left = &mut ecosystem[CellIndex::new(c_i - 3, c_i)];
        left.add_rocks(1.0);

        let down_left = &mut ecosystem[CellIndex::new(c_i - 3, c_i + 1)];
        down_left.add_rocks(3.0);

        let up_left = &mut ecosystem[CellIndex::new(c_i - 3, c_i - 1)];
        up_left.add_humus(3.0);

        ecosystem
    }

    pub fn init_dunes() -> Self {
        let mut ecosystem = Self::init();
        let cell = &mut ecosystem[CellIndex::new(0, 1)];
        cell.add_sand(1.0);
        let cell = &mut ecosystem[CellIndex::new(0, 2)];
        cell.add_sand(2.0);
        let cell = &mut ecosystem[CellIndex::new(0, 3)];
        cell.add_sand(3.0);
        let cell = &mut ecosystem[CellIndex::new(0, 4)];
        cell.add_sand(4.0);

        // let cell = &mut ecosystem[CellIndex::new(2, 2)];
        // cell.add_sand(2.0);
        // let cell = &mut ecosystem[CellIndex::new(1, 2)];
        // cell.add_sand(1.0);
        // let cell = &mut ecosystem[CellIndex::new(3, 2)];
        // cell.add_sand(1.0);
        // let cell = &mut ecosystem[CellIndex::new(2, 1)];
        // cell.add_sand(1.0);
        // let cell = &mut ecosystem[CellIndex::new(2, 3)];
        // cell.add_sand(1.0);

        ecosystem
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
        // println!("face normals {face_normals:?}");
        let normal_sum: Vector3<f32> = face_normals.iter().sum();
        normal_sum.normalize()
    }

    pub(crate) fn estimate_curvature(&self, index: CellIndex) -> f32 {
        let mut curvatures = vec![];
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

        // take mean
        // println!("curvatures: {curvatures:?}",);
        let sum = curvatures.iter().sum::<f32>();
        // println!("sum {sum}");
        sum / curvatures.len() as f32
    }

    fn estimate_curvature_between_points(&self, i1: CellIndex, i2: CellIndex) -> f32 {
        let n1 = self.get_normal(i1);
        let n2 = self.get_normal(i2);
        let p1 = self.get_position_of_cell(&i1);
        let p2 = self.get_position_of_cell(&i2);

        // println!("normals {n1}, {n2}");
        // println!("positions {p1}, {p2}");
        let num = (n2 - n1).dot(&(p2 - p1).normalize());
        let denom = (p2 - p1).norm();
        // println!("num {num}, denom {denom}");
        num / denom
        // (n2 - n1).dot(&(p2-p1)) / (f32::powf((p2 - p1).norm(),2.0))
    }

    pub(crate) fn get_position_of_cell(&self, index: &CellIndex) -> Vector3<f32> {
        let cell = &self[*index];
        let height = cell.get_height();
        Vector3::new(index.x as f32, index.y as f32, height)
    }

    pub(crate) fn get_slope_between_points(&self, i1: CellIndex, i2: CellIndex) -> f32 {
        //s(q)=(E(p)−E(q))/∥p−q∥
        let height_1 = self[i1].get_height();
        let height_2 = self[i2].get_height();
        let pos_1 = self.get_position_of_cell(&i1);
        let pos_2 = self.get_position_of_cell(&i2);
        (height_1 - height_2) / (pos_1 - pos_2).norm()
    }

    // returns angle in degrees
    pub(crate) fn get_angle(slope: f32) -> f32 {
        if slope < 0.0 {
            let slope = -slope;
            -f32::asin(slope).to_degrees()
        } else {
            f32::asin(slope).to_degrees()
        }
    }

    // estimates the illumination of the cell based on traced rays from the sun moving across the sky
    // returns average daily hours of direct sunlight
    pub(crate) fn estimate_illumination(&self, index: &CellIndex, month: usize) -> f32 {
        // todo placeholder estimate
        constants::AVERAGE_SUNLIGHT_HOURS[month]
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

    // returns number of "Some" fields
    pub fn len(&self) -> usize {
        self.as_array().iter().filter(|n| n.is_some()).count()
    }
}

impl Cell {
    pub(crate) fn init() -> Self {
        Cell {
            soil_moisture: 0.0,
            sunlight: 0.0,
            bedrock: None,
            rock: None,
            sand: None,
            humus: None,
            trees: None,
            bushes: None,
            grasses: None,
            dead_vegetation: None,
        }
    }
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
                neighbors.southwest = Some(CellIndex { x: x - 1, y: y + 1 });
            }
        }
        if x < constants::AREA_SIDE_LENGTH - 1 {
            neighbors.east = Some(CellIndex { x: x + 1, y });
            if y > 0 {
                neighbors.northeast = Some(CellIndex { x: x + 1, y: y - 1 });
            }
            if y < constants::AREA_SIDE_LENGTH - 1 {
                neighbors.southeast = Some(CellIndex { x: x + 1, y: y + 1 });
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
        let mut height = 0.0;
        if let Some(bedrock) = &self.bedrock {
            // println!("bedrock height {}", bedrock.height);
            height += bedrock.height;
        }
        if let Some(rock) = &self.rock {
            height += rock.height;
        }
        if let Some(sand) = &self.sand {
            height += sand.height;
        }
        if let Some(humus) = &self.humus {
            height += humus.height;
        }
        height
    }

    pub(crate) fn get_monthly_temperature(self: &Cell, month: usize) -> f32 {
        // modulate temperature with height
        let height = self.get_height();
        constants::AVERAGE_MONTHLY_TEMPERATURES[month] - 0.0065 * height
    }

    pub(crate) fn get_monthly_soil_moisture(self: &Cell, month: usize) -> f32 {
        // distribute cell moisture by monthly rainfall patterns
        // cell moisture is volume of water in a cell
        let rainfall = constants::AVERAGE_MONTHLY_RAINFALL[month];
        let annual_rainfall: f32 = constants::AVERAGE_MONTHLY_RAINFALL.into_iter().sum();
        self.soil_moisture * (rainfall / annual_rainfall)
    }

    // *** LAYER ADDERS ***
    pub(crate) fn add_bedrock(&mut self, height: f32) {
        if let Some(bedrock) = &mut self.bedrock {
            bedrock.height += height;
        } else {
            self.bedrock = Some(Bedrock { height });
        }
    }

    pub(crate) fn add_rocks(&mut self, height: f32) {
        if let Some(rocks) = &mut self.rock {
            rocks.height += height;
        } else {
            self.rock = Some(Rock { height });
        }
    }

    pub(crate) fn add_sand(&mut self, height: f32) {
        if let Some(sand) = &mut self.sand {
            sand.height += height;
        } else {
            self.sand = Some(Sand { height });
        }
    }

    pub(crate) fn add_humus(&mut self, height: f32) {
        if let Some(humus) = &mut self.humus {
            humus.height += height;
        } else {
            self.humus = Some(Humus { height });
        }
    }

    pub(crate) fn add_dead_vegetation(&mut self, biomass: f32) {
        if let Some(dead_vegetation) = &mut self.dead_vegetation {
            dead_vegetation.biomass += biomass;
        } else {
            self.dead_vegetation = Some(DeadVegetation { biomass });
        }
    }

    // *** LAYER REMOVERS ***
    pub(crate) fn remove_bedrock(&mut self, height: f32) {
        if let Some(bedrock) = &mut self.bedrock {
            bedrock.height -= height;
        }
    }

    pub(crate) fn remove_sand(&mut self, height: f32) {
        if let Some(sand) = &mut self.sand {
            sand.height -= height;
        }
    }

    pub(crate) fn remove_rocks(&mut self, height: f32) {
        if let Some(rock) = &mut self.rock {
            rock.height -= height;
        }
    }

    pub(crate) fn remove_humus(&mut self, height: f32) {
        if let Some(humus) = &mut self.humus {
            humus.height -= height;
        }
    }

    // *** HEIGHT GETTERS ***

    pub(crate) fn get_bedrock_height(&self) -> f32 {
        if let Some(bedrock) = &self.bedrock {
            bedrock.height
        } else {
            0.0
        }
    }

    pub(crate) fn get_sand_height(&self) -> f32 {
        if let Some(sand) = &self.sand {
            sand.height
        } else {
            0.0
        }
    }

    pub(crate) fn get_humus_height(&self) -> f32 {
        if let Some(humus) = &self.humus {
            humus.height
        } else {
            0.0
        }
    }

    pub(crate) fn get_rock_height(&self) -> f32 {
        if let Some(rock) = &self.rock {
            rock.height
        } else {
            0.0
        }
    }

    pub(crate) fn get_dead_vegetation_biomass(&self) -> f32 {
        if let Some(dead_vegetation) = &self.dead_vegetation {
            dead_vegetation.biomass
        } else {
            0.0
        }
    }

    // *** HEIGHT SETTERS ***
    pub(crate) fn set_height_of_bedrock(&mut self, height: f32) {
        if let Some(bedrock) = &mut self.bedrock {
            bedrock.height = height;
        } else {
            self.bedrock = Some(Bedrock { height });
        }
    }

    // *** ECOLOGICAL ESTIMATERS ***

    pub(crate) fn estimate_tree_biomass(&self) -> f32 {
        let mut biomass = 0.0;
        // assume max one tree layer
        if let Some(trees) = &self.trees {
            biomass += trees.estimate_biomass();
        }
        biomass
    }

    pub(crate) fn estimate_bush_biomass(&self) -> f32 {
        let mut biomass = 0.0;
        // assume max one bush layer
        if let Some(bushes) = &self.bushes {
            biomass += bushes.estimate_biomass();
        }
        biomass
    }

    pub(crate) fn estimate_vegetation_density(&self) -> f32 {
        // sum density of trees, bushes, and grasses
        let mut density = 0.0;
        if let Some(trees) = &self.trees {
            density += Self::estimate_tree_density(trees);
        }
        if let Some(bushes) = &self.bushes {
            density += Self::estimate_bushes_density(bushes);
        }
        if let Some(grasses) = &self.grasses {
            density += grasses.coverage_density;
        }

        density
    }

    pub(crate) fn estimate_tree_density(trees: &Trees) -> f32 {
        // d =nπ(r ·h/n)^2 /w ^2
        // d = density, n = number of plants, h = sum of plant heights, w = width of cell, r = ratio of plant's canopy radius to height
        let n = trees.number_of_plants;
        let h = trees.plant_height_sum;
        let average_height = h / n as f32;
        let average_diameter = Trees::estimate_diameter_from_height(average_height);
        let average_crown_area = Trees::estimate_crown_area_from_diameter(average_diameter);
        let crown_area_sum = average_crown_area * n as f32;
        crown_area_sum / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH)
    }

    pub(crate) fn estimate_bushes_density(bushes: &Bushes) -> f32 {
        let n = bushes.number_of_plants;
        let biomass = bushes.estimate_biomass();
        let average_biomass = biomass / n as f32;
        let average_crown_area = Bushes::estimate_crown_area_from_biomass(average_biomass);
        let crown_area_sum = average_crown_area * n as f32;
        crown_area_sum / (constants::CELL_SIDE_LENGTH * constants::CELL_SIDE_LENGTH)
    }

    // fn estimate_plant_density(&self) -> f32 {

    // }
}

impl CellLayer {
    pub(crate) fn get_height(&self) -> f32 {
        match self {
            CellLayer::Bedrock(Some(bedrock)) => bedrock.height,
            CellLayer::Rock(Some(rock)) => rock.height,
            CellLayer::Sand(Some(sand)) => sand.height,
            CellLayer::Humus(Some(humus)) => humus.height,
            _ => 0.0,
        }
    }
}

impl Trees {
    pub(crate) fn estimate_biomass(&self) -> f32 {
        // based on allometric equation for red maples
        // source: https://academic.oup.com/forestry/article/87/1/129/602137#9934369
        // ln(biomass in kg) = -2.0470 + 2.3852 * ln(diameter in cm)
        let average_height = self.plant_height_sum / self.number_of_plants as f32;
        let average_diameter = Trees::estimate_diameter_from_height(average_height);
        let average_biomass = f32::powf(
            std::f32::consts::E,
            -2.0470 + 2.3852 * f32::ln(average_diameter),
        );
        average_biomass * self.number_of_plants as f32
    }

    pub(crate) fn estimate_diameter_from_height(height: f32) -> f32 {
        // based on red maples
        // source: https://www.ccsenet.org/journal/index.php/jps/article/view/69956
        // log(height in m) = 0.6 * log(diameter in cm) - 0.4
        f32::powf(10.0, (f32::log10(height) - 0.4) / 0.6)
    }

    pub(crate) fn estimate_crown_area_from_diameter(diameter: f32) -> f32 {
        // based on red maples
        // source: https://www.fs.usda.gov/rds/archive/Catalog/RDS-2016-0005
        // crown diameter in m = a + b * (dbh in cm) + c * dhb^2
        let crown_diameter = 0.72545 + 0.2537 * diameter + -0.00123 * diameter * diameter;
        // assume crown is circular, compute area in square meters
        let radius = crown_diameter / 2.0;
        std::f32::consts::PI * radius * radius
    }
}

impl Bushes {
    pub(crate) fn estimate_biomass(&self) -> f32 {
        // based on allometric equation for rhododendron mariesii
        // source: https://link.springer.com/article/10.1007/s11056-023-09963-z
        // ln(biomass in kg) = -2.635 + 3.614 * ln(height in m)
        let average_height = self.plant_height_sum / self.number_of_plants as f32;
        let average_biomass = f32::powf(
            std::f32::consts::E,
            -2.635 + 3.614 * f32::ln(average_height),
        );
        average_biomass * self.number_of_plants as f32
    }

    pub(crate) fn estimate_crown_area_from_biomass(biomass: f32) -> f32 {
        // based on allometric equation for rhododendron mariesii
        // source: https://link.springer.com/article/10.1007/s11056-023-09963-z
        // ln(crown area in m^2) = (ln(biomass in kg) + 0.435) / 1.324
        f32::powf(std::f32::consts::E, (f32::ln(biomass) + 0.435) / 1.324)
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use nalgebra::Vector3;

    use super::{Bedrock, CellIndex, Ecosystem, Humus, Rock, Sand};
    use crate::{
        constants,
        ecology::{Bushes, Cell, Trees},
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
        let x = 2;
        let y = 3;
        let index = CellIndex::new(x, y);
        let neighbors = Cell::get_neighbors(&index);
        assert!(neighbors.west == Some(CellIndex::new(x - 1, y)));
        assert!(neighbors.north == Some(CellIndex::new(x, y - 1)));
        assert!(neighbors.south == Some(CellIndex::new(x, y + 1)));
        assert!(neighbors.east == Some(CellIndex::new(x + 1, y)));
        assert!(neighbors.northeast == Some(CellIndex::new(x + 1, y - 1)));
        assert!(neighbors.southeast == Some(CellIndex::new(x + 1, y + 1)));
        assert!(neighbors.northwest == Some(CellIndex::new(x - 1, y - 1)));
        assert!(neighbors.southwest == Some(CellIndex::new(x - 1, y + 1)));

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

        let x = 2;
        let y = 0;
        let index = CellIndex::new(x, y);
        let neighbors = Cell::get_neighbors(&index);
        assert!(neighbors.north.is_none());
        assert_eq!(neighbors.east, Some(CellIndex::new(x + 1, y)));
        assert_eq!(neighbors.west, Some(CellIndex::new(x - 1, y)));
        assert_eq!(neighbors.south, Some(CellIndex::new(x, y + 1)));
        assert!(neighbors.northeast.is_none());
        assert_eq!(neighbors.southeast, Some(CellIndex::new(x + 1, y + 1)));
        assert!(neighbors.northwest.is_none());
        assert_eq!(neighbors.southwest, Some(CellIndex::new(x - 1, y + 1)));
    }

    #[test]
    fn test_get_height() {
        let bedrock = Bedrock { height: 100.0 };
        let rock = Rock { height: 10.0 };
        let sand = Sand { height: 5.0 };
        let humus = Humus { height: 1.1 };
        let trees = Trees {
            number_of_plants: 1,
            plant_height_sum: 10.0,
            plant_age_sum: 10.0,
        };
        let cell = Cell {
            soil_moisture: 0.0,
            sunlight: 0.0,
            bedrock: Some(bedrock),
            rock: Some(rock),
            sand: Some(sand),
            humus: Some(humus),
            trees: Some(trees),
            bushes: None,
            grasses: None,
            dead_vegetation: None,
        };
        assert_eq!(cell.get_height(), 116.1);
    }

    #[test]
    fn test_get_temperature() {
        let mut cell = Cell {
            soil_moisture: 0.0,
            sunlight: 0.0,
            bedrock: None,
            rock: None,
            sand: None,
            humus: None,
            trees: None,
            bushes: None,
            grasses: None,
            dead_vegetation: None,
        };
        assert_eq!(
            cell.get_monthly_temperature(0),
            constants::AVERAGE_MONTHLY_TEMPERATURES[0]
        );
        assert_eq!(
            cell.get_monthly_temperature(11),
            constants::AVERAGE_MONTHLY_TEMPERATURES[11]
        );

        cell.add_bedrock(100.0);
        assert_eq!(
            cell.get_monthly_temperature(0),
            constants::AVERAGE_MONTHLY_TEMPERATURES[0] - 0.0065 * 100.0
        );

        cell.add_rocks(10.0);
        cell.add_sand(10.0);
        cell.add_dead_vegetation(10.0);
        assert_eq!(
            cell.get_monthly_temperature(0),
            constants::AVERAGE_MONTHLY_TEMPERATURES[0] - 0.0065 * 120.0
        );
    }

    #[test]
    fn test_get_normal() {
        let mut ecosystem = Ecosystem::init();
        let normal = ecosystem.get_normal(CellIndex::new(2, 2));
        let unit_z = Vector3::z();
        assert!(normal == unit_z, "Expected {unit_z}, actual: {normal}");

        let up = &mut ecosystem[CellIndex::new(1, 2)];
        let bedrock = &mut up.bedrock.as_mut().unwrap();
        bedrock.height = 99.0;

        let down = &mut ecosystem[CellIndex::new(3, 2)];
        let bedrock = &mut down.bedrock.as_mut().unwrap();
        bedrock.height = 101.0;

        let normal = ecosystem.get_normal(CellIndex::new(2, 2));
        let expected = Vector3::new(f32::sqrt(0.5), 0.0, f32::sqrt(0.5));
        assert!(approx_eq!(f32, expected[0], normal[0], epsilon = 0.001));
        assert!(approx_eq!(f32, expected[1], normal[1], epsilon = 0.001));
        assert!(approx_eq!(f32, expected[2], normal[2], epsilon = 0.001));
    }

    // #[test]
    // fn test_estimate_curvature() {
    //     // curvature of sphere should be 1/r^2
    //     let mut ecosystem = Ecosystem::init();
    //     // let curvature = ecosystem.estimate_curvature(CellIndex::new(5,5));
    //     // let expected = 0.0;
    //     // assert!(curvature == expected, "Expected {expected}, actual {curvature}", );

    //     let neighbor_height = 100.0 + f32::sqrt(3.0) / 2.0;
    //     // println!("neighbor_height {neighbor_height}");

    //     let center = &mut ecosystem[CellIndex::new(5, 5)];
    //     let bedrock = &mut center.bedrock.as_mut().unwrap();
    //     bedrock.height = 101.0;

    //     let up = &mut ecosystem[CellIndex::new(5, 4)];
    //     let bedrock = &mut up.bedrock.as_mut().unwrap();
    //     bedrock.height = neighbor_height;

    //     let down = &mut ecosystem[CellIndex::new(5, 6)];
    //     let bedrock = &mut down.bedrock.as_mut().unwrap();
    //     bedrock.height = neighbor_height;

    //     let left = &mut ecosystem[CellIndex::new(4, 5)];
    //     let bedrock = &mut left.bedrock.as_mut().unwrap();
    //     bedrock.height = neighbor_height;

    //     let right = &mut ecosystem[CellIndex::new(6, 5)];
    //     let bedrock = &mut right.bedrock.as_mut().unwrap();
    //     bedrock.height = neighbor_height;

    //     let curvature = ecosystem.estimate_curvature(CellIndex::new(5, 5));
    //     let expected = 0.25;
    //     assert!(
    //         curvature == expected,
    //         "Expected {expected}, actual {curvature}",
    //     );
    // }

    #[test]
    fn test_get_slope() {
        let mut ecosystem = Ecosystem::init();
        let slope = ecosystem.get_slope_between_points(CellIndex::new(3, 3), CellIndex::new(3, 2));
        assert!(slope == 0.0);

        let center = &mut ecosystem[CellIndex::new(3, 3)];
        let bedrock = &mut center.bedrock.as_mut().unwrap();
        bedrock.height = 1.0;

        let up = &mut ecosystem[CellIndex::new(3, 2)];
        let bedrock = &mut up.bedrock.as_mut().unwrap();
        bedrock.height = 2.0;
        let slope = ecosystem.get_slope_between_points(CellIndex::new(3, 3), CellIndex::new(3, 2));
        let expected = -0.707;
        assert!(
            approx_eq!(f32, slope, expected, epsilon = 0.001),
            "Expected {expected}, actual {slope}"
        );
        assert!(
            Ecosystem::get_angle(slope) == -45.0,
            "expected {}, actual {}",
            -45.0,
            Ecosystem::get_angle(slope)
        );

        let slope = ecosystem.get_slope_between_points(CellIndex::new(3, 2), CellIndex::new(3, 3));
        let expected = 0.707;
        assert!(
            approx_eq!(f32, slope, expected, epsilon = 0.001),
            "Expected {expected}, actual {slope}"
        );
        assert!(
            Ecosystem::get_angle(slope) == 45.0,
            "expected {}, actual {}",
            45.0,
            Ecosystem::get_angle(slope)
        );
    }

    #[test]
    fn test_estimate_tree_biomass() {
        let trees = Trees {
            number_of_plants: 1,
            plant_height_sum: 10.0,
            plant_age_sum: 10.0,
        };
        let mut cell = Cell {
            soil_moisture: 0.0,
            sunlight: 0.0,
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
        let expected = 31.3472;
        assert!(
            approx_eq!(f32, biomass, expected, epsilon = 0.001),
            "Expected biomass {expected}, actual biomass {biomass}"
        );

        if let Some(trees) = &mut cell.trees {
            trees.number_of_plants = 5;
            trees.plant_height_sum = 50.0;
        }
        let biomass = cell.estimate_tree_biomass();
        let expected = 156.7362;
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
    fn test_estimate_tree_density() {
        // one tree
        let trees = Trees {
            number_of_plants: 1,
            plant_height_sum: 10.0,
            plant_age_sum: 10.0,
        };
        let density = Cell::estimate_tree_density(&trees);
        let expected = 0.0774;
        assert!(
            approx_eq!(f32, density, expected, epsilon = 0.001),
            "Expected {expected}, actual {density}"
        );

        // two trees
        let trees = Trees {
            number_of_plants: 2,
            plant_height_sum: 20.0,
            plant_age_sum: 10.0,
        };
        let density = Cell::estimate_tree_density(&trees);
        let expected = 0.0774 * 2.0;
        assert!(
            approx_eq!(f32, density, expected, epsilon = 0.001),
            "Expected {expected}, actual {density}"
        );

        // many trees
        let trees = Trees {
            number_of_plants: 15,
            plant_height_sum: 150.0,
            plant_age_sum: 10.0,
        };
        let density = Cell::estimate_tree_density(&trees);
        let expected = 0.0774 * 15.0;
        assert!(
            approx_eq!(f32, density, expected, epsilon = 0.001),
            "Expected {expected}, actual {density}"
        );
    }

    #[test]
    fn test_estimate_bush_biomass() {
        let bushes = Bushes {
            number_of_plants: 1,
            plant_height_sum: 1.5,
            plant_age_sum: 1.0,
        };
        let mut cell = Cell {
            soil_moisture: 0.0,
            sunlight: 0.0,
            bedrock: None,
            rock: None,
            sand: None,
            humus: None,
            trees: None,
            bushes: Some(bushes),
            grasses: None,
            dead_vegetation: None,
        };
        let volume = cell.estimate_bush_biomass();
        let expected = 0.3104;
        assert!(
            approx_eq!(f32, volume, expected, epsilon = 0.001),
            "Expected volume {expected}, actual volume {volume}"
        );

        if let Some(trees) = &mut cell.bushes {
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

    #[test]
    fn test_estimate_bushes_density() {
        // one bush
        let bushes = Bushes {
            number_of_plants: 1,
            plant_height_sum: 2.0,
            plant_age_sum: 10.0,
        };
        let density = Cell::estimate_bushes_density(&bushes);
        let expected = 0.0126;
        assert!(
            approx_eq!(f32, density, expected, epsilon = 0.001),
            "Expected {expected}, actual {density}"
        );

        // many bushes
        let bushes = Bushes {
            number_of_plants: 10,
            plant_height_sum: 20.0,
            plant_age_sum: 10.0,
        };
        let density = Cell::estimate_bushes_density(&bushes);
        let expected = 0.126;
        assert!(
            approx_eq!(f32, density, expected, epsilon = 0.001),
            "Expected {expected}, actual {density}"
        );
    }

    #[test]
    fn test_get_monthly_soil_moisture() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(2, 2);
        let cell = &mut ecosystem[index];

        // January
        let moisture = cell.get_monthly_soil_moisture(0);
        assert_eq!(moisture, 0.0);

        // 1 L of moisture
        cell.soil_moisture = 1.0;
        let moisture = cell.get_monthly_soil_moisture(0);
        assert_eq!(moisture, 96.0 / 1151.0);

        // 50 L of moisture
        cell.soil_moisture = 50.0;
        let moisture = cell.get_monthly_soil_moisture(0);
        assert_eq!(moisture, 50.0 * 96.0 / 1151.0);

        // July
        let moisture = cell.get_monthly_soil_moisture(6);
        assert_eq!(moisture, 50.0 * 87.0 / 1151.0);
    }
}

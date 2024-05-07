use nalgebra::Vector2;
use rand::Rng;
use stackblur_iter::{
    blur_argb,
    imgref::{Img, ImgExtMut},
    par_blur_argb,
};

use crate::{
    constants,
    ecology::{Cell, CellIndex, Ecosystem},
};

use super::Events;

const SALTATION_DISTANCE_FACTOR: f32 = 1.0;
const CARRYING_CAPACITY: f32 = 0.2; // each wind event can carry this much height of sand
const REPTATION_HEIGHT: f32 = 0.1;
const VENTURI_FACTOR: f32 = 5e-3;
const HIGH_FREQ_KERNEL_RADIUS: usize = 11;
const LOW_FREQ_KERNEL_RADIUS: usize = 25;
const HIGH_FREQ_DEVIATION: f32 = 5.0;
const LOW_FREQ_DEVIATION: f32 = 30.0;
const HIGH_FREQ_WEIGHT: f32 = 0.2;
const LOW_FREQ_WEIGHT: f32 = 0.8;

pub(crate) struct WindState {
    pub(crate) wind_rose: WindRose,
    pub(crate) wind_direction: f32,
    pub(crate) wind_strength: f32,
    pub(crate) high_freq_convolution: [f32; constants::NUM_CELLS],
    pub(crate) low_freq_convolution: [f32; constants::NUM_CELLS],
}

impl WindState {
    pub(crate) fn new() -> Self {
        WindState {
            wind_rose: WindRose::new(
                constants::WIND_DIRECTION,
                constants::WIND_STRENGTH,
                constants::WIND_STRENGTH,
            ),
            wind_direction: constants::WIND_DIRECTION,
            wind_strength: constants::WIND_STRENGTH,
            high_freq_convolution: [0.0; constants::NUM_CELLS],
            low_freq_convolution: [0.0; constants::NUM_CELLS],
        }
    }
}

// 8 slices of 45° each
// each slice has a min and max wind speed
pub(crate) struct WindRose {
    pub(crate) min_speed: [f32; 8],
    pub(crate) max_speed: [f32; 8],
    // the weight for the given slice being sampled
    pub(crate) weights: [f32; 8],
}

impl WindRose {
    // init based on default wind direction and speed
    pub(crate) fn new(direction: f32, min_strength: f32, max_strength: f32) -> Self {
        let mut min_speed = [0.0; 8];
        let mut max_speed = [0.0; 8];
        let mut weights = [0.0; 8];

        let bucket = (direction / 45.0) as usize;
        min_speed[bucket] = min_strength;
        max_speed[bucket] = max_strength;
        weights[bucket] = 1.0;

        WindRose {
            min_speed,
            max_speed,
            weights,
        }
    }

    pub(crate) fn update_wind(
        &mut self,
        direction: f32,
        min_strength: f32,
        max_strength: f32,
        weight: f32,
    ) {
        let bucket = (direction / 45.0) as usize;
        self.min_speed[bucket] = min_strength;
        self.max_speed[bucket] = max_strength;
        self.weights[bucket] = weight;
    }

    // probabilistically samples the wind distribution
    pub(crate) fn sample_wind(&self) -> (f32, f32) {
        let weight_sum: f32 = self.weights.iter().sum();
        if weight_sum == 0.0 {
            return (0.0, 0.0);
        }

        // get direction
        let mut rng = rand::thread_rng();
        let rand: f32 = rng.gen();
        let mut weight_acc = 0.0;
        let mut bucket = 0;
        for i in 0..7 {
            weight_acc += self.weights[i] / weight_sum;
            if rand < weight_acc {
                bucket = i;
                break;
            }
        }
        let direction = bucket as f32 * 45.0;

        // get strength
        let rand: f32 = rng.gen();
        let diff = self.max_speed[bucket] - self.min_speed[bucket];
        let strength = rand * diff + self.min_speed[bucket];

        (direction, strength)
    }
}

impl Events {
    pub(crate) fn apply_wind_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let (wind_dir, wind_str) = if let Some(wind_state) = &ecosystem.wind_state {
            get_local_wind(
                ecosystem,
                index,
                wind_state.wind_direction,
                wind_state.wind_strength,
            )
            // (wind_state.wind_direction, wind_state.wind_strength)
        } else {
            (constants::WIND_DIRECTION, constants::WIND_STRENGTH)
        };

        // Saltation
        // 1) lift a small amount of sand
        let cell = &mut ecosystem[index];
        let sand_height = cell.get_sand_height();
        let moved_height = f32::min(CARRYING_CAPACITY, sand_height);
        cell.remove_sand(moved_height);

        // 2) transport sand to target cell
        let wind_shadowing = get_wind_shadowing(ecosystem, index, wind_dir);
        // let local_strength = get_local_sand_strength(wind_str, wind_shadowing);
        let distance = get_saltation_distance(wind_str);
        let direction = get_wind_direction_vector(wind_dir);
        let target_vec = direction * distance;
        // the area is topologically a torus so wrap around edges
        // note: want modulus, not remainder, so ((a % b) + b) % b
        let target_x = (((index.x as i32 + target_vec.x as i32)
            % constants::AREA_SIDE_LENGTH as i32)
            + constants::AREA_SIDE_LENGTH as i32)
            % constants::AREA_SIDE_LENGTH as i32;
        let target_y = (((index.y as i32 + target_vec.y as i32)
            % constants::AREA_SIDE_LENGTH as i32)
            + constants::AREA_SIDE_LENGTH as i32)
            % constants::AREA_SIDE_LENGTH as i32;

        // println!("({target_x}, {target_y})");
        let target_index = CellIndex::new(target_x as usize, target_y as usize);
        let target = &mut ecosystem[target_index];
        target.add_sand(moved_height);

        // 3) on landing, sand can bounce or be deposited
        let bounce_probability = get_bounce_probability(ecosystem, index, wind_shadowing);
        let mut rng = rand::thread_rng();
        let rand: f32 = rng.gen();

        let result = if rand > bounce_probability {
            // bounce
            Some((Events::Wind, target_index))
        } else {
            // deposit
            None
        };

        // Reptation
        perform_reptation(ecosystem, target_index, moved_height);

        result
    }
}

fn perform_reptation(ecosystem: &mut Ecosystem, target_index: CellIndex, moved_height: f32) {
    // transport sand to 2 steepest neighbors (proportionally)
    let target = &mut ecosystem[target_index];
    let usable_sand = f32::max(target.get_sand_height() - moved_height, 0.0);
    let reptation_height = f32::min(REPTATION_HEIGHT, usable_sand);
    let (neighbor_1, neighbor_2) = get_two_steepest_neighbors(ecosystem, target_index);
    if let Some((slope_1, neighbor_1)) = neighbor_1 {
        let target = &mut ecosystem[target_index];
        target.remove_sand(reptation_height);

        if let Some((slope_2, neighbor_2)) = neighbor_2 {
            // proportionally distribute sand
            let reptation_ratio = if slope_1 + slope_2 == 0.0 {
                0.5
            } else {
                slope_1 / (slope_1 + slope_2)
            };
            let reptation_for_one = reptation_ratio * reptation_height;
            let reptation_for_two = reptation_height - reptation_for_one;
            ecosystem[neighbor_1].add_sand(reptation_for_one);
            ecosystem[neighbor_2].add_sand(reptation_for_two);
        } else {
            // only one neighbor so move all sand to it
            ecosystem[neighbor_1].add_sand(reptation_height);
        }
    }
}

fn get_wind_direction_vector(wind_angle: f32) -> Vector2<f32> {
    let wind_dir = wind_angle.to_radians();
    let x = wind_dir.sin();
    let y = wind_dir.cos();
    Vector2::new(x, y).normalize()
}

fn get_wind_direction_angle(wind_vec: Vector2<f32>) -> f32 {
    f32::atan2(wind_vec.y, wind_vec.x).to_degrees() + 180.0
}

fn get_local_wind(
    ecosystem: &Ecosystem,
    index: CellIndex,
    wind_dir: f32,
    wind_str: f32,
) -> (f32, f32) {
    // warp wind based on local relief
    // Venturi effects acceleratase wind at higher altitudes
    let local_wind_str = wind_str * (1.0 + VENTURI_FACTOR * ecosystem[index].get_height());
    let local_wind_dir = wind_dir;
    let mut local_wind_vec = get_wind_direction_vector(local_wind_dir) * local_wind_str;

    // change wind direction based on terrain gradient
    // ωi ◦v(p,t) = (1−α)v(p,t)+αkT i ∇Ti⊥(p) α = ∥∇Ti(p)∥
    let (high_freq_slope, dir) = get_slope_at_point_blurred(ecosystem, index, true);
    let mut orth_vec = Vector2::new(dir.y as f32, -dir.x as f32);
    if orth_vec.dot(&get_wind_direction_vector(local_wind_dir)) < 0.0 {
        orth_vec = -orth_vec;
    }
    let warp_high_freq =
        (1.0 - high_freq_slope) * local_wind_vec + high_freq_slope * HIGH_FREQ_DEVIATION * orth_vec;

    let (low_freq_slope, dir) = get_slope_at_point_blurred(ecosystem, index, false);
    let mut orth_vec = Vector2::new(dir.y as f32, -dir.x as f32);
    if orth_vec.dot(&get_wind_direction_vector(local_wind_dir)) < 0.0 {
        orth_vec = -orth_vec;
    }
    let warp_low_freq =
        (1.0 - low_freq_slope) * local_wind_vec + low_freq_slope * LOW_FREQ_DEVIATION * orth_vec;

    local_wind_vec = warp_high_freq * HIGH_FREQ_WEIGHT + warp_low_freq * LOW_FREQ_WEIGHT;

    // add wind shadowing
    let wind_shadowing = get_wind_shadowing(ecosystem, index, wind_dir);
    local_wind_vec = get_local_sand_strength_vec(local_wind_vec, wind_shadowing);

    (
        get_wind_direction_angle(local_wind_vec.normalize()),
        local_wind_vec.norm(),
    )
}

pub(crate) fn convolve_terrain(ecosystem: &mut Ecosystem) {
    let mut heights = [0.0; constants::NUM_CELLS];
    let mut min_height = f32::MAX;
    let mut max_height = f32::MIN;
    for i in 0..constants::AREA_SIDE_LENGTH {
        for j in 0..constants::AREA_SIDE_LENGTH {
            let height = ecosystem[CellIndex::new(i, j)].get_height();
            heights[i + j * constants::AREA_SIDE_LENGTH] = height;
            if height > max_height {
                max_height = height;
            }
            if height < min_height {
                min_height = height;
            }
        }
    }
    // normalize heights to fit within 256 values
    let norm_factor = 256.0 / (max_height - min_height);
    heights = heights.map(|v| (v - min_height) * norm_factor);

    let mut argb_heights: [u32; constants::NUM_CELLS] = [0; constants::NUM_CELLS];
    for (i, height) in heights.iter().enumerate() {
        let height = *height;
        let argb = (255 << 24) | ((height as u32) << 16) | ((height as u32) << 8) | (height as u32);
        argb_heights[i] = argb;
    }

    // high frequency blur
    let mut img = Img::new(
        argb_heights,
        constants::AREA_SIDE_LENGTH,
        constants::AREA_SIDE_LENGTH,
    );
    par_blur_argb(&mut img.as_mut(), HIGH_FREQ_KERNEL_RADIUS);

    // convert back to f32 heights
    let mut high_freq_terrain = [0.0; constants::NUM_CELLS];
    for (i, pixel) in img.buf().iter().enumerate() {
        high_freq_terrain[i] = (*pixel as u8) as f32 * (1.0 / norm_factor);
    }
    let wind_state = ecosystem.wind_state.as_mut().unwrap();
    wind_state.high_freq_convolution = high_freq_terrain;

    // low frequency blur
    let mut img = Img::new(
        argb_heights,
        constants::AREA_SIDE_LENGTH,
        constants::AREA_SIDE_LENGTH,
    );
    blur_argb(&mut img.as_mut(), LOW_FREQ_KERNEL_RADIUS);

    // convert back to f32 heights
    let mut low_freq_terrain = [0.0; constants::NUM_CELLS];
    for (i, pixel) in img.buf().iter().enumerate() {
        low_freq_terrain[i] = (*pixel as u8) as f32 * (1.0 / norm_factor);
    }
    let wind_state = ecosystem.wind_state.as_mut().unwrap();
    wind_state.low_freq_convolution = low_freq_terrain;
}

// gradient at this point
pub(crate) fn get_slope_at_point_blurred(
    ecosystem: &Ecosystem,
    index: CellIndex,
    high_freq: bool,
) -> (f32, Vector2<i32>) {
    // negative slope between points means point 1 is lower than point 2
    // looking for largest slope
    let neighbors = Cell::get_neighbors(&index);
    let mut max_slope = f32::MIN;
    let mut dir = (0, 0);
    for neighbor_index in neighbors.as_array().into_iter().flatten() {
        let slope = get_slope_between_points_blurred(ecosystem, index, neighbor_index, high_freq);
        if slope > max_slope {
            max_slope = slope;
            dir = (
                index.x as i32 - neighbor_index.x as i32,
                index.y as i32 - neighbor_index.y as i32,
            );
        }
    }
    (max_slope, Vector2::new(dir.0, dir.1))
}

pub(crate) fn get_slope_between_points_blurred(
    ecosystem: &Ecosystem,
    i1: CellIndex,
    i2: CellIndex,
    high_freq: bool,
) -> f32 {
    //s(q)=(E(p)−E(q))/∥p−q∥
    let wind_state = ecosystem.wind_state.as_ref().unwrap();
    let flat_index_1 = i1.x + i1.y * constants::AREA_SIDE_LENGTH;
    let flat_index_2 = i2.x + i2.y * constants::AREA_SIDE_LENGTH;
    let (height_1, height_2) = if high_freq {
        (
            wind_state.high_freq_convolution[flat_index_1],
            wind_state.high_freq_convolution[flat_index_2],
        )
    } else {
        (
            wind_state.low_freq_convolution[flat_index_1],
            wind_state.low_freq_convolution[flat_index_2],
        )
    };
    let pos_1 = ecosystem.get_position_of_cell(&i1);
    let pos_2 = ecosystem.get_position_of_cell(&i2);
    (height_1 - height_2) / (pos_1 - pos_2).norm()
}

fn get_local_sand_strength(wind_strength: f32, wind_shadowing: f32) -> f32 {
    wind_strength * (1.0 - wind_shadowing)
}

fn get_local_sand_strength_vec(wind_vec: Vector2<f32>, wind_shadowing: f32) -> Vector2<f32> {
    wind_vec * (1.0 - wind_shadowing)
}

fn get_wind_shadowing(ecosystem: &Ecosystem, index: CellIndex, wind_angle: f32) -> f32 {
    // wind shadowing
    // cells are shadowed under 15° up to 10 cells away
    let dir = get_wind_direction_vector(wind_angle);

    let mut steepest_slope = 0.0;
    for i in 0..10 {
        let target_x = index.x as i32 + (dir.x * i as f32) as i32;
        let target_y = index.y as i32 + (dir.y * i as f32) as i32;

        // check boundary
        if target_x < 0
            || target_x >= constants::AREA_SIDE_LENGTH as i32
            || target_y < 0
            || target_y >= constants::AREA_SIDE_LENGTH as i32
        {
            break;
        }
        // check slope
        let slope = ecosystem
            .get_slope_between_points(index, CellIndex::new(target_x as usize, target_y as usize));
        if slope < steepest_slope {
            steepest_slope = slope;
        }
    }
    if steepest_slope < 0.0 {
        let angle = f32::atan(steepest_slope).to_degrees();
        let theta_min = -10.0;
        let theta_max = -15.0;
        f32::min((angle - theta_min) / (theta_max - theta_min), 1.0)
    } else {
        0.0
    }
}

fn get_saltation_distance(wind_strength: f32) -> f32 {
    wind_strength * SALTATION_DISTANCE_FACTOR
}

// returns probability from 0-1 of sand slab bouncing when landing at the given index
fn get_bounce_probability(ecosystem: &Ecosystem, index: CellIndex, wind_shadowing: f32) -> f32 {
    //β = σ(q)+ fS(S(q,t))+ fV(V(q,t))
    let cell = &ecosystem[index];
    let sand_height = cell.get_sand_height();
    let fs = if sand_height == 0.0 { 0.4 } else { 0.6 };

    // average density of three types of vegetation
    let vegetation_density = f32::min(cell.estimate_vegetation_density() / 3.0, 1.0);
    let fv = 1.0 - vegetation_density;

    // clamp to 0 to 1
    f32::max(f32::min(wind_shadowing + fs + fv, 1.0), 0.0)
}

// get the two cells that have the steepest slope from the given one, i.e. are lowest
fn get_two_steepest_neighbors(
    ecosystem: &Ecosystem,
    index: CellIndex,
) -> (Option<(f32, CellIndex)>, Option<(f32, CellIndex)>) {
    let neighbors = Cell::get_neighbors(&index);
    let mut slopes: Vec<(f32, CellIndex)> = vec![];
    for neighbor_index in neighbors.as_array().into_iter().flatten() {
        let slope = ecosystem.get_slope_between_points(index, neighbor_index);
        if !slope.is_nan() && slope >= 0.0 {
            slopes.push((slope, neighbor_index));
        }
    }
    slopes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    if slopes.len() >= 2 {
        (Some(slopes[0]), Some(slopes[1]))
    } else if slopes.len() == 1 {
        (Some(slopes[0]), None)
    } else {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        get_bounce_probability, get_local_sand_strength, get_two_steepest_neighbors,
        perform_reptation, WindRose, CARRYING_CAPACITY,
    };
    use crate::{
        constants,
        ecology::{Bushes, CellIndex, Ecosystem, Grasses, Trees},
        events::wind::get_wind_shadowing,
    };
    use float_cmp::approx_eq;

    #[test]
    fn test_get_local_sand_strength() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(3, 3);
        let wind_angle = 270.0;
        let wind_shadowing = get_wind_shadowing(&ecosystem, index, wind_angle);
        let wind_strength = get_local_sand_strength(constants::WIND_STRENGTH, wind_shadowing);
        assert_eq!(wind_strength, constants::WIND_STRENGTH);

        // adding small hill to east should not affect strength
        ecosystem[CellIndex::new(4, 3)].add_bedrock(2.0);
        let wind_shadowing = get_wind_shadowing(&ecosystem, index, wind_angle);
        let wind_strength = get_local_sand_strength(constants::WIND_STRENGTH, wind_shadowing);
        assert_eq!(wind_strength, constants::WIND_STRENGTH);

        // adding large hill to west should decrease wind strength
        ecosystem[CellIndex::new(2, 3)].add_bedrock(1.0);
        let wind_shadowing = get_wind_shadowing(&ecosystem, index, wind_angle);
        let wind_strength = get_local_sand_strength(constants::WIND_STRENGTH, wind_shadowing);
        assert_eq!(wind_strength, 0.0 * constants::WIND_STRENGTH);

        // make hill smaller
        ecosystem[CellIndex::new(2, 3)].remove_bedrock(0.8);
        let wind_shadowing = get_wind_shadowing(&ecosystem, index, wind_angle);
        let wind_strength = get_local_sand_strength(constants::WIND_STRENGTH, wind_shadowing);
        let expected = (1.0 - 0.22) * constants::WIND_STRENGTH;
        assert!(
            approx_eq!(f32, wind_strength, expected, epsilon = 0.1),
            "Expected {expected}, actual {wind_strength}"
        );

        // add taller hill further away
        ecosystem[CellIndex::new(1, 3)].add_bedrock(0.5);
        let wind_shadowing = get_wind_shadowing(&ecosystem, index, wind_angle);
        let wind_strength = get_local_sand_strength(constants::WIND_STRENGTH, wind_shadowing);
        let expected = (1.0 - 0.72) * constants::WIND_STRENGTH;
        assert!(
            approx_eq!(f32, wind_strength, expected, epsilon = 0.1),
            "Expected {expected}, actual {wind_strength}"
        );

        // check boundaries
        let wind_shadowing = get_wind_shadowing(&ecosystem, CellIndex::new(0, 0), wind_angle);
        let wind_strength = get_local_sand_strength(constants::WIND_STRENGTH, wind_shadowing);
        assert_eq!(wind_strength, constants::WIND_STRENGTH);
    }

    #[test]
    fn test_get_bounce_probability() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(2, 2);
        let prob = get_bounce_probability(&ecosystem, index, 0.0);
        assert_eq!(prob, 1.0);

        // vegetation reduces bouncing
        let cell = &mut ecosystem[index];
        cell.trees = Some(Trees {
            number_of_plants: 2,
            plant_height_sum: 45.0,
            plant_age_sum: 40.0,
        });

        cell.bushes = Some(Bushes {
            number_of_plants: 20,
            plant_height_sum: 70.0,
            plant_age_sum: 40.0,
        });

        cell.grasses = Some(Grasses {
            coverage_density: 1.0,
        });
        let prob = get_bounce_probability(&ecosystem, index, 0.0);
        assert_eq!(prob, 0.4);

        // sand presence increases bouncing
        let cell = &mut ecosystem[index];
        cell.add_sand(0.5);
        let prob = get_bounce_probability(&ecosystem, index, 0.0);
        assert_eq!(prob, 0.6);

        // wind shadowing increases bouncing
        let prob = get_bounce_probability(&ecosystem, index, 0.2);
        assert_eq!(prob, 0.8);
    }

    #[test]
    fn test_get_two_steepest_neighbors() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(2, 2);

        // add some terrain variation
        ecosystem[CellIndex::new(2, 2)].add_sand(1.0);
        ecosystem[CellIndex::new(2, 2)].remove_bedrock(1.0);
        ecosystem[CellIndex::new(1, 1)].add_sand(1.0);
        ecosystem[CellIndex::new(1, 3)].add_sand(2.0);
        ecosystem[CellIndex::new(3, 2)].remove_bedrock(2.0);
        ecosystem[CellIndex::new(2, 1)].remove_bedrock(1.0);

        let (n1, n2) = get_two_steepest_neighbors(&ecosystem, index);
        assert_eq!(n1.unwrap().1, CellIndex::new(3, 2));
        assert_eq!(n2.unwrap().1, CellIndex::new(2, 1));
    }

    #[test]
    fn test_perform_reptation() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(2, 2);

        // add some terrain variation
        ecosystem[CellIndex::new(2, 2)].add_sand(1.0);
        ecosystem[CellIndex::new(2, 2)].remove_bedrock(1.0);
        ecosystem[CellIndex::new(1, 1)].add_sand(1.0);
        ecosystem[CellIndex::new(1, 3)].add_sand(2.0);
        ecosystem[CellIndex::new(3, 2)].remove_bedrock(2.0);
        ecosystem[CellIndex::new(2, 1)].remove_bedrock(1.0);

        perform_reptation(&mut ecosystem, index, CARRYING_CAPACITY);
        // slope1 = 0.894
        // slope2 = 0.707
        // ratio = .558
        assert_eq!(ecosystem[index].get_sand_height(), 1.0 - CARRYING_CAPACITY);
        let expected = 0.558 * CARRYING_CAPACITY;
        let actual = ecosystem[CellIndex::new(3, 2)].get_sand_height();
        assert!(
            approx_eq!(f32, actual, expected, epsilon = 0.01),
            "Expected {expected}, actual {actual}"
        );

        let expected = (1.0 - 0.558) * CARRYING_CAPACITY;
        let actual = ecosystem[CellIndex::new(2, 1)].get_sand_height();
        assert!(
            approx_eq!(f32, actual, expected, epsilon = 0.01),
            "Expected {expected}, actual {actual}"
        );
        assert!(
            approx_eq!(f32, actual, expected, epsilon = 0.01),
            "Expected {expected}, actual {actual}"
        );
    }

    #[test]
    fn test_wind_rose() {
        let wind_rose = WindRose::new(0.0, 10.0, 10.0);
        let mut expected_min_speed = [0.0; 8];
        expected_min_speed[0] = 10.0;
        assert_eq!(wind_rose.min_speed, expected_min_speed);
        assert_eq!(wind_rose.max_speed, expected_min_speed);
        assert_eq!(wind_rose.weights, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_sample_wind() {
        let mut wind_rose = WindRose::new(0.0, 10.0, 10.0);
        let (dir, str) = wind_rose.sample_wind();
        assert_eq!(dir, 0.0);
        assert_eq!(str, 10.0);

        wind_rose.max_speed[0] = 15.0;
        let (dir, str) = wind_rose.sample_wind();
        assert_eq!(dir, 0.0);
        assert!((10.0..=15.0).contains(&str));

        wind_rose.min_speed[4] = 5.0;
        wind_rose.max_speed[4] = 10.0;
        wind_rose.weights[4] = 1.0;

        let (dir, str) = wind_rose.sample_wind();
        assert!(dir == 0.0 || dir == 180.0);
        if dir == 0.0 {
            assert!((10.0..=15.0).contains(&str));
        } else {
            assert!((5.0..=10.0).contains(&str));
        }
    }
}

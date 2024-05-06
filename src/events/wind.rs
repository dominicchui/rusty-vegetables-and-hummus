use nalgebra::Vector2;
use rand::Rng;

use crate::{constants, ecology::{Cell, CellIndex, Ecosystem}};

use super::Events;

const SALTATION_DISTANCE_FACTOR: f32 = 0.5;
const CARRYING_CAPACITY: f32 = 0.1; // each wind event can carry this much height of sand
const REPTATION_HEIGHT: f32 = 0.1;

impl Events {
    pub(crate) fn apply_wind_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        
        // Saltation
        // 1) lift a small amount of sand
        let cell = &mut ecosystem[index];
        let sand_height = cell.get_sand_height();
        let moved_height = f32::min(CARRYING_CAPACITY, sand_height);
        cell.remove_sand(moved_height);

        // 2) transport sand to target cell
        let local_strength = get_local_sand_strength(ecosystem, index);
        let distance = get_saltation_distance(local_strength);
        let direction = get_wind_direction_vector();
        let target_vec = direction * distance;
        let target_x = index.x as i32 + target_vec.x as i32;
        let target_y = index.y as i32 + target_vec.y as i32;

        // check bounds
        if target_x < 0|| target_x >= constants::AREA_SIDE_LENGTH as i32 || target_y < 0 || target_y >= constants::AREA_SIDE_LENGTH as i32 {
            return None;
        }

        let target_index = CellIndex::new(target_x as usize, target_y as usize);
        let target = &mut ecosystem[target_index];
        target.add_sand(moved_height);

        // 3) on landing, sand can bounce or be deposited
        let bounce_probability = get_bounce_probability(ecosystem, index, get_wind_shadowing(ecosystem, index));
        let mut rng = rand::thread_rng();
        let rand: f32 = rng.gen();

        let result = if rand > bounce_probability {
            // bounce
            Some((Events::Wind, target_index))
        } else {
            // deposited
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
            let reptation_ratio = 
                if slope_1 + slope_2 == 0.0 {
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

fn get_wind_direction_vector() -> Vector2<f32> {
    let wind_dir = constants::WIND_DIRECTION.to_radians();
    let x = wind_dir.sin();
    let y = wind_dir.cos();
    Vector2::new(x,y).normalize()
}

fn get_local_sand_strength(ecosystem: &Ecosystem, index: CellIndex) -> f32 {
    let shadowing = get_wind_shadowing(ecosystem, index);
    constants::WIND_STRENGTH * (1.0 - shadowing)
}

fn get_wind_shadowing(ecosystem: &Ecosystem, index: CellIndex) -> f32 {
// wind shadowing
    // cells are shadowed under 15° up to 10 cells away
    let dir = get_wind_direction_vector();

    let mut steepest_slope = 0.0;
    for i in 0..10 {
        let target_x = index.x as i32 + (dir.x * i as f32) as i32;
        let target_y = index.y as i32 + (dir.y * i as f32) as i32;

        // check boundary
        if target_x < 0|| target_x >= constants::AREA_SIDE_LENGTH as i32 || target_y < 0 || target_y >= constants::AREA_SIDE_LENGTH as i32 {
            break;
        }
        // check slope
        let slope = ecosystem.get_slope_between_points(index, CellIndex::new(target_x as usize, target_y as usize));
        if slope < steepest_slope {
            steepest_slope = slope;
        }
    }
    if steepest_slope < 0.0 {
        let angle = f32::atan(steepest_slope).to_degrees();
        let theta_min = -10.0;
        let theta_max = -15.0;
        let shadowing = f32::min((angle - theta_min) / (theta_max - theta_min), 1.0);
        shadowing
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
    let fs = if sand_height == 0.0 {
        0.4
    } else {
        0.6
    };

    // average density of three types of vegetation
    let vegetation_density = f32::min(cell.estimate_vegetation_density() / 3.0, 1.0);
    let fv = 1.0 - vegetation_density;

    // clamp to 0 to 1
    f32::max(f32::min(wind_shadowing + fs + fv, 1.0), 0.0)
}

// get the two cells that have the steepest slope from the given one, i.e. are lowest
fn get_two_steepest_neighbors(ecosystem: &Ecosystem, index: CellIndex) -> (Option<(f32, CellIndex)>, Option<(f32, CellIndex)>) {
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
    use float_cmp::approx_eq;
    use crate::{constants, ecology::{Bushes, CellIndex, Ecosystem, Grasses, Trees}};
    use super::{get_bounce_probability, get_local_sand_strength, get_two_steepest_neighbors, perform_reptation, CARRYING_CAPACITY};


    #[test]
    fn test_get_local_sand_strength() {
        let mut ecosystem = Ecosystem::init();
        let wind_strength = get_local_sand_strength(&ecosystem, CellIndex::new(3,3));
        assert_eq!(wind_strength, constants::WIND_STRENGTH);

        // adding small hill to east should not affect strength
        ecosystem[CellIndex::new(4,3)].add_bedrock(2.0);
        let wind_strength = get_local_sand_strength(&ecosystem, CellIndex::new(3,3));
        assert_eq!(wind_strength, constants::WIND_STRENGTH);

        // adding large hill to west should decrease wind strength
        ecosystem[CellIndex::new(2,3)].add_bedrock(1.0);
        let wind_strength = get_local_sand_strength(&ecosystem, CellIndex::new(3,3));
        assert_eq!(wind_strength, 0.0 * constants::WIND_STRENGTH);

        // make hill smaller
        ecosystem[CellIndex::new(2,3)].remove_bedrock(0.8);
        let wind_strength = get_local_sand_strength(&ecosystem, CellIndex::new(3,3));
        let expected = (1.0- 0.22) * constants::WIND_STRENGTH;
        assert!(approx_eq!(f32, wind_strength, expected, epsilon=0.1), "Expected {expected}, actual {wind_strength}");

        // add taller hill further away
        ecosystem[CellIndex::new(1,3)].add_bedrock(0.5);
        let wind_strength = get_local_sand_strength(&ecosystem, CellIndex::new(3,3));
        let expected = (1.0- 0.72) * constants::WIND_STRENGTH;
        assert!(approx_eq!(f32, wind_strength, expected, epsilon=0.1), "Expected {expected}, actual {wind_strength}");

        // check boundaries
        let wind_strength = get_local_sand_strength(&ecosystem, CellIndex::new(0,0));
        assert_eq!(wind_strength, constants::WIND_STRENGTH);
    }

    #[test]
    fn test_get_bounce_probability() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(2,2);
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
        let index = CellIndex::new(2,2);

        // add some terrain variation
        ecosystem[CellIndex::new(2,2)].add_sand(1.0);
        ecosystem[CellIndex::new(2,2)].remove_bedrock(1.0);
        ecosystem[CellIndex::new(1,1)].add_sand(1.0);
        ecosystem[CellIndex::new(1,3)].add_sand(2.0);
        ecosystem[CellIndex::new(3,2)].remove_bedrock(2.0);
        ecosystem[CellIndex::new(2,1)].remove_bedrock(1.0);

        let (n1, n2) = get_two_steepest_neighbors(&ecosystem, index);
        assert_eq!(n1.unwrap().1, CellIndex::new(3,2));
        assert_eq!(n2.unwrap().1, CellIndex::new(2,1));
    }

    #[test]
    fn test_perform_reptation() {
        let mut ecosystem = Ecosystem::init();
        let index = CellIndex::new(2,2);

        // add some terrain variation
        ecosystem[CellIndex::new(2,2)].add_sand(1.0);
        ecosystem[CellIndex::new(2,2)].remove_bedrock(1.0);
        ecosystem[CellIndex::new(1,1)].add_sand(1.0);
        ecosystem[CellIndex::new(1,3)].add_sand(2.0);
        ecosystem[CellIndex::new(3,2)].remove_bedrock(2.0);
        ecosystem[CellIndex::new(2,1)].remove_bedrock(1.0);

        perform_reptation(&mut ecosystem, index, CARRYING_CAPACITY);
        // slope1 = 0.894
        // slope2 = 0.707
        // ratio = .558
        assert_eq!(ecosystem[index].get_sand_height(), 1.0 - CARRYING_CAPACITY);
        let expected = 0.558 * CARRYING_CAPACITY;
        let actual = ecosystem[CellIndex::new(3,2)].get_sand_height();
        assert!(approx_eq!(f32, actual, expected, epsilon = 0.01), "Expected {expected}, actual {actual}");

        let expected = (1.0 - 0.558) * CARRYING_CAPACITY;
        let actual = ecosystem[CellIndex::new(2,1)].get_sand_height();
        assert!(approx_eq!(f32, actual, expected, epsilon = 0.01), "Expected {expected}, actual {actual}");
        assert!(approx_eq!(f32, actual, expected, epsilon = 0.01), "Expected {expected}, actual {actual}");


    }
}
use super::Events;
use crate::{
    constants,
    ecology::{Cell, CellIndex, Ecosystem},
};
use rand::Rng;
use std::collections::HashMap;

impl Events {
    pub(crate) fn apply_rock_slide_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let mut critical_neighbors: HashMap<CellIndex, f32> = HashMap::new();
        let neighbors = Cell::get_neighbors(&index);
        for neighbor_index in neighbors.as_array().into_iter().flatten() {
            let slope = ecosystem.get_slope_between_points(index, neighbor_index);
            let angle = Ecosystem::get_angle(slope);
            if angle >= constants::CRITICAL_ANGLE_ROCK {
                critical_neighbors.insert(neighbor_index, slope);
            }
        }
        // if current cell does not have a slope of at least the critical angle, no slide and no propagation
        if critical_neighbors.is_empty() {
            return None;
        } else {
            // else randomly select neighbor weighted by slope
            let mut neighbor_probabilities: HashMap<CellIndex, f32> = HashMap::new();
            let slope_sum: f32 = critical_neighbors.values().sum();
            for (neighbor, slope) in critical_neighbors {
                let prob = slope / slope_sum;
                neighbor_probabilities.insert(neighbor, prob);
            }
            let mut rng = rand::thread_rng();
            let mut rand: f32 = rng.gen();
            for (neighbor, prob) in neighbor_probabilities {
                rand -= prob;
                if rand < 0.0 {
                    // to propagate, reduce appropriate amount of material and move it to neighbor
                    let rock_height =
                        Events::compute_rock_height_to_slide(ecosystem, index, neighbor);
                    let cell = &mut ecosystem[index];
                    cell.remove_rocks(rock_height);

                    let neighbor_cell = &mut ecosystem[neighbor];
                    neighbor_cell.add_rocks(rock_height);

                    return Some((Events::RockSlide, neighbor));
                }
            }
        }
        None
    }

    fn compute_rock_height_to_slide(
        ecosystem: &Ecosystem,
        origin: CellIndex,
        target: CellIndex,
    ) -> f32 {
        let cell = &ecosystem[origin];
        if let Some(rock) = &cell.rock {
            let rock_height = rock.height;
            let origin_pos = ecosystem.get_position_of_cell(&origin);
            let target_pos = ecosystem.get_position_of_cell(&target);
            let ideal_height = Events::compute_ideal_slide_height(
                origin_pos,
                target_pos,
                constants::CRITICAL_ANGLE_ROCK,
            );

            let non_rock_height = cell.get_height() - rock_height;

            // simplifying assumption: half of the excess slides away
            if non_rock_height >= ideal_height {
                rock_height / 2.0
            } else {
                ((non_rock_height + rock_height) - ideal_height) / 2.0
            }
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecology::{CellIndex, Ecosystem},
        events::Events,
    };
    use float_cmp::approx_eq;

    #[test]
    fn test_apply_rock_slide_event() {
        let mut ecosystem = Ecosystem::init();
        let center = &mut ecosystem[CellIndex::new(3, 3)];
        let bedrock = &mut center.bedrock.as_mut().unwrap();
        bedrock.height = 0.0;
        center.add_rocks(1.0);

        let up = &mut ecosystem[CellIndex::new(3, 2)];
        let bedrock = &mut up.bedrock.as_mut().unwrap();
        bedrock.height = 0.0;

        let propagation = Events::apply_rock_slide_event(&mut ecosystem, CellIndex::new(3, 3));

        assert!(propagation.is_some());
        let (event, index) = propagation.unwrap();
        assert_eq!(event, Events::RockSlide);
        assert_eq!(index, CellIndex::new(3, 2));

        let center = &mut ecosystem[CellIndex::new(3, 3)];
        let rock_height = center.rock.as_ref().unwrap().height;
        let expected = 0.916;
        assert!(
            approx_eq!(f32, rock_height, expected, epsilon = 0.01),
            "Expected {expected}, actual {rock_height}"
        );

        let up = &mut ecosystem[CellIndex::new(3, 2)];
        let rock_height = up.rock.as_ref().unwrap().height;
        let expected = 0.084;
        assert!(
            approx_eq!(f32, rock_height, expected, epsilon = 0.01),
            "Expected {expected}, actual {rock_height}"
        );
    }
}

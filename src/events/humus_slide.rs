use super::Events;
use crate::{
    constants,
    ecology::{Cell, CellIndex, Ecosystem},
};
use rand::Rng;
use std::collections::HashMap;

impl Events {
    pub(crate) fn apply_humus_slide_event(
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<(Events, CellIndex)> {
        let mut critical_neighbors: HashMap<CellIndex, f32> = HashMap::new();
        let neighbors = Cell::get_neighbors(&index);
        for neighbor_index in neighbors.as_array().into_iter().flatten() {
            let slope = ecosystem.get_slope_between_points(index, neighbor_index);
            let angle = Ecosystem::get_angle(slope);
            if angle >= constants::CRITICAL_ANGLE_HUMUS {
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
                    let humus_height =
                        Events::compute_humus_height_to_slide(ecosystem, index, neighbor);
                    // println!("Humus of height {humus_height} sliding from {index} to {neighbor}");
                    let cell = &mut ecosystem[index];
                    cell.remove_humus(humus_height);

                    let neighbor_cell = &mut ecosystem[neighbor];
                    neighbor_cell.add_humus(humus_height);

                    return Some((Events::HumusSlide, neighbor));
                }
            }
        }
        None
    }

    fn compute_humus_height_to_slide(
        ecosystem: &Ecosystem,
        origin: CellIndex,
        target: CellIndex,
    ) -> f32 {
        let cell = &ecosystem[origin];
        let humus_height = cell.get_humus_height();
        if humus_height > 0.0 {
            let origin_pos = ecosystem.get_position_of_cell(&origin);
            let target_pos = ecosystem.get_position_of_cell(&target);
            let ideal_height = Events::compute_ideal_slide_height(
                origin_pos,
                target_pos,
                constants::CRITICAL_ANGLE_HUMUS,
            );

            let non_humus_height = cell.get_height() - humus_height;

            // simplifying assumption: half of the excess slides away
            if non_humus_height >= ideal_height {
                humus_height / 2.0
            } else {
                ((non_humus_height + humus_height) - ideal_height) / 2.0
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
    fn test_apply_humus_slide_event() {
        let mut ecosystem = Ecosystem::init();
        let center = &mut ecosystem[CellIndex::new(3, 3)];
        center.set_height_of_bedrock(0.0);
        center.add_humus(1.0);

        let up = &mut ecosystem[CellIndex::new(3, 2)];
        up.set_height_of_bedrock(0.0);

        let propagation = Events::apply_humus_slide_event(&mut ecosystem, CellIndex::new(3, 3));

        assert!(propagation.is_some());
        let (event, index) = propagation.unwrap();
        assert_eq!(event, Events::HumusSlide);
        assert_eq!(index, CellIndex::new(3, 2));

        let center = &mut ecosystem[CellIndex::new(3, 3)];
        let humus_height = center.get_humus_height();
        let expected = 0.916;
        assert!(
            approx_eq!(f32, humus_height, expected, epsilon = 0.01),
            "Expected {expected}, actual {humus_height}"
        );

        let up = &mut ecosystem[CellIndex::new(3, 2)];
        let humus_height = up.get_humus_height();
        let expected = 0.084;
        assert!(
            approx_eq!(f32, humus_height, expected, epsilon = 0.01),
            "Expected {expected}, actual {humus_height}"
        );
    }
}

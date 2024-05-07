use rand::Rng;

use crate::{
    constants,
    ecology::{CellIndex, Ecosystem, Trees},
    events::wind::{self, WindRose, WindState},
};

impl Ecosystem {
    pub fn init_standard() -> Self {
        let mut ecosystem = Self::init();

        // add terrain variation
        let height = 2.0;
        let cell = &mut ecosystem[CellIndex::new(1, 0)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(2, 0)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(3, 0)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(4, 0)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(5, 0)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(1, 1)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(2, 1)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(3, 1)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(4, 1)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(5, 1)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(1, 2)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(5, 2)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(1, 3)];
        cell.add_bedrock(height);
        let cell = &mut ecosystem[CellIndex::new(5, 3)];
        cell.add_bedrock(height);
        ecosystem.update_tets();

        // add humus
        let mut humus_heights = [[0.0; constants::AREA_SIDE_LENGTH]; constants::AREA_SIDE_LENGTH];
        for (i, heights) in humus_heights.iter_mut().enumerate() {
            for (j, height) in heights.iter_mut().enumerate() {
                let index = CellIndex::new(i, j);
                let slope = ecosystem.get_slope_at_point(index);
                let humus_height = Self::get_initial_humus_height(slope);
                *height = humus_height;
            }
        }
        for (i, heights) in humus_heights.iter().enumerate() {
            for (j, height) in heights.iter().enumerate() {
                let index = CellIndex::new(i, j);
                let cell = &mut ecosystem[index];
                cell.add_humus(*height);
            }
        }

        ecosystem
    }

    pub fn init_with_heights(
        heights: [f32; constants::AREA_SIDE_LENGTH * constants::AREA_SIDE_LENGTH],
    ) -> Self {
        let mut ecosystem = Self::init();
        for (index, height) in heights.iter().enumerate() {
            let j = index / constants::AREA_SIDE_LENGTH;
            let i = index - j * constants::AREA_SIDE_LENGTH;
            let cell = &mut ecosystem[CellIndex::new(i, j)];
            cell.add_bedrock(*height);
        }
        ecosystem.update_tets();

        // add humus
        // let mut humus_heights = [[0.0; constants::AREA_SIDE_LENGTH]; constants::AREA_SIDE_LENGTH];
        // for (i, heights) in humus_heights.iter_mut().enumerate() {
        //     for (j, height) in heights.iter_mut().enumerate() {
        //         let index = CellIndex::new(i, j);
        //         let slope = ecosystem.get_slope_at_point(index);
        //         let humus_height = Self::get_initial_humus_height(slope);
        //         *height = humus_height;
        //     }
        // }
        // for (i, heights) in humus_heights.iter().enumerate() {
        //     for (j, height) in heights.iter().enumerate() {
        //         let index = CellIndex::new(i, j);
        //         let cell = &mut ecosystem[index];
        //         cell.add_humus(*height);
        //     }
        // }

        // add sand for fun
        Self::add_blanket_sand(&mut ecosystem, 1.0);

        ecosystem
    }

    pub fn init_test() -> Self {
        let mut ecosystem = Self::init();
        let c_i = 2;

        let trees = Trees {
            number_of_plants: 2,
            plant_height_sum: 20.0,
            plant_age_sum: 40.0,
        };

        let center = &mut ecosystem[CellIndex::new(c_i, c_i)];
        center.add_bedrock(2.0);
        center.add_humus(0.5);
        // center.soil_moisture = 1.8E5;
        // center.trees = Some(trees.clone());

        let up = &mut ecosystem[CellIndex::new(c_i, c_i - 1)];
        up.add_bedrock(1.0);
        up.add_humus(0.5);
        // up.soil_moisture = 1.8E5;
        // up.trees = Some(trees.clone());

        let down = &mut ecosystem[CellIndex::new(c_i, c_i + 1)];
        down.add_bedrock(1.0);
        down.add_humus(0.5);
        // down.soil_moisture = 1.8E5;
        // down.trees = Some(trees.clone());

        let left = &mut ecosystem[CellIndex::new(c_i - 1, c_i)];
        left.add_bedrock(1.0);
        left.add_humus(0.5);
        left.soil_moisture = 1.8E5;
        left.trees = Some(trees.clone());

        let right = &mut ecosystem[CellIndex::new(c_i + 1, c_i)];
        right.add_bedrock(1.0);
        right.add_humus(0.5);
        // right.soil_moisture = 1.8E5;
        // right.trees = Some(trees.clone());

        let up_left = &mut ecosystem[CellIndex::new(c_i - 1, c_i - 1)];
        up_left.add_bedrock(1.0);
        up_left.add_humus(0.5);
        up_left.soil_moisture = 1.8E5;
        up_left.trees = Some(trees.clone());

        let up_right = &mut ecosystem[CellIndex::new(c_i + 1, c_i - 1)];
        up_right.add_bedrock(1.0);
        up_right.add_humus(0.5);
        // up_right.soil_moisture = 1.8E5;
        // up_right.trees = Some(trees.clone());

        let down_left = &mut ecosystem[CellIndex::new(c_i - 1, c_i + 1)];
        down_left.add_bedrock(1.0);
        down_left.add_humus(0.5);
        // down_left.soil_moisture = 1.8E5;
        down_left.trees = Some(trees.clone());

        let down_right = &mut ecosystem[CellIndex::new(c_i + 1, c_i + 1)];
        down_right.add_bedrock(1.0);
        down_right.add_humus(0.5);
        // down_right.soil_moisture = 1.8E5;
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

    pub fn init_sand() -> Self {
        let sand_height = 0.2;
        let mut ecosystem = Self::init();
        Self::add_blanket_sand(&mut ecosystem, sand_height);

        // set up wind rose
        let mut wind_rose = WindRose::new(45.0, 10.0, 15.0);
        // wind_rose.update_wind(90.0, 10.0, 15.0, 1.0);
        wind_rose.update_wind(180.0, 10.0, 15.0, 1.0);
        wind_rose.update_wind(270.0, 10.0, 15.0, 1.0);
        let mut wind_state = WindState::new();
        wind_state.wind_rose = wind_rose;
        ecosystem.wind_state = Some(wind_state);

        // // randomly perturbate bedrock
        // let mut rng = rand::thread_rng();
        // for i in 0..constants::AREA_SIDE_LENGTH {
        //     for j in 0..constants::AREA_SIDE_LENGTH {
        //         let rand: f32 = rng.gen();
        //         ecosystem[CellIndex::new(i,j)].add_bedrock(rand);
        //     }
        // }

        // // add obstacles
        // for i in 130..160 {
        //     for j in 80..100 {
        //         ecosystem[CellIndex::new(i, j)].add_bedrock(10.0);
        //     }
        // }
        // for i in 90..120 {
        //     for j in 110..130 {
        //         ecosystem[CellIndex::new(i, j)].add_bedrock(10.0);
        //     }
        // }
        // for i in 50..80 {
        //     for j in 140..160 {
        //         ecosystem[CellIndex::new(i, j)].add_bedrock(10.0);
        //     }
        // }

        ecosystem
    }

    fn add_blanket_sand(ecosystem: &mut Ecosystem, height: f32) {
        for i in 0..constants::AREA_SIDE_LENGTH {
            for j in 0..constants::AREA_SIDE_LENGTH {
                ecosystem[CellIndex::new(i, j)].add_sand(height);
            }
        }
    }

    fn get_initial_humus_height(slope: f32) -> f32 {
        // a 30° slope should have about half the humus as a 0° slope
        constants::DEFAULT_HUMUS_HEIGHT
            * f32::powf(std::f32::consts::E, -(slope * slope) / (1.0 / 3.0))
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;

    use crate::{constants, ecology::Ecosystem};

    #[test]
    fn test_get_initial_humus_height() {
        let height = Ecosystem::get_initial_humus_height(0.0);
        let expected = constants::DEFAULT_HUMUS_HEIGHT * 1.0;
        assert_eq!(height, expected);

        // half humus on 30° slope
        let height = Ecosystem::get_initial_humus_height(f32::sin(f32::to_radians(30.0)));
        let expected = constants::DEFAULT_HUMUS_HEIGHT * 0.5;
        assert!(
            approx_eq!(f32, height, expected, epsilon = 0.03),
            "Expected {expected}, actual{height}"
        );

        // about zero humus on 0˚slope
        let height = Ecosystem::get_initial_humus_height(1.0);
        let expected = constants::DEFAULT_HUMUS_HEIGHT * 0.0;
        assert!(
            approx_eq!(f32, height, expected, epsilon = 0.03),
            "Expected {expected}, actual{height}"
        );
    }
}

use nalgebra::{Vector3, Vector4};

use crate::constants;

use super::{Cell, CellIndex, Ecosystem};

// a three dimensional rectangle representing the two planes constructed from a cell index and its neighboring three points
// for index (x,y), rectangle is formed with (x,y), (x+1, y), (x, y+1), and (x+1, y+1)
// planes are (x,y), (x+1, y), (x, y+1) and (x+1, y), (x, y+1), (x+1, y+1)
struct CellTetrahedron {
    coordinates: Vector4<Vector3<f32>>,
    top_left: CellIndex,
    top_right: CellIndex,
    bottom_left: CellIndex,
    bottom_right: CellIndex,
    is_dirty: bool,
    normal_one: Vector3<f32>,
    normal_two: Vector3<f32>,
    scalar_one: f32,
    scalar_two: f32,
}

impl CellTetrahedron {
    pub(crate) fn new(index: CellIndex, ecosystem: &Ecosystem) -> Self {
        let mut tet = CellTetrahedron {
            coordinates: Vector4::zeros(),
            top_left: index,
            top_right: CellIndex::new(index.x + 1, index.y),
            bottom_left: CellIndex::new(index.x, index.y + 1),
            bottom_right: CellIndex::new(index.x + 1, index.y + 1),
            is_dirty: true,
            normal_one: Vector3::zeros(),
            normal_two: Vector3::zeros(),
            scalar_one: 0.0,
            scalar_two: 0.0,
        };
        tet.update(ecosystem);
        tet
    }

    pub(crate) fn update(&mut self, ecosystem: &Ecosystem) {
        if self.is_dirty {
            let height = ecosystem[self.top_left].get_height();
            let a = Vector3::new(self.top_left.x as f32, self.top_left.y as f32, height);
            self.coordinates[0] = a;

            let height = ecosystem[self.top_right].get_height();
            let b = Vector3::new(self.top_right.x as f32, self.top_right.y as f32, height);
            self.coordinates[1] = b;

            let height = ecosystem[self.bottom_left].get_height();
            let c = Vector3::new(self.bottom_left.x as f32, self.bottom_left.y as f32, height);
            self.coordinates[2] = c;

            let height = ecosystem[self.bottom_right].get_height();
            let d = Vector3::new(
                self.bottom_right.x as f32,
                self.bottom_right.y as f32,
                height,
            );
            self.coordinates[3] = d;

            // compute plane definitions
            let normal_one = Cell::get_normal_of_triangle(
                ecosystem,
                self.top_left,
                self.bottom_left,
                self.top_right,
            );
            self.normal_one = normal_one;
            let scalar_one = -normal_one.dot(&a);
            self.scalar_one = scalar_one;

            let normal_two = Cell::get_normal_of_triangle(
                ecosystem,
                self.bottom_left,
                self.bottom_right,
                self.top_right,
            );
            self.normal_two = normal_two;
            let scalar_two = -normal_two.dot(&c);
            self.scalar_two = scalar_two;

            self.is_dirty = false;
        }
    }

    fn has_intersection(&self, pos: Vector3<f32>, dir: Vector3<f32>) -> bool {
        let mut has_intersect = false;
        let height_top_left = self.coordinates[0][2];
        let height_top_right = self.coordinates[1][2];
        let height_bottom_left = self.coordinates[2][2];
        let height_bottom_right = self.coordinates[3][2];
        // find intesect with plane one
        let denom = self.normal_one.dot(&dir);
        if denom != 0.0 {
            let t = -(self.normal_one.dot(&pos) + self.scalar_one) / denom;
            let intersect = pos + t * dir;
            // println!("intersect {intersect}");
            // check if intersect is in bounds
            let min_x = self.top_left.x as f32;
            let max_x = self.top_right.x as f32;
            let min_y = self.top_left.y as f32;
            let max_y = self.bottom_left.y as f32;
            let min_z = f32::min(
                height_top_left,
                f32::min(height_top_right, height_bottom_left),
            );
            let max_z = f32::max(
                height_top_left,
                f32::max(height_top_right, height_bottom_left),
            );
            // println!("min_x {min_x}, max_x {max_x}");
            // println!("min_y {min_y}, max_y {max_y}");
            // println!("min_z {min_z}, max_z {max_z}");
            if min_x < intersect.x
                && max_x > intersect.x
                && min_y < intersect.y
                && max_y > intersect.y
                && min_z < intersect.z
                && max_z > intersect.z
            {
                return true;
            }
        }
        // find intersect with plane two
        let denom = self.normal_two.dot(&dir);
        if denom != 0.0 {
            let t = -(self.normal_two.dot(&pos) + self.scalar_two) / denom;
            let intersect = pos + t * dir;
            // check if intersect is in bounds
            let min_x = self.top_left.x as f32;
            let max_x = self.top_right.x as f32;
            let min_y = self.top_left.y as f32;
            let max_y = self.bottom_left.y as f32;
            let min_z = f32::min(
                height_bottom_right,
                f32::min(height_top_right, height_bottom_left),
            );
            let max_z = f32::max(
                height_bottom_right,
                f32::max(height_top_right, height_bottom_left),
            );
            let margin = 0.001;
            if min_x < intersect.x + margin
                && max_x > intersect.x - margin
                && min_y < intersect.y + margin
                && max_y > intersect.y - margin
                && min_z < intersect.z + margin
                && max_z > intersect.z - margin
            {
                has_intersect = true;
            }
        }

        has_intersect
    }
}

impl Ecosystem {
    // estimates the illumination of the cell based on traced rays from the sun moving across the sky
    // returns average daily hours of direct sunlight
    pub(crate) fn estimate_illumination_simple(&self, index: &CellIndex, month: usize) -> f32 {
        // todo placeholder estimate
        constants::AVERAGE_SUNLIGHT_HOURS[month]
    }

    pub(crate) fn estimate_illumination_ray_traced(&self, index: &CellIndex, month: usize) -> f32 {
        // estimate illumination of given cell using rays traced from sun's position across the sky over the year

        // compute sun arc for 1st of every month
        let mut hours_of_sun = 0.0;
        for i in 0..24 {
            // for every hour, determine if sun is above horizon
            let (azimuth, elevation) = get_azimuth_and_elevation(month, i as f32);
            if elevation < 0.0 {
                continue;
            }
            // if so, trace rays to determine hours of light
        }

        // ray trace for each hour(?) of the day to estimate illumination

        // todo replace placeholder value
        0.0
    }
}

// correction between the apparent solar time and mean solar time,
// i.e. difference between sundial noon and clock noon
// https://en.wikipedia.org/wiki/Equation_of_time
fn compute_equation_of_time(month: usize) -> f32 {
    let b = ((360.0 / 365.0) * (days_since_start_of_year(month) - 81) as f32).to_radians();

    9.87 * f32::sin(2.0 * b) - 7.53 * f32::cos(b) - 1.5 * f32::sin(b)
}

// returns the number of days since the start of the year for the first day of the given month
fn days_since_start_of_year(month: usize) -> i32 {
    match month {
        0 => 0,
        1 => 31,
        2 => 59,
        3 => 90,
        4 => 120,
        5 => 151,
        6 => 181,
        7 => 212,
        8 => 243,
        9 => 273,
        10 => 304,
        11 => 334,
        _ => panic!("Month {month} is invalid"),
    }
}

// in degrees
fn get_local_standard_time_meridian() -> i32 {
    15 * constants::TIMEZONE
}

fn get_time_correction_factor(month: usize) -> f32 {
    4.0 * (constants::LONGITUDE - get_local_standard_time_meridian() as f32)
        + compute_equation_of_time(month)
}

// local time is in hours since midnight
// returns the adjusted time based on sun's position
fn get_local_solar_time(month: usize, local_time: f32) -> f32 {
    let time_correction_factor = get_time_correction_factor(month);
    local_time + time_correction_factor / 60.0
}

// converts local solar time (LST) to number of degrees which the sun moves across the sky
// hour angle is 0° at noon
fn get_hour_angle(month: usize, local_time: f32) -> f32 {
    15.0 * (get_local_solar_time(month, local_time) - 12.0)
}

fn get_declination(month: usize) -> f32 {
    let days = days_since_start_of_year(month);
    23.45 * f32::sin((360.0 / 365.0 * (days - 81) as f32).to_radians())
}

fn get_elevation(month: usize, local_time: f32) -> f32 {
    let declination = get_declination(month).to_radians();
    let hra = get_hour_angle(month, local_time).to_radians();
    let latitude = constants::LATITUDE.to_radians();
    f32::asin(declination.sin() * latitude.sin() + declination.cos() * latitude.cos() * hra.cos())
}

fn get_azimuth_and_elevation(month: usize, local_time: f32) -> (f32, f32) {
    let elevation = get_elevation(month, local_time);
    let declination = get_declination(month).to_radians();
    let hra = get_hour_angle(month, local_time).to_radians();
    let latitude = constants::LATITUDE.to_radians();
    // angle between 0-π radians
    let angle = f32::acos(
        (declination.sin() * latitude.cos() - declination.cos() * latitude.sin() * hra.cos())
            / elevation.cos(),
    );
    // convert to full 2π radians
    if local_time >= 12.0 {
        ((360.0 - angle.to_degrees()).to_radians(), elevation)
    } else {
        (angle, elevation)
    }
}

// convert from angles given in the azimuth-altitude/elevation system to x,y,z cartesian (z up)
fn convert_from_spherical_to_cartesian(azimuth: f32, elevation: f32) -> (f32, f32, f32) {
    let x = azimuth.sin() * elevation.cos();
    let y = azimuth.cos() * elevation.cos();
    let z = elevation.sin();
    (x, y, z)
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use nalgebra::Vector3;

    use crate::ecology::{
        illumination::{
            compute_equation_of_time, get_azimuth_and_elevation, get_declination, get_elevation,
            get_hour_angle,
        },
        CellIndex, Ecosystem,
    };

    use super::{convert_from_spherical_to_cartesian, CellTetrahedron};

    #[test]
    fn test_compute_equation_of_time() {
        let eot = compute_equation_of_time(0);
        let expected = -3.256234;
        assert!(
            approx_eq!(f32, eot, expected, epsilon = 0.001),
            "Expected {expected}, actual {eot}"
        );

        let eot = compute_equation_of_time(3);
        let expected = -4.66170;
        assert!(
            approx_eq!(f32, eot, expected, epsilon = 0.001),
            "Expected {expected}, actual {eot}"
        );

        let eot = compute_equation_of_time(6);
        let expected = -3.28165;
        assert!(
            approx_eq!(f32, eot, expected, epsilon = 0.001),
            "Expected {expected}, actual {eot}"
        );

        let eot = compute_equation_of_time(9);
        let expected = 10.84467;
        assert!(
            approx_eq!(f32, eot, expected, epsilon = 0.001),
            "Expected {expected}, actual {eot}"
        );
    }

    #[test]
    fn test_get_declination() {
        let declination = get_declination(0);
        let expected = -23.1;
        assert!(
            approx_eq!(f32, declination, expected, epsilon = 0.1),
            "Expected {expected}, actual {declination}"
        );

        let declination = get_declination(3);
        let expected = 3.62;
        assert!(
            approx_eq!(f32, declination, expected, epsilon = 0.1),
            "Expected {expected}, actual {declination}"
        );

        let declination = get_declination(6);
        let expected = 23.2;
        assert!(
            approx_eq!(f32, declination, expected, epsilon = 0.1),
            "Expected {expected}, actual {declination}"
        );
    }

    #[test]
    fn test_get_azimuth_and_elevation() {
        let (azimuth, elevation) = get_azimuth_and_elevation(0, 12.0);
        let azimuth = azimuth.to_degrees();
        let elevation = elevation.to_degrees();
        let expected = 183.1;
        assert!(
            approx_eq!(f32, azimuth, expected, epsilon = 1.0),
            "Expected {expected}, actual {azimuth}"
        );
        let expected = 25.8;
        assert!(
            approx_eq!(f32, elevation, expected, epsilon = 1.0),
            "Expected {expected}, actual {elevation}"
        );

        let (azimuth, elevation) = get_azimuth_and_elevation(0, 15.0);
        let azimuth = azimuth.to_degrees();
        let elevation = elevation.to_degrees();
        let expected = 224.4;
        assert!(
            approx_eq!(f32, azimuth, expected, epsilon = 1.0),
            "Expected {expected}, actual {azimuth}"
        );
        let expected = 11.98;
        assert!(
            approx_eq!(f32, elevation, expected, epsilon = 1.0),
            "Expected {expected}, actual {elevation}"
        );

        let (azimuth, elevation) = get_azimuth_and_elevation(6, 9.0);
        let azimuth = azimuth.to_degrees();
        let elevation = elevation.to_degrees();
        let expected = 104.06;
        assert!(
            approx_eq!(f32, azimuth, expected, epsilon = 1.0),
            "Expected {expected}, actual {azimuth}"
        );
        let expected = 50.54;
        assert!(
            approx_eq!(f32, elevation, expected, epsilon = 1.0),
            "Expected {expected}, actual {elevation}"
        );
    }

    #[test]
    fn test_convert_from_spherical_to_cartesian() {
        // on the horizon, exactly north
        let (azimuth, elevation) = (0.0, 0.0);
        let (x, y, z) = convert_from_spherical_to_cartesian(azimuth, elevation);

        let expected = 0.0;
        assert!(
            approx_eq!(f32, x, expected, epsilon = 0.01),
            "Expected {expected}, actual {x}"
        );

        let expected = 1.0;
        assert!(
            approx_eq!(f32, y, expected, epsilon = 0.01),
            "Expected {expected}, actual {y}"
        );

        let expected = 0.0;
        assert!(
            approx_eq!(f32, z, expected, epsilon = 0.01),
            "Expected {expected}, actual {z}"
        );

        // directly overhead
        let (azimuth, elevation) = (f32::to_radians(90.0), f32::to_radians(90.0));
        let (x, y, z) = convert_from_spherical_to_cartesian(azimuth, elevation);

        let expected = 0.0;
        assert!(
            approx_eq!(f32, x, expected, epsilon = 0.01),
            "Expected {expected}, actual {x}"
        );

        let expected = 0.0;
        assert!(
            approx_eq!(f32, y, expected, epsilon = 0.01),
            "Expected {expected}, actual {y}"
        );

        let expected = 1.0;
        assert!(
            approx_eq!(f32, z, expected, epsilon = 0.01),
            "Expected {expected}, actual {z}"
        );

        // 45° up, exactly NE
        let (azimuth, elevation) = (f32::to_radians(45.0), f32::to_radians(45.0));
        let (x, y, z) = convert_from_spherical_to_cartesian(azimuth, elevation);

        let expected = 0.5;
        assert!(
            approx_eq!(f32, x, expected, epsilon = 0.01),
            "Expected {expected}, actual {x}"
        );

        let expected = 0.5;
        assert!(
            approx_eq!(f32, y, expected, epsilon = 0.01),
            "Expected {expected}, actual {y}"
        );

        let expected = 0.707;
        assert!(
            approx_eq!(f32, z, expected, epsilon = 0.01),
            "Expected {expected}, actual {z}"
        );
    }

    #[test]
    fn test_cell_tetradehdron() {
        let ecosystem = Ecosystem::init();

        let tet = CellTetrahedron::new(CellIndex::new(0, 0), &ecosystem);
        assert_eq!(tet.top_left, CellIndex::new(0, 0));
        assert_eq!(tet.top_right, CellIndex::new(1, 0));
        assert_eq!(tet.bottom_left, CellIndex::new(0, 1));
        assert_eq!(tet.bottom_right, CellIndex::new(1, 1));
        assert_eq!(tet.normal_one.x, 0.0);
        assert_eq!(tet.normal_one.y, 0.0);
        assert_eq!(tet.normal_one.z, 1.0);

        // test intersections
        let pos = Vector3::new(0.0, 0.0, 0.0);
        let dir = Vector3::new(0.0, 0.0, 1.0);
        assert!(tet.has_intersection(pos, dir));

        let pos = Vector3::new(0.5, 0.5, 0.0);
        let dir = Vector3::new(0.0, 0.0, 1.0);
        assert!(tet.has_intersection(pos, dir));

        let pos = Vector3::new(1.5, 0.5, 0.0);
        let dir = Vector3::new(0.0, 0.0, 1.0);
        assert!(!tet.has_intersection(pos, dir));

        let pos = Vector3::new(0.5, 0.5, 99.0);
        let dir = Vector3::new(0.3, 0.3, 1.0).normalize();
        assert!(tet.has_intersection(pos, dir));

        let pos = Vector3::new(0.5, 0.5, 99.0);
        let dir = Vector3::new(0.3, 0.3, -1.0).normalize();
        assert!(tet.has_intersection(pos, dir));
    }
}

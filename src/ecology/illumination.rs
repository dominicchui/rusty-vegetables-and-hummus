use crate::constants;

use super::{CellIndex, Ecosystem};

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
    f32::asin(
        declination.sin() * latitude.sin()
            + declination.cos() * latitude.cos() * hra.cos(),
    )
}

fn get_azimuth(month: usize, local_time: f32) -> f32 {
    let elevation = get_elevation(month, local_time);
    let declination = get_declination(month).to_radians();
    let hra = get_hour_angle(month, local_time).to_radians();
    let latitude = constants::LATITUDE.to_radians();
    // angle between 0-π radians
    let angle = f32::acos(
        (declination.sin() * latitude.cos() - declination.cos() * latitude.sin() * hra.cos()) /
        elevation.cos()
    );
    // convert to full 2π radians
    if local_time >= 12.0 {
        (360.0 - angle.to_degrees()).to_radians()
    } else {
        angle
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;

    use crate::ecology::{
        illumination::{compute_equation_of_time, get_azimuth, get_declination, get_elevation, get_hour_angle},
        Ecosystem,
    };

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
    fn test_get_elevation() {
        let elevation = get_elevation(0, 12.0).to_degrees();
        let expected = 25.8;
        assert!(
            approx_eq!(f32, elevation, expected, epsilon = 1.0),
            "Expected {expected}, actual {elevation}"
        );

        let elevation = get_elevation(0, 15.0).to_degrees();
        let expected = 11.98;
        assert!(
            approx_eq!(f32, elevation, expected, epsilon = 1.0),
            "Expected {expected}, actual {elevation}"
        );

        let elevation = get_elevation(6, 9.0).to_degrees();
        let expected = 50.54;
        assert!(
            approx_eq!(f32, elevation, expected, epsilon = 1.0),
            "Expected {expected}, actual {elevation}"
        );
    }

    #[test]
    fn test_get_azimuth() {
        let azimuth = get_azimuth(0, 12.0).to_degrees();
        let expected = 183.1;
        assert!(
            approx_eq!(f32, azimuth, expected, epsilon = 1.0),
            "Expected {expected}, actual {azimuth}"
        );

        let azimuth = get_azimuth(0, 15.0).to_degrees();
        let expected = 224.4;
        assert!(
            approx_eq!(f32, azimuth, expected, epsilon = 1.0),
            "Expected {expected}, actual {azimuth}"
        );

        let azimuth = get_azimuth(6, 9.0).to_degrees();
        let expected = 104.06;
        assert!(
            approx_eq!(f32, azimuth, expected, epsilon = 1.0),
            "Expected {expected}, actual {azimuth}"
        );
    }
}

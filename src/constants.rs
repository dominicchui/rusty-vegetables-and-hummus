pub(crate) const SCREEN_WIDTH: usize = 1400;
pub(crate) const SCREEN_HEIGHT: usize = 1000;
pub(crate) const SPEED: f32 = 10.0;

// https://extension.psu.edu/calculating-the-green-weight-of-wood-species
pub(crate) const AREA_SIDE_LENGTH: usize = 100; // in cells
pub(crate) const CELL_SIDE_LENGTH: f32 = 10.0; // in meters
pub(crate) const DEFAULT_BEDROCK_HEIGHT: f32 = 100.0; // in meters

// https://en.wikipedia.org/wiki/Angle_of_repose#Of_various_materials
pub(crate) const CRITICAL_ANGLE_ROCK: f32 = 40.0;
pub(crate) const CRITICAL_ANGLE_SAND: f32 = 34.0;
pub(crate) const CRITICAL_ANGLE_HUMUS: f32 = 40.0;

// https://link.springer.com/referenceworkentry/10.1007/978-1-4020-3995-9_406
pub(crate) const HUMUS_DENSITY: f32 = 0.15;

// LIGHTNING
// based on ~10 lightning strikes per km per year
pub(crate) const MAX_LIGHTNING_PROBABILITY: f32 = 0.002;
// https://www.sciencedirect.com/science/article/pii/S0169555X13003929
const NUM_CELLS: usize = AREA_SIDE_LENGTH * AREA_SIDE_LENGTH;
const AREA_SIZE: f32 = (CELL_SIDE_LENGTH * CELL_SIDE_LENGTH) * NUM_CELLS as f32 / 1000000.0; // in km^3
pub(crate) const LIGHTNING_BEDROCK_DISPLACEMENT_VOLUME: f32 = 4.0; //4.0; // m^3

// https://www.sciencedirect.com/science/article/pii/S2351989421002973
// density of highland grasses
pub(crate) const GRASS_DENSITY: f32 = 1.0; // kg/m^3

pub(crate) const AVERAGE_MONTHLY_RAINFALL: [f32; 12] = [
    96.0, 81.0, 111.0, 99.0, 86.0, 91.0, 87.0, 103.0, 93.0, 106.0, 88.0, 110.0,
]; // in mm per month

pub(crate) const PER_CELL_RAINFALL: f32 = 1151.0;

//Sediment constants idk ask stupid Musgrave
pub(crate) const KC: f32 = 5.0;
pub(crate) const KD: f32 = 0.1;
pub(crate) const KS: f32 = 0.3;
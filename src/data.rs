use crate::*;
// TODO: Store using pixel clusters (fewer than 8bits per pixel)
const SINGLE_TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize; // in bytes

pub const TILE_EMPTY: [u8; SINGLE_TILE_LEN] = [0; SINGLE_TILE_LEN];

pub const TILE_CROSSHAIRS: [u8; SINGLE_TILE_LEN] = [
    1, 1, 0, 0, 0, 0, 0, 1, //
    1, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    1, 0, 0, 0, 0, 0, 0, 0, //
];

pub const TILE_SOLID: [u8; SINGLE_TILE_LEN] = [
    1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 1, 1, 1, 1, 1, //
];

pub const TILE_OUTLINE: [u8; SINGLE_TILE_LEN] = [
    1, 1, 1, 1, 1, 1, 1, 0, //
    1, 1, 1, 1, 1, 1, 1, 0, //
    1, 1, 1, 1, 1, 1, 1, 0, //
    1, 1, 1, 1, 1, 1, 1, 0, //
    1, 1, 1, 1, 1, 1, 1, 0, //
    1, 1, 1, 1, 1, 1, 1, 0, //
    1, 1, 1, 1, 1, 1, 1, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
];

pub const TILE_CHECKERS: [u8; SINGLE_TILE_LEN] = [
    0, 0, 0, 0, 1, 1, 1, 1, //
    0, 0, 0, 0, 1, 1, 1, 1, //
    0, 0, 0, 0, 1, 1, 1, 1, //
    0, 0, 0, 0, 1, 1, 1, 1, //
    2, 2, 2, 2, 3, 3, 3, 3, //
    2, 2, 2, 2, 3, 3, 3, 3, //
    2, 2, 2, 2, 3, 3, 3, 3, //
    2, 2, 2, 2, 3, 3, 3, 3, //
];

pub const TILE_CORNER: [u8; SINGLE_TILE_LEN] = [
    1, 1, 1, 1, 0, 0, 0, 0, //
    1, 1, 1, 1, 0, 0, 0, 0, //
    1, 1, 1, 1, 0, 0, 0, 0, //
    1, 1, 1, 1, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
];

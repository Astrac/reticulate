use reticulate::Coords3D;

use crate::utils;

pub fn single(_coords: Coords3D) -> u16 {
    1
}

pub fn terrain_single(_x: i32, _y: i32, _z: i32, _surface: i32) -> u16 {
    1
}

/// 1 grass, 2 dirt, 3 stone, 4 ore, 5 sand (low ground)
pub fn terrain_layered(x: i32, y: i32, z: i32, surface: i32) -> u16 {
    let depth = surface - y; // 1 = topmost voxel
    let top = if surface < 10 { 5 } else { 1 };
    // Jitter the dirt/stone boundary per column so strata aren't flat.
    let dirt_depth = 3 + (utils::hash3(x, 0, z, 7) % 3) as i32;
    if depth == 1 {
        top
    } else if depth <= dirt_depth {
        2
    } else if utils::hash3(x, y, z, 11).is_multiple_of(48) {
        4
    } else {
        3
    }
}

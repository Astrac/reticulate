use reticulate::{CompactOctree, Coords3D, DenseWalk, Fixed8Octree, Morton3D, Walk, utils::*};

pub fn test_scene() -> impl DenseWalk<()> {
    let mut octree: Fixed8Octree<()> = Fixed8Octree::new();

    // add_2x2_with_hole(&mut octree);
    // add_2x2(&mut octree);
    // add_8x8(&mut octree);
    // add_nested_cubes(&mut octree);
    add_tunnel(&mut octree);
    // add_3d_ortho_labyrint(&mut octree);
    // add_gyroid(&mut octree);
    // add_diagonal_helix(&mut octree);
    // add_fractal_staircase(&mut octree);

    let compressed = CompactOctree::from_fixed8(&octree);
    let mut print_walker = PrintWalker::default();
    compressed.walk(&mut print_walker);

    compressed
    // octree
}

pub fn add_8x8(octree: &mut Fixed8Octree<()>) {
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                octree.update(Coords3D { x: x, y: y, z: z }, Some(()));
            }
        }
    }
}

pub fn add_tunnel(octree: &mut Fixed8Octree<()>) {
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                if (y == 3 || y == 4) && (x == 3 || x == 4) {
                    continue;
                }

                octree.update(Coords3D { x: x, y: y+1, z: z }, Some(()));
            }
        }
    }
}

pub fn add_nested_cubes(octree: &mut Fixed8Octree<()>) {
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                if (y == 3 || y == 4) && (x == 3 || x == 4) && (z == 3 || z == 4) {
                    continue;
                }

                octree.update(Coords3D { x: x, y: y, z: z }, Some(()));
            }
        }
    }
}

pub fn add_2x2_with_hole(octree: &mut Fixed8Octree<()>) {
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }

                octree.update(Coords3D { x: x, y: y, z: z }, Some(()));
            }
        }
    }
}

pub fn add_2x2(octree: &mut Fixed8Octree<()>) {
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                octree.update(Coords3D { x: x, y: y, z: z }, Some(()));
            }
        }
    }
}

pub fn add_3d_ortho_labyrint(octree: &mut Fixed8Octree<()>) {
    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                let mut is_solid = false;

                if y % 4 == 0 && z % 4 == 0 {
                    is_solid = true;
                }

                if x % 4 == 2 && z % 4 == 2 {
                    is_solid = true;
                }

                if x % 4 == 0 && y % 4 == 2 {
                    is_solid = true;
                }

                if is_solid {
                    let coords = Coords3D { x, y, z };
                    octree.update(coords, Some(()));
                }
            }
        }
    }
}

fn add_diagonal_helix(octree: &mut Fixed8Octree<()>) {
    let steps = 5000;
    let radius = 6.0;
    let revolutions = 8.0;

    for i in 0..steps {
        let t = i as f64 / steps as f64;

        let cx = t * 63.0;
        let cy = t * 63.0;
        let cz = t * 63.0;

        let angle = t * revolutions * 2.0 * std::f64::consts::PI;

        let offset_x = radius * angle.cos();
        let offset_y = radius * angle.sin();
        let offset_z = radius * (-angle.cos() - angle.sin()) * 0.5;

        let rx = (cx + offset_x).round() as i32;
        let ry = (cy + offset_y).round() as i32;
        let rz = (cz + offset_z).round() as i32;

        if rx >= 0 && rx < 32 && ry >= 0 && ry < 32 && rz >= 0 && rz < 32 {
            let x = rx as u8;
            let y = ry as u8;
            let z = rz as u8;

            octree.update(Coords3D { x, y, z }, Some(()));
        }
    }
}

fn add_gyroid(octree: &mut Fixed8Octree<()>) {
    let period_wavelength = 16.0;
    let k = (2.0 * std::f64::consts::PI) / period_wavelength;

    for z in 0..32 {
        let z_f = z as f64 * k;
        let sin_z = z_f.sin();
        let cos_z = z_f.cos();

        for y in 0..32 {
            let y_f = y as f64 * k;
            let sin_y = y_f.sin();
            let cos_y = y_f.cos();

            for x in 0..32 {
                let x_f = x as f64 * k;
                let sin_x = x_f.sin();
                let cos_x = x_f.cos();

                let value = sin_x * cos_y + sin_y * cos_z + sin_z * cos_x;

                if value > 0.0 {
                    octree.update(Coords3D { x, y, z }, Some(()));
                }
            }
        }
    }
}

fn add_fractal_staircase(octree: &mut Fixed8Octree<()>) {
    let max_stress_code = 0o331332;
    let total_voxels_to_populate = max_stress_code + 1;

    for code in 0..total_voxels_to_populate {
        let coords = Coords3D::from_morton_unscaled(Morton3D::from(code));
        octree.update(coords, Some(()));
    }
}

use crate::{
    ChunkCoord, Coords2D,
    axis::Axis,
    meshing::{
        bitfields::{Claims, ClaimsMask, PARITY_WORDS, PLANE_SIDE, Parity, ParityMask},
        surface_layer::SurfaceLayer,
        triangulation::{filler::Filler, edges::Edges},
    },
};

struct Fixture {
    parity: [Vec<u64>; 3],
    claims: [Vec<u64>; 3],
}

fn storage() -> [Vec<u64>; 3] {
    core::array::from_fn(|_| vec![0u64; PARITY_WORDS])
}

fn parity_of(s: &mut [Vec<u64>; 3]) -> Parity<'_> {
    let [x, y, z] = s;
    Parity {
        masks: [
            ParityMask::from_u64_slice(x),
            ParityMask::from_u64_slice(y),
            ParityMask::from_u64_slice(z),
        ],
    }
}

fn claims_of(s: &mut [Vec<u64>; 3]) -> Claims<'_> {
    let [x, y, z] = s;
    Claims {
        masks: [
            ClaimsMask::from_u64_slice(x),
            ClaimsMask::from_u64_slice(y),
            ClaimsMask::from_u64_slice(z),
        ],
    }
}

/// Builds parity from a set of solid voxels, with the same rules as
/// pass 1: a transition along axis A at coordinate c wherever the voxel
/// at c differs from the one at c-1. Outside the chunk counts as empty.
/// (u, v) follow projected_axis_uv: X -> (y, z), Y -> (z, x), Z -> (x, y).
fn fixture(solid: &[(usize, usize, usize)]) -> Fixture {
    let mut grid = vec![false; PLANE_SIDE * PLANE_SIDE * PLANE_SIDE];
    let idx = |x: usize, y: usize, z: usize| (x * PLANE_SIDE + y) * PLANE_SIDE + z;
    for &(x, y, z) in solid {
        grid[idx(x, y, z)] = true;
    }
    let at = |x: isize, y: isize, z: isize| {
        x >= 0 && y >= 0 && z >= 0 && grid[idx(x as usize, y as usize, z as usize)]
    };

    let mut f = Fixture {
        parity: storage(),
        claims: storage(),
    };
    let mut parity = parity_of(&mut f.parity);
    for x in 0..PLANE_SIDE {
        for y in 0..PLANE_SIDE {
            for z in 0..PLANE_SIDE {
                let (xi, yi, zi) = (x as isize, y as isize, z as isize);
                let here = at(xi, yi, zi);
                let uv = |u: usize, v: usize| Coords2D {
                    u: u as u8,
                    v: v as u8,
                };
                if here != at(xi - 1, yi, zi) {
                    parity.set_transition(SurfaceLayer::new(Axis::X, x as ChunkCoord), uv(y, z), 1);
                }
                if here != at(xi, yi - 1, zi) {
                    parity.set_transition(SurfaceLayer::new(Axis::Y, y as ChunkCoord), uv(z, x), 1);
                }
                if here != at(xi, yi, zi - 1) {
                    parity.set_transition(SurfaceLayer::new(Axis::Z, z as ChunkCoord), uv(x, y), 1);
                }
            }
        }
    }
    drop(parity);
    f
}

fn cells(list: &[(usize, usize)]) -> [u32; PLANE_SIDE] {
    let mut m = [0u32; PLANE_SIDE];
    for &(row, col) in list {
        m[row] |= 1 << col;
    }
    m
}

fn plane_fixture(cells_rc: &[(usize, usize)]) -> Fixture {
    let voxels: Vec<_> = cells_rc.iter().map(|&(r, c)| (c, 0, r)).collect();
    fixture(&voxels)
}

fn discover(f: &mut Fixture, axis: Axis, axis_coord: u8, seed: (u32, u32)) -> [u32; PLANE_SIDE] {
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);
    let layer = SurfaceLayer::new(axis, axis_coord);
    let edges = Edges::from_parity(&parity, layer);
    Filler::new(&claims, &parity).fill_region(
        layer,
        &edges,
        Coords2D {
            u: seed.0 as u8,
            v: seed.1 as u8,
        },
    )
}

/// Draws the rows that have any bit set, top row first, for readable failures.
fn render(mask: &[u32; PLANE_SIDE]) -> String {
    (0..PLANE_SIDE)
        .rev()
        .filter(|&r| mask[r] != 0)
        .map(|r| {
            let line: String = (0..PLANE_SIDE)
                .map(|c| if (mask[r] >> c) & 1 == 1 { '#' } else { '.' })
                .collect();
            format!("{r:>2} {line}\n")
        })
        .collect()
}

fn assert_region(region: &[u32; PLANE_SIDE], expected: &[(usize, usize)]) {
    let expected = cells(expected);
    assert_eq!(
        region,
        &expected,
        "\ngot:\n{}expected:\n{}",
        render(region),
        render(&expected)
    );
}

#[test]
fn rectangle_from_an_interior_seed() {
    let shape: Vec<_> = (2..5).flat_map(|r| (1..4).map(move |c| (r, c))).collect();
    let mut f = plane_fixture(&shape);
    assert_region(&discover(&mut f, Axis::Y, 0, (3, 2)), &shape);
}

/// Rows below the seed are found by the downward pass.
#[test]
fn l_shape_seeded_mid_face() {
    let mut shape = Vec::new();
    for r in 0..3 {
        for c in 1..=3 {
            if (r, c) != (0, 1) {
                shape.push((r, c));
            }
        }
    }
    let mut f = plane_fixture(&shape);
    assert_region(&discover(&mut f, Axis::Y, 0, (1, 1)), &shape);
}

/// The right leg hangs below the bar: a valley under the upward pass.
#[test]
fn inverted_u_from_left_leg() {
    let mut shape: Vec<_> = (1..=5).map(|c| (3, c)).collect();
    for r in 0..3 {
        shape.extend([(r, 1), (r, 5)]);
    }
    let mut f = plane_fixture(&shape);
    assert_region(&discover(&mut f, Axis::Y, 0, (0, 1)), &shape);
}

/// Column 4 rises above row 1, outside the seed's run: a hill over the
/// downward pass.
#[test]
fn hill_beside_downward_extension() {
    let mut shape = vec![(2, 1), (2, 2), (2, 4), (3, 4)];
    shape.extend((1..=4).map(|c| (1, c)));
    let mut f = plane_fixture(&shape);
    assert_region(&discover(&mut f, Axis::Y, 0, (2, 1)), &shape);
}

#[test]
fn table_seeded_on_its_top() {
    let mut shape: Vec<_> = (1..=6).map(|c| (3, c)).collect();
    for r in 0..3 {
        shape.extend([(r, 1), (r, 2), (r, 5), (r, 6)]);
    }
    let mut f = plane_fixture(&shape);
    assert_region(&discover(&mut f, Axis::Y, 0, (3, 3)), &shape);
}

/// A snake that changes vertical direction twice: up leg A, across the
/// top bar, down leg B, across the bottom bar, up leg C. The loop needs
/// more than one round to reach C.
#[test]
fn snake_needs_several_rounds() {
    let mut shape: Vec<_> = (0..=3).map(|r| (r, 1)).collect(); // leg A
    shape.extend((1..=4).map(|c| (4, c))); // top bar
    shape.extend((1..=3).map(|r| (r, 4))); // leg B
    shape.extend((4..=7).map(|c| (0, c))); // bottom bar
    shape.extend((1..=4).map(|r| (r, 7))); // leg C
    let mut f = plane_fixture(&shape);
    assert_region(&discover(&mut f, Axis::Y, 0, (0, 1)), &shape);
}

/// On plane y = 1, row 3 is a P face and row 2 directly below it is an N
/// face: they touch but aren't connected. The column at x = 5 is a
/// separate shape. Only the seed's row belongs to the region.
#[test]
fn unconnected_shapes_are_excluded() {
    let mut voxels = Vec::new();
    for x in 1..=3 {
        voxels.push((x, 0, 3)); // P faces on plane y = 1, row 3
        voxels.push((x, 1, 2)); // N faces on plane y = 1, row 2
    }
    for z in 0..=2 {
        voxels.push((5, 0, z)); // separate P-face column
    }
    let mut f = fixture(&voxels);
    assert_region(
        &discover(&mut f, Axis::Y, 1, (3, 1)),
        &[(3, 1), (3, 2), (3, 3)],
    );
}

/// A claimed cell is excluded, and so is the leg that could only be
/// reached through it.
#[test]
fn claims_cut_off_what_lies_beyond_them() {
    let mut shape: Vec<_> = (1..=5).map(|c| (2, c)).collect();
    shape.extend([(0, 5), (1, 5)]);
    let mut f = plane_fixture(&shape);
    claims_of(&mut f.claims).claim(SurfaceLayer::new(Axis::Y, 0), &cells(&[(2, 5)]));
    assert_region(
        &discover(&mut f, Axis::Y, 0, (2, 1)),
        &[(2, 1), (2, 2), (2, 3), (2, 4)],
    );
}

/// A column spanning every row, seeded in the middle, so both passes
/// have to reach the chunk's first and last rows.
#[test]
fn full_height_column_from_the_middle() {
    let shape: Vec<_> = (0..PLANE_SIDE).map(|r| (r, 3)).collect();
    let mut f = plane_fixture(&shape);
    assert_region(&discover(&mut f, Axis::Y, 0, (16, 3)), &shape);
}

/// A run reaching the last column. Its right end would be a wall at
/// column 32, which a 32-bit row can't hold.
#[test]
#[ignore = "Disabled: touching chunk boundary not yet implemented"]
fn run_reaching_last_column() {
    let shape = [(3, 29), (3, 30), (3, 31)];
    let mut f = plane_fixture(&shape);
    assert_region(&discover(&mut f, Axis::Y, 0, (3, 29)), &shape);
}

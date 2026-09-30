use bitvec::prelude::*;

use crate::{
    Coords2D, Depth, Morton2D, Morton3D,
    axis::*,
    meshing::bitfields::{MATERIAL_BITS, Material, PLANE_AREA},
    octree::TREE_DEPTH,
};

fn depth_area(d: usize) -> usize {
    PLANE_AREA >> (d << 1)
}

fn coords(cell: usize) -> Coords2D {
    Morton2D::from(cell).as_coords()
}

fn is_power_of_four(n: usize) -> bool {
    n.is_power_of_two() && n.trailing_zeros() % 2 == 0
}

struct Planes {
    bits: [BitArr!(for PLANE_AREA * MATERIAL_BITS, in u64, Lsb0); 3],
}

impl Planes {
    fn new() -> Self {
        Self {
            bits: core::array::from_fn(|_| BitArray::ZERO),
        }
    }

    fn material(&mut self) -> Material<'_> {
        let [x, y, z] = &mut self.bits;
        Material {
            cells: [
                x.as_mut_bitslice(),
                y.as_mut_bitslice(),
                z.as_mut_bitslice(),
            ],
        }
    }

    fn plane(&self, axis: Axis) -> &BitSlice<u64, Lsb0> {
        self.bits[axis.as_usize()].as_bitslice()
    }

    fn plane_mut(&mut self, axis: Axis) -> &mut BitSlice<u64, Lsb0> {
        self.bits[axis.as_usize()].as_mut_bitslice()
    }

    fn with_background(background: u16) -> Self {
        let mut p = Self::new();
        for axis in AXES {
            write_run(p.plane_mut(axis), 0, PLANE_AREA, background);
        }
        p
    }
}

fn read_cell(plane: &BitSlice<u64, Lsb0>, cell: usize) -> u16 {
    let bit = cell * MATERIAL_BITS;
    plane[bit..bit + MATERIAL_BITS].load_le::<u16>()
}

fn write_run(plane: &mut BitSlice<u64, Lsb0>, start: usize, len: usize, material: u16) {
    let bit_start = start * MATERIAL_BITS;
    let bit_len = len * MATERIAL_BITS;
    for bit in (bit_start..bit_start + bit_len).step_by(MATERIAL_BITS) {
        plane[bit..bit + MATERIAL_BITS].store_le::<u16>(material);
    }
}

fn is_uniform(plane: &BitSlice<u64, Lsb0>, start: usize, len: usize) -> bool {
    let first = read_cell(plane, start);
    (start..start + len).all(|c| read_cell(plane, c) == first)
}

type Run = (u16, Coords2D, usize);

fn reference_runs(plane: &BitSlice<u64, Lsb0>, start: usize, len: usize) -> Vec<Run> {
    let end = start + len;

    let mut sizes = Vec::new();
    let mut s = 1;
    while s <= len {
        sizes.push(s);
        s *= 4;
    }
    sizes.reverse();

    let mut runs = Vec::new();
    let mut offset = start;
    while offset < end {
        let size = *sizes
            .iter()
            .find(|&&s| offset % s == 0 && offset + s <= end && is_uniform(plane, offset, s))
            .expect("a single cell is always uniform");
        runs.push((read_cell(plane, offset), coords(offset), size));
        offset += size;
    }
    runs
}

fn collect_runs(planes: &mut Planes, axis: Axis, offset: usize, d: usize) -> Vec<Run> {
    let mut runs = Vec::new();
    planes.material().iterate_materials(
        axis,
        Morton2D::from(offset),
        Depth(d as u8),
        |m, c, len| runs.push((m, c, len)),
    );
    runs
}

#[test]
fn uniform_plane_is_a_single_run() {
    for axis in AXES {
        let mut p = Planes::new();
        write_run(p.plane_mut(axis), 0, PLANE_AREA, 5);

        assert_eq!(
            collect_runs(&mut p, axis, 0, 0),
            vec![(5, coords(0), PLANE_AREA)]
        );
    }
}

#[test]
fn every_node_of_a_uniform_plane_is_one_run_limited_to_that_node() {
    let mut p = Planes::new();
    write_run(p.plane_mut(Axis::Z), 0, PLANE_AREA, 7);

    for d in 0..=TREE_DEPTH {
        let size = depth_area(d);
        for offset in (0..PLANE_AREA).step_by(size) {
            assert_eq!(
                collect_runs(&mut p, Axis::Z, offset, d),
                vec![(7, coords(offset), size)],
                "depth {d}, offset {offset}"
            );
        }
    }
}

#[test]
fn each_quadrant_reports_its_own_material_and_start_coords() {
    let quad = depth_area(1);
    let mut p = Planes::new();
    for q in 0..4 {
        write_run(p.plane_mut(Axis::X), q * quad, quad, q as u16);
    }

    for q in 0..4 {
        let offset = q * quad;
        assert_eq!(
            collect_runs(&mut p, Axis::X, offset, 1),
            vec![(q as u16, coords(offset), quad)],
            "quadrant {q}"
        );
    }
}

#[test]
fn axis_selects_which_plane_is_read() {
    let mut p = Planes::new();
    write_run(p.plane_mut(Axis::X), 0, PLANE_AREA, 1);
    write_run(p.plane_mut(Axis::Y), 0, PLANE_AREA, 2);
    write_run(p.plane_mut(Axis::Z), 0, PLANE_AREA, 3);

    for (axis, material) in [(Axis::X, 1), (Axis::Y, 2), (Axis::Z, 3)] {
        assert_eq!(
            collect_runs(&mut p, axis, 0, 0),
            vec![(material, coords(0), PLANE_AREA)]
        );
    }
}

fn check_single_odd_cell(odd_offset: usize) {
    let mut p = Planes::new();
    write_run(p.plane_mut(Axis::Y), 0, PLANE_AREA, 1);
    write_run(p.plane_mut(Axis::Y), odd_offset, 1, 2);

    let runs = collect_runs(&mut p, Axis::Y, 0, 0);

    assert_eq!(runs, reference_runs(p.plane(Axis::Y), 0, PLANE_AREA));

    let odd: Vec<_> = runs.iter().filter(|r| r.0 == 2).collect();
    assert_eq!(odd.len(), 1);
    assert_eq!(*odd[0], (2, coords(odd_offset), 1));

    assert_eq!(runs.len(), 1 + 3 * TREE_DEPTH);
    assert!(runs.iter().all(|r| is_power_of_four(r.2)), "{runs:?}");
    assert_eq!(runs.iter().map(|r| r.2).sum::<usize>(), PLANE_AREA);
}

#[test]
fn odd_cell_at_start_splits_area_into_quadtree_nodes() {
    check_single_odd_cell(0);
}

#[test]
fn odd_cell_at_end_splits_area_into_quadtree_nodes() {
    check_single_odd_cell(PLANE_AREA - 1);
}

#[test]
fn odd_cell_in_the_middle_splits_area_into_quadtree_nodes() {
    check_single_odd_cell(PLANE_AREA / 2 + 1);
}

#[test]
fn two_uniform_cells_are_not_merged_into_a_non_quadtree_run() {
    let mut p = Planes::new();
    write_run(p.plane_mut(Axis::X), 0, PLANE_AREA, 1);
    write_run(p.plane_mut(Axis::X), 2, 2, 2);

    let runs = collect_runs(&mut p, Axis::X, 0, 0);
    assert!(runs.iter().all(|r| is_power_of_four(r.2)), "{runs:?}");
    assert_eq!(
        &runs[..4],
        &[
            (1, coords(0), 1),
            (1, coords(1), 1),
            (2, coords(2), 1),
            (2, coords(3), 1),
        ]
    );
}

#[test]
fn checkerboard_of_cells_yields_only_leaves_in_morton_order() {
    let mut p = Planes::new();
    for i in 0..PLANE_AREA {
        write_run(p.plane_mut(Axis::Z), i, 1, (i % 2) as u16 + 1);
    }

    let runs = collect_runs(&mut p, Axis::Z, 0, 0);

    assert_eq!(runs.len(), PLANE_AREA);
    assert!(runs.iter().all(|r| r.2 == 1));
    assert_eq!(runs, reference_runs(p.plane(Axis::Z), 0, PLANE_AREA));
}

#[test]
fn smallest_node_is_a_single_run() {
    let mut p = Planes::new();
    write_run(p.plane_mut(Axis::X), 0, PLANE_AREA, 5);
    let offset = 5;
    write_run(p.plane_mut(Axis::X), offset, 1, 6);

    assert_eq!(
        collect_runs(&mut p, Axis::X, offset, TREE_DEPTH),
        vec![(6, coords(offset), 1)]
    );
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % bound
    }
}

#[test]
fn random_planes_match_the_reference_decomposition() {
    let mut rng = Lcg(0xC0FFEE);

    for case in 0..64 {
        let axis = AXES[rng.next(3)];
        let mut p = Planes::new();
        write_run(p.plane_mut(axis), 0, PLANE_AREA, 1);

        // Scatter uniform nodes of random size/material to build nested structure.
        for _ in 0..12 {
            let d = rng.next(TREE_DEPTH + 1);
            let size = depth_area(d);
            let offset = rng.next(PLANE_AREA / size) * size;
            write_run(p.plane_mut(axis), offset, size, 1 + rng.next(3) as u16);
        }

        let before = p.bits.clone();

        let d = rng.next(TREE_DEPTH + 1);
        let size = depth_area(d);
        let offset = rng.next(PLANE_AREA / size) * size;
        let runs = collect_runs(&mut p, axis, offset, d);

        assert_eq!(
            runs,
            reference_runs(p.plane(axis), offset, size),
            "case {case}: depth {d}, offset {offset}"
        );
        assert_eq!(runs.iter().map(|r| r.2).sum::<usize>(), size, "case {case}");
        assert!(runs.iter().all(|r| is_power_of_four(r.2)), "case {case}");
        assert!(
            p.bits == before,
            "case {case}: iterate_materials must not modify planes"
        );
    }
}

const BACKGROUND: u16 = 1;

fn assert_only_area_changed(p: &Planes, axis: Axis, start: usize, len: usize, material: u16) {
    for a in AXES {
        for cell in 0..PLANE_AREA {
            let expected =
                if a.as_usize() == axis.as_usize() && (start..start + len).contains(&cell) {
                    material
                } else {
                    BACKGROUND
                };
            assert_eq!(
                read_cell(p.plane(a), cell),
                expected,
                "plane {}, cell {cell} (filled {start}..{})",
                a.as_usize(),
                start + len
            );
        }
    }
}

#[test]
fn fill_at_origin_writes_exactly_the_node_at_every_depth() {
    for axis in AXES {
        for d in 0..=TREE_DEPTH {
            let mut p = Planes::with_background(BACKGROUND);
            p.material()
                .fill(axis, Morton3D::from_xyz(0, 0, 0), Depth(d as u8), 2);
            assert_only_area_changed(&p, axis, 0, depth_area(d), 2);
        }
    }
}

#[test]
fn fill_at_offset_node_writes_exactly_the_node_at_every_depth() {
    for axis in AXES {
        for d in 1..=TREE_DEPTH {
            let m = Morton3D::from_xyz(1, 1, 1);
            let size = depth_area(d);
            let (morton_2d, _) = m.projected_axis_morton(axis, Depth(d as u8));
            let start = morton_2d.as_usize();
            assert_eq!(
                start,
                3 * size,
                "test assumption about node (1,1)'s position"
            );

            let mut p = Planes::with_background(BACKGROUND);
            p.material().fill(axis, m, Depth(d as u8), 3);

            assert_only_area_changed(&p, axis, start, size, 3);
        }
    }
}

#[test]
fn fill_handles_extreme_material_values() {
    for material in [0, 1, u16::MAX, u16::MAX - 1, u16::MAX / 2] {
        let mut p = Planes::with_background(BACKGROUND);
        p.material()
            .fill(Axis::Y, Morton3D::from_xyz(0, 0, 0), Depth(1), material);
        assert_only_area_changed(&p, Axis::Y, 0, depth_area(1), material);
    }
}

#[test]
fn fill_overwrites_previous_content_of_the_area() {
    let mut p = Planes::new();
    let axis = Axis::X;
    p.material()
        .fill(axis, Morton3D::from_xyz(0, 0, 0), Depth(1), 4);
    p.material()
        .fill(axis, Morton3D::from_xyz(0, 0, 0), Depth(2), 5);

    let inner = depth_area(2);
    let outer = depth_area(1);
    for cell in 0..outer {
        let expected = if cell < inner { 5 } else { 4 };
        assert_eq!(read_cell(p.plane(axis), cell), expected, "cell {cell}");
    }
    assert_eq!(
        read_cell(p.plane(axis), outer),
        0,
        "outside stays untouched"
    );
}

#[test]
fn fill_then_iterate_round_trips() {
    for axis in AXES {
        for d in 0..=TREE_DEPTH {
            let mut p = Planes::new();
            let m = Morton3D::from_xyz(0, 0, 0);
            p.material().fill(axis, m, Depth(d as u8), 6);

            let offset = m.drop_axis(axis).as_usize();
            assert_eq!(
                collect_runs(&mut p, axis, offset, d),
                vec![(6, coords(offset), depth_area(d))],
                "depth {d}"
            );
        }
    }
}

#[test]
fn fill_z_axis_bug_fix() {
    let mut p = Planes::new();

    let m0 = Morton3D::from(0);
    p.material().fill(Axis::Z, m0, Depth(4), 1);

    let m1 = Morton3D::from(1);
    p.material().fill(Axis::Z, m1, Depth(4), 0);

    let size = depth_area(4);
    assert_eq!(
        collect_runs(&mut p, Axis::Z, 0, 4),
        vec![(1, coords(0), size)],
    );
}

#[test]
fn filling_a_child_makes_the_parent_decompose() {
    let mut p = Planes::new();
    p.material()
        .fill(Axis::Z, Morton3D::from_xyz(0, 0, 0), Depth(0), 1);
    p.material()
        .fill(Axis::Z, Morton3D::from_xyz(0, 0, 0), Depth(2), 2);

    let runs = collect_runs(&mut p, Axis::Z, 0, 0);
    assert_eq!(runs, reference_runs(p.plane(Axis::Z), 0, PLANE_AREA));
    assert_eq!(runs[0], (2, coords(0), depth_area(2)));
    assert_eq!(runs.iter().map(|r| r.2).sum::<usize>(), PLANE_AREA);
}

#[test]
fn clear_material_planes_zeroes_all_three_planes() {
    let mut p = Planes::with_background(BACKGROUND);
    p.material().clear();
    assert!(p.bits.iter().all(|b| b.not_any()));
}

#[test]
fn clear_material_planes_on_empty_planes_is_a_no_op() {
    let mut p = Planes::new();
    p.material().clear();
    assert!(p.bits.iter().all(|b| b.not_any()));
}

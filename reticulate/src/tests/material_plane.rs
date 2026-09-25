use bitvec::prelude::*;

use crate::{
    Coords2D, Depth, Morton2D, Morton3D,
    axis::*,
    meshing::bitfields::{MATERIAL_BITS, Material, MaterialId, PLANE_AREA, PLANE_SIDE},
    octree::TREE_DEPTH,
    utils::print_materials,
};

const LEAF: usize = size_of::<MaterialId>();

fn depth_area(d: usize) -> usize {
    PLANE_AREA >> (d << 1)
}

struct Planes {
    bits: [BitArr!(for PLANE_AREA * MATERIAL_BITS, in u64, Lsb0); 3],
}

impl Planes {
    fn new() -> Self {
        Self {
            bits: Default::default(),
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

    fn with_background(background: MaterialId) -> Self {
        let mut p = Self::new();
        for axis in AXES {
            write_run(p.plane_mut(axis), 0, PLANE_AREA, background);
        }
        p
    }
}

fn read_leaf(plane: &BitSlice<u64, Lsb0>, offset: usize) -> MaterialId {
    let offset = offset * MATERIAL_BITS;
    plane[offset..offset + MATERIAL_BITS].load_le::<MaterialId>()
}

fn write_run(plane: &mut BitSlice<u64, Lsb0>, start: usize, len: usize, material: u16) {
    let len = len * MATERIAL_BITS;
    let start = start * MATERIAL_BITS;
    let step = MATERIAL_BITS;
    for leaf in (start..start + len).step_by(step) {
        plane[leaf..leaf + step].store_le::<u16>(material);
    }
}

fn is_uniform(plane: &BitSlice<u64, Lsb0>, start: usize, len: usize) -> bool {
    let start = start * MATERIAL_BITS;
    let len = len * MATERIAL_BITS;
    let first = read_leaf(plane, start);
    (start..start + len)
        .step_by(LEAF)
        .all(|o| read_leaf(plane, o) == first)
}

type Run = (MaterialId, Coords2D, usize);

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
            .expect("a single leaf is always uniform");

        runs.push((
            read_leaf(plane, offset),
            Morton2D::from(offset).as_coords(),
            size,
        ));
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

fn coords(offset: usize) -> Coords2D {
    Morton2D::from(offset).as_coords()
}

fn is_power_of_four(n: usize) -> bool {
    n.is_power_of_two() && n.trailing_zeros() % 2 == 0
}

#[test]
fn uniform_plane_is_a_single_run() {
    for axis in AXES {
        let mut p = Planes::new();
        write_run(p.plane_mut(axis), 0, PLANE_AREA, 0x1234);

        assert_eq!(
            collect_runs(&mut p, axis, 0, 0),
            vec![(0x1234, coords(0), PLANE_AREA)]
        );
    }
}

#[test]
fn every_node_of_a_uniform_plane_is_one_run_limited_to_that_node() {
    let mut p = Planes::new();
    write_run(p.plane_mut(AXIS_Z), 0, PLANE_AREA, 7);

    for d in 0..=TREE_DEPTH {
        let size = depth_area(d);
        for offset in (0..PLANE_AREA).step_by(size) {
            assert_eq!(
                collect_runs(&mut p, AXIS_Z, offset, d),
                vec![(7, coords(offset), size)],
                "depth {d}, offset {offset}"
            );
        }
    }
}

#[test]
fn each_quadrant_reports_its_own_material_and_start_coords() {
    // Neighbouring quadrants differ, so a run must stop at the node border,
    // and the reported coords must be those of the node's first cell.
    let quad = depth_area(1);
    let mut p = Planes::new();
    for q in 0..4 {
        write_run(p.plane_mut(AXIS_X), q * quad, quad, 100 + q as u16);
    }

    for q in 0..4 {
        let offset = q * quad;
        assert_eq!(
            collect_runs(&mut p, AXIS_X, offset, 1),
            vec![(100 + q as u16, coords(offset), quad)],
            "quadrant {q}"
        );
    }
}

#[test]
fn axis_selects_which_plane_is_read() {
    let mut p = Planes::new();
    write_run(p.plane_mut(AXIS_X), 0, PLANE_AREA, 1);
    write_run(p.plane_mut(AXIS_Y), 0, PLANE_AREA, 2);
    write_run(p.plane_mut(AXIS_Z), 0, PLANE_AREA, 3);

    for (axis, material) in [(AXIS_X, 1), (AXIS_Y, 2), (AXIS_Z, 3)] {
        assert_eq!(
            collect_runs(&mut p, axis, 0, 0),
            vec![(material, coords(0), depth_area(0))]
        );
    }
}

fn check_single_odd_leaf(odd_offset: usize) {
    let mut p = Planes::new();
    write_run(p.plane_mut(AXIS_Y), 0, PLANE_AREA, 1);
    write_run(p.plane_mut(AXIS_Y), odd_offset, LEAF, 2);

    let runs = collect_runs(&mut p, AXIS_Y, 0, 0);

    assert_eq!(runs, reference_runs(p.plane(AXIS_Y), 0, PLANE_AREA));

    // The odd leaf is reported exactly once, at its own coordinates...
    let odd: Vec<_> = runs.iter().filter(|r| r.0 == 2).collect();
    assert_eq!(odd.len(), 1);
    assert_eq!(*odd[0], (2, coords(odd_offset), LEAF));

    // ...and the rest is the root->leaf path: 3 sibling nodes per level.
    assert_eq!(runs.len(), 1 + 3 * TREE_DEPTH);
    assert!(runs.iter().all(|r| is_power_of_four(r.2)), "{runs:?}");
    assert_eq!(runs.iter().map(|r| r.2).sum::<usize>(), PLANE_AREA);
}

#[test]
fn odd_leaf_at_start_splits_area_into_quadtree_nodes() {
    check_single_odd_leaf(0);
}

#[test]
fn odd_leaf_at_end_splits_area_into_quadtree_nodes() {
    check_single_odd_leaf(PLANE_AREA - LEAF);
}

#[test]
fn odd_leaf_in_the_middle_splits_area_into_quadtree_nodes() {
    check_single_odd_leaf(PLANE_AREA / 2 + LEAF);
}

#[test]
fn two_uniform_leaves_are_not_merged_into_a_non_quadtree_run() {
    // [A A B B] + rest A: A A is uniform but is not a quadtree node, so the
    // decomposition has to report them as two separate leaves.
    let mut p = Planes::new();
    write_run(p.plane_mut(AXIS_X), 0, PLANE_AREA, 1);
    write_run(p.plane_mut(AXIS_X), 2 * LEAF, 2 * LEAF, 2);

    let runs = collect_runs(&mut p, AXIS_X, 0, 0);
    assert!(runs.iter().all(|r| is_power_of_four(r.2)), "{runs:?}");
    assert_eq!(
        runs[..4].to_vec(),
        vec![
            (1, coords(0), LEAF),
            (1, coords(LEAF), LEAF),
            (2, coords(2 * LEAF), LEAF),
            (2, coords(3 * LEAF), LEAF),
        ]
    );
}

#[test]
fn checkerboard_of_leaves_yields_only_leaves_in_morton_order() {
    let mut p = Planes::new();
    let leaves = PLANE_AREA / LEAF;
    for i in 0..leaves {
        write_run(p.plane_mut(AXIS_Z), i * LEAF, LEAF, (i % 2) as u16 + 1);
    }

    let runs = collect_runs(&mut p, AXIS_Z, 0, 0);

    assert_eq!(runs.len(), leaves);
    assert!(runs.iter().all(|r| r.2 == LEAF));
    assert_eq!(runs, reference_runs(p.plane(AXIS_Z), 0, PLANE_AREA));
}

#[test]
fn smallest_node_is_a_single_run() {
    let mut p = Planes::new();
    write_run(p.plane_mut(AXIS_X), 0, PLANE_AREA, 5);
    write_run(p.plane_mut(AXIS_X), 5, 1, 6);

    assert_eq!(
        collect_runs(&mut p, AXIS_X, 5, TREE_DEPTH),
        vec![(6, coords(5), 1)]
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

const BACKGROUND: u16 = 0xA5C3;

fn assert_only_area_changed(p: &Planes, axis: Axis, start: usize, len: usize, material: u16) {
    for a in AXES {
        for leaf in (0..PLANE_AREA).step_by(LEAF) {
            let expected =
                if a.as_usize() == axis.as_usize() && (start..start + len).contains(&leaf) {
                    material
                } else {
                    BACKGROUND
                };

            assert_eq!(
                read_leaf(p.plane(a), leaf),
                expected,
                "plane {}, leaf offset {leaf} (filled {start}..{})",
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
                .fill(axis, Morton3D::from_xyz(0, 0, 0), Depth(d as u8), 0x1234);
            assert_only_area_changed(&p, axis, 0, depth_area(d), 0x1234);
        }
    }
}

#[test]
fn fill_at_offset_node_writes_exactly_the_node_at_every_depth() {
    // Node (1,1) at depth d starts at Morton index 3 * size regardless of
    // which axis is dropped. For small nodes this start is not 64-bit
    // aligned, which exercises the partial-word (head/tail/enclave) paths.
    for axis in AXES {
        for d in 1..=TREE_DEPTH {
            let s = PLANE_SIDE >> d;
            let m = Morton3D::from_xyz(s as u8, s as u8, s as u8);
            let size = depth_area(d);
            let start = m.drop_axis(axis).as_usize();
            assert_eq!(start, 3 * size, "test assumption about Morton layout");

            let mut p = Planes::with_background(BACKGROUND);
            p.material().fill(axis, m, Depth(d as u8), 0xBEEF);

            assert_only_area_changed(&p, axis, start, size, 0xBEEF);
        }
    }
}

#[test]
fn fill_handles_extreme_material_values() {
    for material in [0x0000, 0x0001, 0x8000, 0xFFFF, 0x00FF, 0xFF00] {
        let mut p = Planes::with_background(BACKGROUND);
        p.material()
            .fill(AXIS_Y, Morton3D::from_xyz(0, 0, 0), Depth(1), material);
        assert_only_area_changed(&p, AXIS_Y, 0, depth_area(1), material);
    }
}

#[test]
fn fill_overwrites_previous_content_of_the_area() {
    let mut p = Planes::new();
    let axis = AXIS_X;
    p.material()
        .fill(axis, Morton3D::from_xyz(0, 0, 0), Depth(1), 0x1111);
    p.material()
        .fill(axis, Morton3D::from_xyz(0, 0, 0), Depth(2), 0x2222);

    let inner = depth_area(2);
    let outer = depth_area(1);
    for leaf in (0..outer).step_by(LEAF) {
        let expected = if leaf < inner { 0x2222 } else { 0x1111 };
        assert_eq!(read_leaf(p.plane(axis), leaf), expected, "leaf {leaf}");
    }
    assert_eq!(
        read_leaf(p.plane(axis), outer),
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
            p.material().fill(axis, m, Depth(d as u8), 0x4242);

            let offset = m.drop_axis(axis).as_usize();
            assert_eq!(
                collect_runs(&mut p, axis, offset, d),
                vec![(0x4242, coords(offset), depth_area(d))],
                "depth {d}"
            );
        }
    }
}

#[test]
fn filling_a_child_makes_the_parent_decompose() {
    let mut p = Planes::new();
    p.material()
        .fill(AXIS_Z, Morton3D::from_xyz(0, 0, 0), Depth(0), 1);
    p.material()
        .fill(AXIS_Z, Morton3D::from_xyz(0, 0, 0), Depth(2), 2);

    let runs = collect_runs(&mut p, AXIS_Z, 0, 0);
    assert_eq!(runs, reference_runs(p.plane(AXIS_Z), 0, PLANE_AREA));
    assert_eq!(runs[0], (2, coords(0), depth_area(2)));
    assert_eq!(runs.iter().map(|r| r.2).sum::<usize>(), PLANE_AREA);
}

#[test]
fn clear_material_planes_zeroes_all_three_planes() {
    let mut p = Planes::with_background(BACKGROUND);
    p.material().clear_material_planes();
    assert!(p.bits.iter().all(|b| b.not_any()));
}

#[test]
fn clear_material_planes_on_empty_planes_is_a_no_op() {
    let mut p = Planes::new();
    p.material().clear_material_planes();
    assert!(p.bits.iter().all(|b| b.not_any()));
}

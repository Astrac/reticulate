use bitvec::prelude::*;

use crate::{
    Coords3D, DenseWalker, Morton3D, axis::*, coords::Coords2D, face::*,
    meshing::decision_boundaries::DecisionBoundaries, morton_2d::Morton2D,
};

pub struct FaceWalker<F> {
    boundaries: DecisionBoundaries,
    on_face: F,
}

impl<F> FaceWalker<F>
where
    F: FnMut(Morton3D, u8, Face),
{
    pub fn new(f: F) -> FaceWalker<F> {
        FaceWalker {
            on_face: f,
            boundaries: Default::default(),
        }
    }

    fn walk_on_solid(&mut self, axis: Axis, morton: Morton3D, depth: u8) {
        let negative_face = axis.as_negative_face();

        let committed = self.boundaries.flip_deciding_face(axis, morton, depth);

        harvest_faces(committed, depth, &mut |local_face_morton, face_size| {
            let (face_morton, face_depth) = back_project(
                axis,
                morton,
                depth,
                local_face_morton,
                face_size,
                false,
                false,
            );

            (self.on_face)(face_morton, face_depth, negative_face);
        });

        self.boundaries.fill_deciding_face(axis, morton, depth);
    }

    fn walk_on_void(&mut self, axis: Axis, morton: Morton3D, depth: u8) {
        let positive_face = axis.as_positive_face();

        let committed = self.boundaries.deciding_face_mut(axis, morton, depth);

        harvest_faces(committed, depth, &mut |local_face_morton, face_size| {
            let (face_morton, face_depth) = back_project(
                axis,
                morton,
                depth,
                local_face_morton,
                face_size,
                false,
                true,
            );

            (self.on_face)(face_morton, face_depth, positive_face);
        });

        committed.fill(false);
    }
}

impl<Data: PartialEq + Copy, F> DenseWalker<Data> for FaceWalker<F>
where
    F: FnMut(Morton3D, u8, Face),
{
    fn on_branch_start(&mut self, _depth: u8, _morton: Morton3D, _children_mask: u8) {}

    fn on_branch_end(&mut self, _depth: u8, _morton: Morton3D) {
        // TODO: Harvest outermost+ faces
    }

    fn on_leaf(&mut self, depth: u8, morton: Morton3D, _data: &Data) {
        self.walk_on_solid(AXIS_X, morton, depth);
        self.walk_on_solid(AXIS_Y, morton, depth);
        self.walk_on_solid(AXIS_Z, morton, depth);
    }

    fn on_empty(&mut self, depth: u8, morton: Morton3D) {
        self.walk_on_void(AXIS_X, morton, depth);
        self.walk_on_void(AXIS_Y, morton, depth);
        self.walk_on_void(AXIS_Z, morton, depth);
    }
}

/// Reverses the projection from the local boundary quadtree back to 3D space. Returns a
/// 3D morton code and the depth of the identified volume at that code.
///
/// - volume is the absolute coordinates of an cubic volume represented by the given morton code
/// - volume_depth expresses how deep the morton code is relative to the level-0 chunk,
///   i.e. how many times it subdivided
/// - face expresses the 2d morton index of a square face relative to the +X or -X face of
///   the volume
/// - face_size the area of the square face
/// - is_face_pos determines if we're back-projecting from the +X or -X face of the volume
/// - is_seam_pos determines if we're back-projecting on the internal or external face
///   of the volume
fn back_project(
    axis: Axis,
    volume: Morton3D,
    volume_depth: u8,
    face: Morton2D,
    face_size: usize,
    is_face_pos: bool,
    is_seam_pos: bool,
) -> (Morton3D, u8) {
    let Coords3D {
        x: vol_x,
        y: vol_y,
        z: vol_z,
    } = volume.into();
    let Coords2D {
        u: face_u,
        v: face_v,
    } = face.into();

    let face_depth = 6 - (face_size.trailing_zeros() as u8 / 2);
    let scale_shift = (face_depth - volume_depth) as u32;

    let base_x = vol_x << scale_shift;
    let base_y = vol_y << scale_shift;
    let base_z = vol_z << scale_shift;

    let global_u = face_u;
    let global_v = face_v;

    let boundary: u8 = if is_face_pos {
        (1 << scale_shift) - 1
    } else {
        0
    };

    let seam_offset: i8 = match (is_face_pos, is_seam_pos) {
        (true, true) => 1,
        (false, true) => -1,
        _ => 0,
    };

    let coords = match axis {
        AXIS_X => Coords3D {
            x: (base_x + boundary).saturating_add_signed(seam_offset),
            y: base_y + global_u as u8,
            z: base_z + global_v as u8,
        },
        AXIS_Y => Coords3D {
            x: base_x + global_u as u8,
            y: (base_y + boundary).saturating_add_signed(seam_offset),
            z: base_z + global_v as u8,
        },
        AXIS_Z => Coords3D {
            x: base_x + global_u as u8,
            y: base_y + global_v as u8,
            z: (base_z + boundary).saturating_add_signed(seam_offset),
        },
        _ => unreachable!(),
    };

    (coords.into(), face_depth)
}

/// Harvest faces from a bitfield. This function only returns faces that are aligned
/// with an implicit quadtree grid over the bitfield. This means that faces returned
/// will all be squares and they will be aligned with a grid spanned by the size of
/// their side.
///
/// Rules of harvesting:
///   - Depth provides an upper bound only (at most faces this big)
///   - The numbers of 1s we can take must be a power of 2
///   - If we take N bits the index at which we start must be divisible by N
///   - The position of the first 1 determines the maximum N
///   - The biggest power of 2 that divides the position of the first bit is an
///     upper bound of face size
///   - The number of 1s we have in the block is another upper bound of face size
///
/// The on_face callback is provided the size of any face found and its morton-2d
/// index relative to the bitfield.
fn harvest_faces<F>(bitfield: &BitSlice<u64>, depth: u8, on_face: &mut F)
where
    F: FnMut(Morton2D, usize),
{
    let committed_len = bitfield.len();
    let mut current = bitfield.first_one().unwrap_or(committed_len);
    let mut ones_in_block = bitfield[current..].leading_ones();

    while current < committed_len {
        let face_size = (4096 >> (depth * 2) as usize)
            .min(biggest_power_of_4_dividing(current).unwrap_or(usize::MAX))
            .min(biggest_power_of_4_in(ones_in_block));

        if face_size == 0 {
            return;
        }

        on_face(Morton2D::from(current as u32), face_size);

        let next = current + face_size;

        current = bitfield[next..]
            .first_one()
            .map(|n| next + n)
            .unwrap_or(committed_len);

        ones_in_block = bitfield[current..].leading_ones();
    }
}

fn biggest_power_of_4_in(n: usize) -> usize {
    if n == 0 {
        return 0;
    }

    let msb_index = usize::BITS - 1 - n.leading_zeros();
    let even_index = msb_index & !1;
    1 << even_index
}

#[inline(always)]
const fn biggest_power_of_4_dividing(n: usize) -> Option<usize> {
    if n == 0 {
        None
    } else {
        let tz = n.trailing_zeros();
        Some(1 << (tz & !1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harvest_single_bits() {
        let mut bits = bitvec![u64, Lsb0; 0; 16];
        bits.set(1, true);
        bits.set(3, true);
        bits.set(7, true);

        let mut harvested = Vec::new();
        harvest_faces(&bits, 0, &mut |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(1, 1), (3, 1), (7, 1)]);
    }

    #[test]
    fn test_harvest_perfect_power_of_two_block() {
        let mut bits = bitvec![u64, Lsb0; 0; 16];
        bits[4..8].fill(true);

        let mut harvested = Vec::new();
        harvest_faces(&bits, 0, &mut |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(4, 4)]);
    }

    #[test]
    fn test_harvest_alignment_constraint() {
        let mut bits = bitvec![u64, Lsb0; 0; 16];
        bits[4..12].fill(true);

        let mut harvested = Vec::new();
        harvest_faces(&bits, 0, &mut |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(4, 4), (8, 4)]);
    }

    #[test]
    fn test_harvest_large_imperfect_block() {
        let mut bits = bitvec![u64, Lsb0; 0; 24];
        bits[8..18].fill(true);

        let mut harvested = Vec::new();
        harvest_faces(&bits, 0, &mut |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(8, 4), (12, 4), (16, 1), (17, 1)]);
    }

    #[test]
    fn test_harvest_depth_limit() {
        let mut bits = bitvec![u64, Lsb0; 0; 16];
        bits[0..8].fill(true);

        let mut harvested = Vec::new();
        harvest_faces(&bits, 5, &mut |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(0, 4), (4, 4)]);
    }

    #[test]
    fn test_harvest_empty_bitfield() {
        let bits = bitvec![u64, Lsb0; 0; 64];

        let mut harvested = Vec::new();
        harvest_faces(&bits, 0, &mut |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert!(harvested.is_empty());
    }

    #[test]
    fn test_internal_face_no_seam_pos_x() {
        let volume = Morton3D::from(Coords3D { x: 0, y: 0, z: 0 });
        let face = Morton2D::from(Coords2D { u: 1, v: 2 });

        let (result_morton, depth) = back_project(AXIS_X, volume, 2, face, 16, true, false);

        let coords = Coords3D::from(result_morton);

        assert_eq!(coords.x, 3);
        assert_eq!(coords.y, 1);
        assert_eq!(coords.z, 2);
        assert_eq!(depth, 4);
    }

    #[test]
    fn test_internal_face_no_seam_neg_x() {
        let volume = Morton3D::from(Coords3D { x: 32, y: 0, z: 0 });
        let face = Morton2D::from(Coords2D { u: 0, v: 0 });

        let (result_morton, depth) = back_project(AXIS_X, volume, 2, face, 256, false, false);

        let coords = Coords3D::from(result_morton);

        assert_eq!(coords.x, 32);
        assert_eq!(depth, 2);
    }

    #[test]
    fn test_external_seam_pos_x() {
        let volume = Morton3D::from(Coords3D { x: 0, y: 0, z: 0 });
        let face = Morton2D::from(Coords2D { u: 0, v: 0 });

        let (result_morton, _) = back_project(AXIS_X, volume, 2, face, 256, true, true);

        let coords = Coords3D::from(result_morton);
        assert_eq!(coords.x, 1);
    }

    #[test]
    fn test_external_seam_neg_x() {
        let volume = Morton3D::from(Coords3D { x: 32, y: 0, z: 0 });
        let face = Morton2D::from(Coords2D { u: 0, v: 0 });

        let (result_morton, _) = back_project(AXIS_X, volume, 2, face, 256, false, true);

        let coords = Coords3D::from(result_morton);
        assert_eq!(coords.x, 31);
    }

    #[test]
    fn test_max_depth_resolution() {
        let volume = Morton3D::from(Coords3D { x: 0, y: 0, z: 0 });
        let face = Morton2D::from(Coords2D { u: 1, v: 1 });

        let (result_morton, depth) = back_project(AXIS_X, volume, 3, face, 1, false, false);

        let coords = Coords3D::from(result_morton);

        assert_eq!(coords.y, 1);
        assert_eq!(coords.z, 1);
        assert_eq!(depth, 6);
    }
}

use bitvec::prelude::*;

use crate::{
    DenseWalker, Depth, MeshFace, Morton3D, axis::*,
    meshing::decision_boundaries::DecisionBoundaries, morton_2d::Morton2D,
};

pub struct FaceWalker<F> {
    boundaries: DecisionBoundaries,
    on_face: F,
}

impl<F> FaceWalker<F>
where
    F: FnMut(MeshFace),
{
    pub fn new(f: F) -> FaceWalker<F> {
        FaceWalker {
            on_face: f,
            boundaries: Default::default(),
        }
    }

    fn walk_on_solid(&mut self, axis: Axis, morton: Morton3D, depth: u8) {
        let deciding = self.boundaries.flip_deciding_face(axis, morton, depth);

        harvest_faces(deciding, depth, &mut |local_face_morton, face_size| {
            let mesh_face = MeshFace::from_morton_back_projection(
                axis.as_negative_face(),
                morton,
                Depth(depth),
                local_face_morton,
                face_size,
            );

            (self.on_face)(mesh_face);
        });

        self.boundaries.fill_deciding_face(axis, morton, depth);
    }

    fn walk_on_void(&mut self, axis: Axis, morton: Morton3D, depth: u8) {
        let deciding = self.boundaries.deciding_face_mut(axis, morton, depth);

        harvest_faces(deciding, depth, &mut |local_face_morton, face_size| {
            let mesh_face = MeshFace::from_morton_back_projection(
                axis.as_positive_face(),
                morton,
                Depth(depth),
                local_face_morton,
                face_size,
            );

            (self.on_face)(mesh_face);
        });

        deciding.fill(false);
    }
}

impl<Data: PartialEq + Copy, F> DenseWalker<Data> for FaceWalker<F>
where
    F: FnMut(MeshFace),
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
    let deciding_len = bitfield.len();
    let mut current = bitfield.first_one().unwrap_or(deciding_len);
    let mut ones_in_block = bitfield[current..].leading_ones();

    while current < deciding_len {
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
            .unwrap_or(deciding_len);

        ones_in_block = bitfield[current..].leading_ones();
    }
}

#[inline(always)]
const fn biggest_power_of_4_in(n: usize) -> usize {
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
}

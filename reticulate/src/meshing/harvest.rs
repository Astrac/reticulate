use bitvec::prelude::*;

use crate::{Depth, Morton2D, meshing::decision_boundaries::BOUNDARY_PLANE_SIZE};

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
pub fn harvest_faces<F>(bitfield: &BitSlice<u64>, depth: Depth, mut on_face: F)
where
    F: FnMut(Morton2D, usize),
{
    let harvestable_bits = bitfield.len();
    let mut current = bitfield.first_one().unwrap_or(harvestable_bits);

    while current < harvestable_bits {
        let ones_in_block = bitfield[current..].leading_ones();

        let face_size = (BOUNDARY_PLANE_SIZE >> (depth.as_usize() * 2))
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
            .unwrap_or(harvestable_bits);
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
        harvest_faces(&bits, Depth(0), |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(1, 1), (3, 1), (7, 1)]);
    }

    #[test]
    fn test_harvest_perfect_power_of_two_block() {
        let mut bits = bitvec![u64, Lsb0; 0; 16];
        bits[4..8].fill(true);

        let mut harvested = Vec::new();
        harvest_faces(&bits, Depth(0), |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(4, 4)]);
    }

    #[test]
    fn test_harvest_alignment_constraint() {
        let mut bits = bitvec![u64, Lsb0; 0; 16];
        bits[4..12].fill(true);

        let mut harvested = Vec::new();
        harvest_faces(&bits, Depth(0), |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(4, 4), (8, 4)]);
    }

    #[test]
    fn test_harvest_large_imperfect_block() {
        let mut bits = bitvec![u64, Lsb0; 0; 24];
        bits[8..18].fill(true);

        let mut harvested = Vec::new();
        harvest_faces(&bits, Depth(0), |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert_eq!(harvested, vec![(8, 4), (12, 4), (16, 1), (17, 1)]);
    }

    #[test]
    fn test_harvest_empty_bitfield() {
        let bits = bitvec![u64, Lsb0; 0; 64];

        let mut harvested = Vec::new();
        harvest_faces(&bits, Depth(0), |morton, face_size| {
            harvested.push((morton.as_u32(), face_size));
        });

        assert!(harvested.is_empty());
    }
}

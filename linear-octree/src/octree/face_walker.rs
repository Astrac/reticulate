use bitvec::prelude::*;

use crate::{
    coords::Coords,
    morton::Morton,
    octree::Walker,
    {axis::*, face::*},
};

const BOUNDARY_BITS_COUNT: usize = 64 * 64;

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct Boundaries {
    x_out_pos: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    y_out_pos: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    z_out_pos: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    x_out_neg: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    y_out_neg: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    z_out_neg: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    x_inn_pos: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    y_inn_pos: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    z_inn_pos: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    x_inn_neg: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    y_inn_neg: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
    z_inn_neg: BitArr!(for BOUNDARY_BITS_COUNT, in u64, Lsb0),
}

#[allow(dead_code)]
pub struct FaceWalker<F> {
    on_face: F,
    boundaries: Boundaries,
    stack: Vec<Boundaries>,
}

impl<F> FaceWalker<F> {
    pub fn new(f: F) -> FaceWalker<F> {
        FaceWalker {
            on_face: f,
            boundaries: Boundaries::default(),
            stack: Vec::new(),
        }
    }
}

impl<Data, F> Walker<Data> for FaceWalker<F>
where
    F: FnMut(Morton, u8, Face),
{
    fn on_branch_start(&mut self, _depth: u8, octant: u8, _morton: Morton, _children_mask: u8) {
        let current_boundaries = &self.boundaries;
        let mut new_boundaries = Boundaries::default();

        if octant & AXIS_X.as_u8() == 0 {
            new_boundaries.x_out_pos = current_boundaries.x_inn_neg;
            new_boundaries.x_out_neg = current_boundaries.x_out_neg;
        } else {
            new_boundaries.x_out_neg = current_boundaries.x_inn_pos;
            new_boundaries.x_out_pos = current_boundaries.x_out_pos;
        }

        if octant & AXIS_Y.as_u8() == 0 {
            new_boundaries.y_out_pos = current_boundaries.y_inn_neg;
            new_boundaries.y_out_neg = current_boundaries.y_out_neg;
        } else {
            new_boundaries.y_out_neg = current_boundaries.y_inn_pos;
            new_boundaries.y_out_pos = current_boundaries.y_out_pos;
        }

        if octant & AXIS_Z.as_u8() == 0 {
            new_boundaries.z_out_pos = current_boundaries.z_inn_neg;
            new_boundaries.z_out_neg = current_boundaries.z_out_neg;
        } else {
            new_boundaries.z_out_neg = current_boundaries.z_inn_pos;
            new_boundaries.z_out_pos = current_boundaries.z_out_pos;
        }

        self.stack.push(current_boundaries.clone());
        self.boundaries = new_boundaries;
    }

    fn on_branch_end(&mut self, _depth: u8, octant: u8, _morton: Morton) {
        let current_boundaries = &self.boundaries;

        harvest_faces(
            Some(&current_boundaries.x_inn_pos),
            Some(&current_boundaries.x_inn_neg),
            _morton.as_u32(),
            _depth,
            AXIS_X,
            &mut self.on_face,
        );

        harvest_faces(
            Some(&current_boundaries.y_inn_pos),
            Some(&current_boundaries.y_inn_neg),
            _morton.as_u32(),
            _depth,
            AXIS_Y,
            &mut self.on_face,
        );

        harvest_faces(
            Some(&current_boundaries.z_inn_pos),
            Some(&current_boundaries.z_inn_neg),
            _morton.as_u32(),
            _depth,
            AXIS_Z,
            &mut self.on_face,
        );

        if let Some(mut previous) = self.stack.pop() {
            if octant & AXIS_X.as_u8() == 0 {
                previous.x_inn_neg = current_boundaries.x_out_pos;
                previous.x_out_neg = current_boundaries.x_out_neg;
            } else {
                previous.x_inn_pos = current_boundaries.x_out_neg;
                previous.x_out_pos = current_boundaries.x_out_pos;
            }

            if octant & AXIS_Y.as_u8() == 0 {
                previous.y_inn_neg = current_boundaries.y_out_pos;
                previous.y_out_neg = current_boundaries.y_out_neg;
            } else {
                previous.y_inn_pos = current_boundaries.y_out_neg;
                previous.y_out_pos = current_boundaries.y_out_pos;
            }

            if octant & AXIS_Z.as_u8() == 0 {
                previous.z_inn_neg = current_boundaries.z_out_pos;
                previous.z_out_neg = current_boundaries.z_out_neg;
            } else {
                previous.z_inn_pos = current_boundaries.z_out_neg;
                previous.z_out_pos = current_boundaries.z_out_pos;
            }

            self.boundaries = previous;
        }

        let current_boundaries = &self.boundaries;
        if _depth == 0 {
            harvest_faces(
                Some(&current_boundaries.x_out_pos),
                None,
                _morton.as_u32(),
                _depth,
                AXIS_X,
                &mut self.on_face,
            );

            harvest_faces(
                None,
                Some(&current_boundaries.x_out_neg),
                _morton.as_u32(),
                _depth,
                AXIS_X,
                &mut self.on_face,
            );

            harvest_faces(
                Some(&current_boundaries.y_out_pos),
                None,
                _morton.as_u32(),
                _depth,
                AXIS_Y,
                &mut self.on_face,
            );

            harvest_faces(
                None,
                Some(&current_boundaries.y_out_neg),
                _morton.as_u32(),
                _depth,
                AXIS_Y,
                &mut self.on_face,
            );
            harvest_faces(
                Some(&current_boundaries.z_out_pos),
                None,
                _morton.as_u32(),
                _depth,
                AXIS_Z,
                &mut self.on_face,
            );

            harvest_faces(
                None,
                Some(&current_boundaries.z_out_neg),
                _morton.as_u32(),
                _depth,
                AXIS_Z,
                &mut self.on_face,
            );
        }
    }

    fn on_leaf(&mut self, depth: u8, octant: u8, morton: Morton, _data: &Data) {
        let leaf_size = 4096usize >> (2 * depth);
        let current_boundaries = &mut self.boundaries;

        let leaf_start = (morton.project_on_plane(AXIS_X)) as usize * leaf_size;
        let leaf_end = leaf_start + leaf_size;
        if octant & AXIS_X.as_u8() == 0 {
            let _ = !(&mut current_boundaries.x_out_neg[leaf_start..leaf_end]);
            let _ = !(&mut current_boundaries.x_inn_neg[leaf_start..leaf_end]);
        } else {
            let _ = !(&mut current_boundaries.x_out_pos[leaf_start..leaf_end]);
            let _ = !(&mut current_boundaries.x_inn_pos[leaf_start..leaf_end]);
        }

        let leaf_start = (morton.project_on_plane(AXIS_Y)) as usize * leaf_size;
        let leaf_end = leaf_start + leaf_size;
        if octant & AXIS_Y.as_u8() == 0 {
            let _ = !(&mut current_boundaries.y_out_neg[leaf_start..leaf_end]);
            let _ = !(&mut current_boundaries.y_inn_neg[leaf_start..leaf_end]);
        } else {
            let _ = !(&mut current_boundaries.y_out_pos[leaf_start..leaf_end]);
            let _ = !(&mut current_boundaries.y_inn_pos[leaf_start..leaf_end]);
        }

        let leaf_start = (morton.project_on_plane(AXIS_Z) as usize) * leaf_size;
        let leaf_end = leaf_start + leaf_size;
        if octant & AXIS_Z.as_u8() == 0 {
            let _ = !(&mut current_boundaries.z_out_neg[leaf_start..leaf_end]);
            let _ = !(&mut current_boundaries.z_inn_neg[leaf_start..leaf_end]);
        } else {
            let _ = !(&mut current_boundaries.z_out_pos[leaf_start..leaf_end]);
            let _ = !(&mut current_boundaries.z_inn_pos[leaf_start..leaf_end]);
        }
    }
}

pub fn harvest_faces(
    positive_bitfield: Option<&BitSlice<u64, Lsb0>>,
    negative_bitfield: Option<&BitSlice<u64, Lsb0>>,
    morton_3d: u32,
    depth: u8,
    axis: Axis,
    on_face: &mut impl FnMut(Morton, u8, Face),
) {
    if let Some(p) = positive_bitfield
        && let Some(n) = negative_bitfield
    {
        harvest_inner_faces(p, n, morton_3d, depth, axis, on_face);
    }
}

fn harvest_inner_faces(
    positive_bitfield: &BitSlice<u64>,
    negative_bitfield: &BitSlice<u64>,
    morton_3d: u32,
    depth: u8,
    axis: Axis,
    on_face_raw: &mut impl FnMut(Morton, u8, Face),
) {
    let mut on_face = |m, d, f| {
        println!("ON_FACE: {:?}, {d}, {f:?}", Coords::from(m));
        (on_face_raw)(m, d, f);
    };
    let mut stack = Vec::new();

    let base_morton_2d = Morton::from(morton_3d).project_on_plane(axis) as usize;

    for i in 0..4 {
        stack.push((
            (base_morton_2d << 2) | i, // This is not affecting the result
            append_2d_child(axis, morton_3d, i as u32, true),
            append_2d_child(axis, morton_3d, i as u32, false),
            depth + 1,
        ));
    }

    while let Some((
        face_morton_2d,
        face_owner_morton_3d_positive,
        face_owner_morton_3d_negative,
        current_depth,
    )) = stack.pop()
    {
        let face_size = 4096usize >> (current_depth * 2);
        let face_start = face_morton_2d * face_size;
        let face_end = face_start + face_size;

        println!(
            "-> Depth: {current_depth}; Face: {face_start}..{face_end}; Morton2D: {face_morton_2d}; Owner POS/NEG: {:?} {:?}; Leaf: {:?}",
            Coords::from(Morton::from(face_owner_morton_3d_positive)),
            Coords::from(Morton::from(face_owner_morton_3d_negative)),
            decode_morton_2d(face_morton_2d as u32)
        );

        let positive_all_1s = positive_bitfield[face_start..face_end].all();
        let positive_all_0s = positive_bitfield[face_start..face_end].not_any();
        let negative_all_1s = negative_bitfield[face_start..face_end].all();
        let negative_all_0s = negative_bitfield[face_start..face_end].not_any();

        // cull common
        if positive_all_1s == negative_all_1s && positive_all_1s == true {
            continue;
        }
        // skip empty
        if positive_all_0s && negative_all_0s {
            continue;
        }

        // emit negative face of positive octant
        if positive_all_1s && negative_all_0s {
            on_face(
                face_owner_morton_3d_positive.into(),
                current_depth,
                axis.as_negative_face(),
            );
            continue;
        }

        // emit positive face of negative octant
        if positive_all_0s && negative_all_1s {
            on_face(
                face_owner_morton_3d_negative.into(),
                current_depth,
                axis.as_positive_face(),
            );
            continue;
        }

        for i in 0..4 {
            stack.push((
                (face_morton_2d << 2) | i,
                append_2d_child(axis, face_owner_morton_3d_positive, i as u32, false),
                append_2d_child(axis, face_owner_morton_3d_negative, i as u32, true),
                current_depth + 1,
            ));
        }
    }
}

// TODO: Move from here
#[inline(always)]
const fn append_2d_child(axis: Axis, parent_3d: u32, child_2d: u32, axis_bit: bool) -> u32 {
    let bit0 = (child_2d & 1) as u32;
    let bit1 = ((child_2d >> 1) & 1) as u32;
    let axis_bit = axis_bit as u32;
    let shifted_parent = parent_3d << 3;

    match axis {
        AXIS_X => shifted_parent | (axis_bit << 0) | (bit0 << 1) | (bit1 << 2),
        AXIS_Y => shifted_parent | (bit0 << 0) | (axis_bit << 1) | (bit1 << 2),
        AXIS_Z => shifted_parent | (bit0 << 0) | (bit1 << 1) | (axis_bit << 2),
        _ => unreachable!(),
    }
}

fn decode_morton_2d(morton: u32) -> (u32, u32) {
    let x = compact_1s(morton as u64);
    let y = compact_1s((morton >> 1) as u64);
    (x, y)
}

fn compact_1s(mut x: u64) -> u32 {
    x &= 0x5555555555555555;
    x = (x ^ (x >> 1)) & 0x3333333333333333;
    x = (x ^ (x >> 2)) & 0x0f0f0f0f0f0f0f0f;
    x = (x ^ (x >> 4)) & 0x00ff00ff00ff00ff;
    x = (x ^ (x >> 8)) & 0x0000ffff0000ffff;
    x = (x ^ (x >> 16)) & 0x00000000ffffffff;
    x as u32
}

// TODO: Move from here
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_2d_child_axis_x() {
        let parent = 0b101;
        let child_2d = 0b10;
        let is_positive = true;

        let result = append_2d_child(AXIS_X, parent, child_2d, is_positive);
        assert_eq!(result, 0b101_101);
    }

    #[test]
    fn test_append_2d_child_axis_y() {
        let parent = 0b001;
        let child_2d = 0b11;
        let is_positive = false;

        let result = append_2d_child(AXIS_Y, parent, child_2d, is_positive);
        assert_eq!(result, 0b001_101);
    }

    #[test]
    fn test_append_2d_child_axis_z() {
        let parent = 0b111;
        let child_2d = 0b01;
        let is_positive = true;

        let result = append_2d_child(AXIS_Z, parent, child_2d, is_positive);
        assert_eq!(result, 0b111_101);
    }
}

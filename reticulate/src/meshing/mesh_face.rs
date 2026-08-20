use crate::{Coords2D, Coords3D, Depth, Morton2D, Morton3D, axis::*, face::*};

#[derive(Clone, Copy, Debug)]
pub struct MeshFace {
    pub face: Face,
    pub coords: Coords3D,
    pub width: usize,
    pub height: usize,
    pub subdivisions: u32,
}

impl MeshFace {
    pub fn new(face: Face) -> MeshFace {
        MeshFace {
            face,
            coords: Default::default(),
            width: Default::default(),
            height: Default::default(),
            subdivisions: Default::default(),
        }
    }

    pub fn from_morton_back_projection(
        face: Face,
        morton_3d: Morton3D,
        depth_3d: Depth,
        morton_2d: Morton2D,
        face_size: usize,
    ) -> MeshFace {
        let face_depth = face_size.trailing_zeros() >> 1;
        let side_length = 1u32 << face_depth;
        let coords_2d = Coords2D::from(morton_2d);
        let mut coords = Coords3D::from_morton(morton_3d, depth_3d);

        // if face == FACE_Z_P {
        //     println!(
        //         "{face:?} {:?} {coords:?} {depth_3d:?} {morton_2d:?} {face_size} {face_depth} {coords_2d:?}",
        //         Coords3D::from_morton_unscaled(morton_3d),
        //     );
        // }

        let (dx, dy, dz) = match face.axis() {
            AXIS_X => (0, coords_2d.u, coords_2d.v),
            AXIS_Y => (coords_2d.u, 0, coords_2d.v),
            AXIS_Z => (coords_2d.u, coords_2d.v, 0),
            _ => unreachable!(),
        };

        coords.x = coords.x.saturating_add_signed(dx as i8);
        coords.y = coords.y.saturating_add_signed(dy as i8);
        coords.z = coords.z.saturating_add_signed(dz as i8);

        MeshFace {
            face,
            coords,
            width: side_length as usize,
            height: side_length as usize,
            subdivisions: 0,
        }
    }
}

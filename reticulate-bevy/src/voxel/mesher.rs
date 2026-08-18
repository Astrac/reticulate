use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use reticulate::{Coords3D, Morton3D, face::Face};

const VERTICES: [Vec3; 8] = [
    Vec3::new(0.5, 0.5, 0.5),
    Vec3::new(-0.5, 0.5, 0.5),
    Vec3::new(0.5, -0.5, 0.5),
    Vec3::new(-0.5, -0.5, 0.5),
    Vec3::new(0.5, 0.5, -0.5),
    Vec3::new(-0.5, 0.5, -0.5),
    Vec3::new(0.5, -0.5, -0.5),
    Vec3::new(-0.5, -0.5, -0.5),
];

const FACES: [(Vec3, [usize; 4]); 6] = [
    (Vec3::new(1.0, 0.0, 0.0), [0, 2, 4, 6]),  // +X
    (Vec3::new(0.0, 1.0, 0.0), [0, 1, 4, 5]),  // +Y
    (Vec3::new(0.0, 0.0, 1.0), [0, 1, 2, 3]),  // +Z
    (Vec3::new(-1.0, 0.0, 0.0), [1, 3, 5, 7]), // -X
    (Vec3::new(0.0, -1.0, 0.0), [2, 3, 6, 7]), // -Y
    (Vec3::new(0.0, 0.0, -1.0), [4, 5, 6, 7]), // -Z
];

#[inline(always)]
const fn face_index(id: Face) -> usize {
    let mask = id.face_mask() as u32;
    let sign_offset = ((mask & 0b1000) >> 1) - ((mask & 0b1000) >> 3);
    (mask.trailing_zeros() + sign_offset) as usize
}

#[derive(Resource)]
pub struct Mesher {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    colors: Vec<Vec4>,
    indices: Vec<u32>,
}

const INITIAL_CAPACITY: usize = 8000;

impl Mesher {
    pub fn new() -> Mesher {
        Mesher {
            positions: Vec::with_capacity(INITIAL_CAPACITY),
            normals: Vec::with_capacity(INITIAL_CAPACITY),
            colors: Vec::with_capacity(INITIAL_CAPACITY),
            indices: Vec::with_capacity(INITIAL_CAPACITY),
        }
    }

    pub fn on_face_handler(&mut self) -> impl FnMut(Morton3D, u8, Face) {
        |mut morton: Morton3D, depth: u8, face_id: Face| {
            let mask = (1u32 << ((depth as u32) * 3)) - 1;
            morton.mask(mask);
            let prescale_coords: Coords3D = morton.into();
            let scale = 64.0 / f32::powi(2.0, depth as i32);
            let center = Vec3::new(
                prescale_coords.x as f32,
                prescale_coords.y as f32,
                prescale_coords.z as f32,
            ) * scale;

            self.add_face(center, scale, face_id);
        }
    }

    pub fn build_mesh(&mut self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        mesh.insert_indices(Indices::U32(self.indices.clone()));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors.clone());

        self.positions.clear();
        self.normals.clear();
        self.colors.clear();
        self.indices.clear();

        mesh
    }

    fn add_face(&mut self, center: Vec3, scale: f32, face_id: Face) {
        let (normal, vertex_indices) = FACES[face_index(face_id)];
        let start_index = self.positions.len() as u32;

        for vertex_index in vertex_indices {
            self.positions
                .push(center + (VERTICES[vertex_index] * scale) + (0.5 * scale));
            self.normals.push(normal);
        }

        self.indices.extend_from_slice(&[
            start_index + 0,
            start_index + 1,
            start_index + 2,
            start_index + 1,
            start_index + 2,
            start_index + 3,
        ]);

        self.colors.extend_from_slice(&[
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::new(0.0, 1.0, 0.0, 1.0),
            Vec4::new(0.0, 0.0, 1.0, 1.0),
            Vec4::new(0.0, 1.0, 1.0, 1.0),
        ]);
    }
}

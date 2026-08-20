use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use reticulate::{Coords3D, MeshFace, axis::*, face::*};

const VERTICES: [Vec3; 8] = [
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(1.0, 1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(1.0, 0.0, 1.0),
    Vec3::new(0.0, 1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
];

const FACES: [(Vec3, [usize; 4]); 6] = [
    (Vec3::new(1.0, 0.0, 0.0), [0, 2, 4, 6]),
    (Vec3::new(0.0, 1.0, 0.0), [4, 5, 0, 1]),
    (Vec3::new(0.0, 0.0, 1.0), [0, 1, 2, 3]),
    (Vec3::new(-1.0, 0.0, 0.0), [0, 4, 2, 6]),
    (Vec3::new(0.0, -1.0, 0.0), [4, 0, 5, 1]),
    (Vec3::new(0.0, 0.0, -1.0), [0, 2, 1, 3]),
];

#[inline(always)]
const fn face_index(id: Face) -> usize {
    let mask = id.face_mask() as u32;
    let sign_offset = ((mask & 0b1000) >> 2) | ((mask & 0b1000) >> 3);
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

    pub fn on_mesh_face_handler(&mut self) -> impl FnMut(MeshFace) {
        |mesh_face: MeshFace| {
            if mesh_face.face.axis() == AXIS_X {
                println!("{mesh_face:?}");
            }
            let start_index = self.positions.len() as u32;
            let (normal, vertex_indices) = FACES[face_index(mesh_face.face)];
            let width = mesh_face.width as f32;
            let height = mesh_face.height as f32;
            let Coords3D {
                x: fx,
                y: fy,
                z: fz,
            } = mesh_face.coords;

            let scale = match mesh_face.face.axis() {
                AXIS_X => Vec3::new(1.0, width, height),
                AXIS_Y => Vec3::new(width, 1.0, height),
                AXIS_Z => Vec3::new(width, height, 1.0),
                _ => unreachable!(),
            };

            for vertex_index in vertex_indices {
                let position =
                    Vec3::new(fx as f32, fy as f32, fz as f32) + (VERTICES[vertex_index] * scale);
                self.positions.push(position);
                self.normals.push(normal);
            }

            self.indices.extend_from_slice(&[
                start_index + 0,
                start_index + 1,
                start_index + 3,
                start_index + 0,
                start_index + 3,
                start_index + 2,
            ]);

            self.colors.extend_from_slice(&[
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
                Vec4::new(0.0, 0.0, 1.0, 1.0),
                Vec4::new(0.0, 1.0, 1.0, 1.0),
            ]);
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
}

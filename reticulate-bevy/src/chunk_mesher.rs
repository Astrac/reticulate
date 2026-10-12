use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use reticulate::{Coords3D, DenseWalk, MaterialId, Mesher};

const INITIAL_CAPACITY: usize = 1500;
const NORMALS: [Vec3; 6] = [
    Vec3::new(1., 0., 0.),
    Vec3::new(0., 1., 0.),
    Vec3::new(0., 0., 1.),
    Vec3::new(-1., 0., 0.),
    Vec3::new(0., -1., 0.),
    Vec3::new(0., 0., -1.),
];

pub struct ChunkMesher {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    colors: Vec<Vec4>,
    indices: Vec<u32>,
    mesher: Mesher,
}

impl ChunkMesher {
    pub fn new() -> ChunkMesher {
        ChunkMesher {
            positions: Vec::with_capacity(INITIAL_CAPACITY),
            normals: Vec::with_capacity(INITIAL_CAPACITY),
            colors: Vec::with_capacity(INITIAL_CAPACITY),
            indices: Vec::with_capacity(INITIAL_CAPACITY),
            mesher: Mesher::default(),
        }
    }

    pub fn mesh(&mut self, tree: &impl DenseWalk<MaterialId>) -> Mesh {
        let mut next_index = 0;

        self.mesher.reset();
        self.mesher.make_triangles(
            tree,
            |m| *m,
            |t| {
                let [v1, v2, v3] = t.outward_vertices();
                self.positions.push(to_position(v1));
                self.positions.push(to_position(v2));
                self.positions.push(to_position(v3));

                self.indices.push(next_index);
                self.indices.push(next_index + 1);
                self.indices.push(next_index + 2);

                self.normals.push(to_normal(t.face()));
                self.normals.push(to_normal(t.face()));
                self.normals.push(to_normal(t.face()));

                self.colors.push(Vec4::new(1., 0., 0., 1.));
                self.colors.push(Vec4::new(0., 1., 0., 1.));
                self.colors.push(Vec4::new(0., 0., 1., 1.));

                next_index += 3;
            },
        );

        self.take_mesh()
    }

    fn take_mesh(&mut self) -> Mesh {
        let cap = self.positions.capacity();
        let positions = std::mem::replace(&mut self.positions, Vec::with_capacity(cap));
        let normals = std::mem::replace(&mut self.normals, Vec::with_capacity(cap));
        let colors = std::mem::replace(&mut self.colors, Vec::with_capacity(cap));

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh
    }
}

fn to_normal(face: reticulate::Face) -> Vec3 {
    NORMALS[face.as_usize()]
}

fn to_position(coords: Coords3D) -> Vec3 {
    Vec3::new(coords.x as f32, coords.y as f32, coords.z as f32)
}

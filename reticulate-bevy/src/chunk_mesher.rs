use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use reticulate::{Coords3D, DenseWalk, Face, MaterialId, Mesher};

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
    uvs: Vec<[f32; 2]>,
    materials: Vec<[f32; 2]>,
    mesher: Mesher,
}

impl ChunkMesher {
    pub fn new() -> ChunkMesher {
        ChunkMesher {
            positions: Vec::with_capacity(INITIAL_CAPACITY),
            normals: Vec::with_capacity(INITIAL_CAPACITY),
            uvs: Vec::with_capacity(INITIAL_CAPACITY),
            materials: Vec::with_capacity(INITIAL_CAPACITY),
            mesher: Mesher::default(),
        }
    }

    pub fn mesh(&mut self, tree: &impl DenseWalk<MaterialId>) -> Mesh {
        self.mesher.reset();
        self.mesher.make_triangles(
            tree,
            |m| *m,
            |t| {
                let face = t.face();

                for v in t.outward_vertices() {
                    let pos = to_position(v);
                    self.positions.push(pos);
                    self.normals.push(to_normal(face));
                    self.uvs.push(face_uv(pos, face));
                    self.materials.push([t.material() as f32, 0.0]);
                }
            },
        );

        self.take_mesh()
    }

    fn take_mesh(&mut self) -> Mesh {
        let cap = self.positions.capacity();
        let positions = std::mem::replace(&mut self.positions, Vec::with_capacity(cap));
        let normals = std::mem::replace(&mut self.normals, Vec::with_capacity(cap));
        let uvs = std::mem::replace(&mut self.uvs, Vec::with_capacity(cap));
        let materials = std::mem::replace(&mut self.materials, Vec::with_capacity(cap));

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, materials);
        mesh
    }
}

fn face_uv(p: Vec3, face: Face) -> [f32; 2] {
    match face.axis() {
        reticulate::Axis::X => [p.z, p.y],
        reticulate::Axis::Y => [p.x, p.z],
        reticulate::Axis::Z => [p.x, p.y],
    }
}

fn to_normal(face: Face) -> Vec3 {
    NORMALS[face.as_usize()]
}

fn to_position(coords: Coords3D) -> Vec3 {
    Vec3::new(coords.x as f32, coords.y as f32, coords.z as f32)
}

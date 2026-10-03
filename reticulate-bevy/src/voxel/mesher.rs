use bevy::prelude::*;
use reticulate::{Coords3D, DenseWalk, Triangulator};

const INITIAL_CAPACITY: usize = 8000;
const NORMALS: [Vec3; 6] = [
    Vec3::new(1., 0., 0.),
    Vec3::new(0., 1., 0.),
    Vec3::new(0., 0., 1.),
    Vec3::new(-1., 0., 0.),
    Vec3::new(0., -1., 0.),
    Vec3::new(0., 0., -1.),
];

#[derive(Resource)]
pub struct Mesher {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub colors: Vec<Vec4>,
    pub indices: Vec<u32>,
}

impl Mesher {
    pub fn new() -> Mesher {
        Mesher {
            positions: Vec::with_capacity(INITIAL_CAPACITY),
            normals: Vec::with_capacity(INITIAL_CAPACITY),
            colors: Vec::with_capacity(INITIAL_CAPACITY),
            indices: Vec::with_capacity(INITIAL_CAPACITY),
        }
    }

    pub fn build_mesh(&mut self, octree: impl DenseWalk<()>) {
        let mut next_index = 0;

        let mut triangulator = Triangulator::default();

        triangulator.triangulate(
            &octree,
            |_| 1,
            |t| {
                let [v1, v2, v3] = t.outward_vertices();
                self.positions.push(to_position(v1));
                self.positions.push(to_position(v2));
                self.positions.push(to_position(v3));

                self.indices.push(next_index);
                self.indices.push(next_index + 1);
                self.indices.push(next_index + 2);

                self.normals.push(to_normal(t.face));
                self.normals.push(to_normal(t.face));
                self.normals.push(to_normal(t.face));

                self.colors.push(Vec4::new(1., 0., 0., 1.));
                self.colors.push(Vec4::new(0., 1., 0., 1.));
                self.colors.push(Vec4::new(0., 0., 1., 1.));

                next_index += 3;
            },
        );
    }
}

fn to_normal(face: reticulate::Face) -> Vec3 {
    NORMALS[face.as_usize()]
}

fn to_position(coords: Coords3D) -> Vec3 {
    Vec3::new(coords.x as f32, coords.y as f32, coords.z as f32)
}

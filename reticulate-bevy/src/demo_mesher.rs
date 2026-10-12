use bevy::prelude::*;

use crate::{chunk_mesher::ChunkMesher, demo_scene::DEMOS};

#[derive(Resource)]
pub struct DemoMesher {
    chunk_mesher: ChunkMesher,
    current_demo: usize,
}

impl DemoMesher {
    pub fn new() -> DemoMesher {
        DemoMesher {
            chunk_mesher: ChunkMesher::new(),
            current_demo: 0,
        }
    }

    pub fn next_demo(&mut self) -> Mesh {
        self.current_demo = (self.current_demo + 1) % DEMOS.len();

        let demo = &DEMOS[self.current_demo];
        let tree = demo.make_tree();

        println!("DemoMesher meshing: {}", demo.id());
        self.chunk_mesher.mesh(&tree)
    }
}

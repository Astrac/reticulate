use reticulate::{Coords3D, Fixed8Octree, MaterialId};

pub struct ChunkDemo {
    id: String,
    make_cell: Box<dyn FnMut(Coords3D) -> MaterialId>,
}

impl ChunkDemo {
    pub fn new(
        id: impl Into<String>,
        make_cell: impl FnMut(Coords3D) -> MaterialId + 'static,
    ) -> ChunkDemo {
        ChunkDemo {
            id: id.into(),
            make_cell: Box::new(make_cell),
        }
    }

    pub fn make_tree(&mut self) -> Fixed8Octree<MaterialId> {
        let mut tree = Fixed8Octree::new();
        self.update_tree(&mut tree);
        tree
    }

    pub fn update_tree(&mut self, tree: &mut Fixed8Octree<MaterialId>) {
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let coords = Coords3D::from_xyz(x, y, z);
                    let data = (self.make_cell)(coords);
                    if data != 0 {
                        tree.update(coords, Some(data));
                    }
                }
            }
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

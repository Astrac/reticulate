use std::range::Range;

use reticulate::{ChunkCoord, Coords3D, Fixed8Octree, MaterialId};

pub struct ChunkDemo {
    id: String,
    make_cell: Box<dyn Fn(Coords3D) -> MaterialId + Send + Sync>,
    x_range: Range<ChunkCoord>,
    y_range: Range<ChunkCoord>,
    z_range: Range<ChunkCoord>,
}

impl ChunkDemo {
    pub fn new(
        id: impl Into<String>,
        make_cell: impl Fn(Coords3D) -> MaterialId + 'static + Send + Sync,
    ) -> ChunkDemo {
        ChunkDemo {
            id: id.into(),
            make_cell: Box::new(make_cell),
            x_range: Range::from(0..32),
            y_range: Range::from(0..32),
            z_range: Range::from(0..32),
        }
    }

    pub fn make_tree(&self) -> Fixed8Octree<MaterialId> {
        let mut tree = Fixed8Octree::new();
        self.update_tree(&mut tree);
        tree
    }

    pub fn update_tree(&self, tree: &mut Fixed8Octree<MaterialId>) {
        for x in self.x_range {
            for y in self.y_range {
                for z in self.z_range {
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

    pub fn set_range_x(&mut self, r: std::ops::Range<ChunkCoord>) {
        self.x_range = Range::from(r);
    }

    pub fn set_range_y(&mut self, r: std::ops::Range<ChunkCoord>) {
        self.y_range = Range::from(r);
    }

    pub fn set_range_z(&mut self, r: std::ops::Range<ChunkCoord>) {
        self.z_range = Range::from(r);
    }
}

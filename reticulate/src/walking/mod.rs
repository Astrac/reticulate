mod compact;
mod fixed8;

use crate::{Depth, morton_3d::Morton3D};

pub trait Walker<Data> {
    fn on_branch_start(&mut self, depth: Depth, morton: Morton3D, children_mask: u8);
    fn on_branch_end(&mut self, depth: Depth, morton: Morton3D);
    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data);
}

pub trait DenseWalker<Data> {
    fn on_branch_start(&mut self, depth: Depth, morton: Morton3D, children_mask: u8);
    fn on_branch_end(&mut self, depth: Depth, morton: Morton3D);
    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data);
    fn on_empty(&mut self, depth: Depth, morton: Morton3D);
}

pub trait Walk<Data> {
    fn walk(&self, walker: &mut impl Walker<Data>);
}

pub trait DenseWalk<Data> {
    fn walk_dense(&self, walker: &mut impl DenseWalker<Data>);
}

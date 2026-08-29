use crate::{Depth, Morton3D, Walker};

#[derive(Default)]
pub struct PrintWalker {
    indent: usize,
}

const EMPTY_STRING: &'static str = "";

impl<Data: std::fmt::Debug> Walker<Data> for PrintWalker {
    fn on_branch_start(&mut self, depth: Depth, morton: Morton3D, children_mask: u8) {
        let spaces: usize = self.indent * 2;
        self.indent += 1;
        println!(
            "{EMPTY_STRING:spaces$}BRANCH_START: {depth:?}, {:#06o}, {children_mask:#08b}",
            morton.as_u32()
        );
    }

    fn on_branch_end(&mut self, depth: Depth, morton: Morton3D) {
        self.indent -= 1;
        let spaces: usize = self.indent * 2;
        println!(
            "{EMPTY_STRING:spaces$}BRANCH_END: {depth:?}, {:#06o}",
            morton.as_u32()
        );
    }

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data) {
        let spaces: usize = self.indent * 2;
        println!(
            "{EMPTY_STRING:spaces$}LEAF: {depth:?}, {:#06o}, {data:?}",
            morton.as_u32()
        );
    }
}

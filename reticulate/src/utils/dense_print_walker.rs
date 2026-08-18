use crate::{DenseWalker, Morton3D};

#[derive(Default)]
pub struct DensePrintWalker {
    indent: usize,
}

const EMPTY_STRING: &'static str = "";

impl<Data: std::fmt::Debug> DenseWalker<Data> for DensePrintWalker {
    fn on_branch_start(&mut self, depth: u8, morton: Morton3D, children_mask: u8) {
        let spaces: usize = self.indent * 2;
        self.indent += 1;
        println!(
            "{EMPTY_STRING:spaces$}BRANCH_START: {depth}, {:#06o}, {children_mask:#08b}",
            morton.as_u32()
        );
    }

    fn on_branch_end(&mut self, depth: u8, morton: Morton3D) {
        self.indent -= 1;
        let spaces: usize = self.indent * 2;
        println!(
            "{EMPTY_STRING:spaces$}BRANCH_END: {depth}, {:#06o}",
            morton.as_u32()
        );
    }

    fn on_leaf(&mut self, depth: u8, morton: Morton3D, data: &Data) {
        let spaces: usize = self.indent * 2;
        println!(
            "{EMPTY_STRING:spaces$}LEAF: {depth}, {:#06o}, solid({data:?})",
            morton.as_u32()
        );
    }

    fn on_empty(&mut self, depth: u8, morton: Morton3D) {
        let spaces: usize = self.indent * 2;
        println!(
            "{EMPTY_STRING:spaces$}LEAF: {depth}, {:#06o}, void",
            morton.as_u32()
        );
    }
}

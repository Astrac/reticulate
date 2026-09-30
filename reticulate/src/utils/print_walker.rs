use crate::{Morton3D, Walker};

#[derive(Default)]
pub struct PrintWalker {
    indent: usize,
}

const EMPTY_STRING: &'static str = "";

impl<Data: std::fmt::Debug> Walker<Data> for PrintWalker {
    fn on_branch_start(&mut self, morton: Morton3D, children_mask: u8) {
        let spaces: usize = self.indent * 2;
        self.indent += 1;
        println!(
            "{EMPTY_STRING:spaces$}BRANCH_START: {:?}, {:#06o}, {children_mask:#08b}",
            morton.depth(),
            morton.as_u32()
        );
    }

    fn on_branch_end(&mut self, morton: Morton3D) {
        self.indent -= 1;
        let spaces: usize = self.indent * 2;
        println!(
            "{EMPTY_STRING:spaces$}BRANCH_END: {:?}, {:#06o}",
            morton.depth(),
            morton.as_u32()
        );
    }

    fn on_leaf(&mut self, morton: Morton3D, data: &Data) {
        let spaces: usize = self.indent * 2;
        println!(
            "{EMPTY_STRING:spaces$}LEAF: {:?}, {:#06o}, {data:?}",
            morton.depth(),
            morton.as_u32()
        );
    }
}

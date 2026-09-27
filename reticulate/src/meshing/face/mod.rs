pub mod strip_run;
pub mod strip_weaver;
pub mod strip_zipper;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StripTriangle {
    pub vertices: [(u8, u8); 3],
}

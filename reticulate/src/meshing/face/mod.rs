pub mod strip_run;
pub mod strip_weaver;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StripTriangle {
    pub vertices: [(u8, u8); 3],
}

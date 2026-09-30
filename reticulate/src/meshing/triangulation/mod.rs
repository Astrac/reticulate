pub mod discoverer;
pub mod edges;
pub mod run;
pub mod run_mask;
pub mod runs;
pub mod stepper;
pub mod weaver;
pub mod zipper;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Triangle2D {
    pub vertices: [(u8, u8); 3],
}

pub mod scanner;
pub mod run;
pub mod stepper;
pub mod zipper;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WeavedTriangle {
    pub vertices: [(u8, u8); 3],
}

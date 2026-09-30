//! Unit tests, laid out to mirror the module tree of the crate: the tests
//! for `src/meshing/triangulation/stepper.rs` live in
//! `src/tests/meshing/triangulation/stepper.rs`, and so on. Shared fixtures
//! and assertions live in [`crate::utils::testing`].

mod axis;
mod coords_3d;
mod depth;
mod face;
mod meshing;
mod morton_2d;
mod morton_3d;
mod octree;
mod utils;
mod walking;

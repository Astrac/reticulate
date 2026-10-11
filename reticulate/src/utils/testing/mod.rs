//! Shared helpers for the crate's unit tests.
//!
//! - [`assertions`]: custom assertions for aspects of the mesher logic and invariants
//! - [`logging`]: one-time logger setup that is safe to call from every test.
//! - [`fixtures`]: voxel grids plus the parity/claims storage derived from them.
//! - [`mesh`]: drives the surface walker over a fixture and validates the mesh.
//! - [`weaving`]: ASCII-art regions and a reference weave for the stepper.
//! - [`walkers`]: a recording `Walker`/`DenseWalker` for octree walk tests.

pub mod assertions;
pub mod fixtures;
pub mod lcg;
pub mod logging;
pub mod mesh;
pub mod walkers;
pub mod weaving;

pub use fixtures::{DIRT, EMPTY, Fixture, STONE, VoxelGrid, cells, claims_of, parity_of, storage};
pub use lcg::SeededRandom;
pub use logging::init_logging;
pub use walkers::{RecordingWalker, WalkEvent};
pub use weaving::{AsciiGrid, weave_rows};

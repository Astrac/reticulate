//! Shared helpers for the crate's unit tests.
//!
//! Only compiled under `cfg(test)`. The helpers fall into a few groups:
//!
//! - [`logging`]: one-time logger setup that is safe to call from every test.
//! - [`rng`]: a tiny deterministic random number generator.
//! - [`fixtures`]: voxel grids plus the parity/claims storage derived from them.
//! - [`geometry`]: 2D/3D coverage, winding and T-junction assertions.
//! - [`mesh`]: drives the surface walker over a fixture and validates the mesh.
//! - [`weaving`]: ASCII-art regions and a reference weave for the stepper.
//! - [`walkers`]: a recording `Walker`/`DenseWalker` for octree walk tests.

pub mod fixtures;
pub mod geometry;
pub mod logging;
pub mod mesh;
pub mod rng;
pub mod walkers;
pub mod weaving;

pub use fixtures::{DIRT, EMPTY, Fixture, STONE, VoxelGrid, cells, claims_of, parity_of, storage};
pub use geometry::{
    area2, assert_covers_exactly, assert_no_t_junctions, assert_no_t_junctions_3d, assert_scan,
};
pub use logging::init_logging;
pub use mesh::assert_mesh_matches;
pub use rng::Lcg;
pub use walkers::{RecordingWalker, WalkEvent};
pub use weaving::{AsciiGrid, weave_rows};

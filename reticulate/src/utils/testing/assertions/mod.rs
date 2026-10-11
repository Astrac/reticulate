use crate::meshing::Triangle;
use crate::utils::testing::VoxelGrid;
use crate::{PLANE_SIDE, meshing::triangulation::Triangle2D, utils::testing::fixtures::cells};

mod coverage;
mod mesh;
mod minimal_triangulation;
mod t_junctions;

pub use coverage::assert_covers_exactly;
pub use mesh::assert_mesh_matches_grid;
pub use minimal_triangulation::assert_minimal_triangulation;
pub use t_junctions::assert_no_t_junctions;
pub use t_junctions::assert_no_t_junctions_2d;

pub fn assert_canonical_weave(
    filled: &[u32; PLANE_SIDE],
    expected_cells: &[(usize, usize)],
    triangles: &[Triangle2D],
) {
    assert_claimed_cells(filled, expected_cells);
    assert_canonical_winding(triangles);
    assert_no_t_junctions_2d(triangles);
}

pub fn assert_canonical_mesh(
    grid: &VoxelGrid,
    triangles: &[Triangle],
    tolerate_redundant_buried_material_transitions: bool,
) {
    assert_mesh_matches_grid(grid, triangles);
    assert_no_t_junctions(triangles);
    assert_minimal_triangulation(
        triangles,
        grid,
        tolerate_redundant_buried_material_transitions,
    );
}

/// Checks a scan of one plane: the claimed cells are exactly `expected`,
/// the triangles cover them once, wind counter-clockwise, and leave no
/// T-junctions.
pub fn assert_claimed_cells(filled: &[u32; PLANE_SIDE], expected: &[(usize, usize)]) {
    assert_eq!(filled, &cells(expected), "claimed cells differ");
}

/// Checks a scan of one plane: the claimed cells are exactly `expected`,
/// the triangles cover them once, wind counter-clockwise, and leave no
/// T-junctions.
pub fn assert_canonical_winding(tris: &[Triangle2D]) {
    for t in tris {
        assert!(area2(t) != 0, "Degenerate triangle: {t:?}");
        assert!(area2(t) > 0, "Non-canonical winding: {t:?}");
    }
}

/// Twice the signed area of a 2D triangle; positive when counter-clockwise.
fn area2(t: &Triangle2D) -> i32 {
    let [(x0, y0), (x1, y1), (x2, y2)] = t.vertices.map(|(c, r)| (c as i32, r as i32));
    (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0)
}

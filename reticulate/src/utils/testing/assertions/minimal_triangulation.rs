use crate::{MaterialId, meshing::Triangle, utils::testing::VoxelGrid};

pub fn assert_minimal_triangulation(
    tris: &[Triangle],
    grid: &VoxelGrid,
    tolerate_interfaces: bool,
) {
    let at = |p: [i32; 3]| grid.at((p[0], p[1], p[2]));

    let verts = tris
        .iter()
        .flat_map(|t| t.vertices().map(|c| [c.x as i32, c.y as i32, c.z as i32]));
    assert!(unjustified_vertices(&at, verts, tolerate_interfaces).is_empty());
}

type Coords = [i32; 3];
/// Exposed face type of a unit cell: (face points towards +axis, material of the solid side).
type CellType = Option<(bool, MaterialId)>;

fn unjustified_vertices(
    at: &impl Fn(Coords) -> MaterialId,
    vertices: impl IntoIterator<Item = Coords>,
    tolerate_interfaces: bool,
) -> Vec<Coords> {
    let mut seen = std::collections::BTreeSet::new();
    vertices
        .into_iter()
        .filter(|&p| seen.insert(p))
        .filter(|&p| {
            !is_required_vertex(at, p)
                && !(tolerate_interfaces && touches_material_interface(at, p))
        })
        .collect()
}

/// Voxel at coordinate `c` along `axis`, at in-plane position (u, v): X -> (y, z), Y -> (z, x), Z -> (x, y).
fn voxel(axis: usize, c: i32, u: i32, v: i32) -> Coords {
    match axis {
        0 => [c, u, v],
        1 => [v, c, u],
        _ => [u, v, c],
    }
}

fn cell_type(at: &impl Fn(Coords) -> MaterialId, axis: usize, c: i32, u: i32, v: i32) -> CellType {
    let (below, here) = (at(voxel(axis, c - 1, u, v)), at(voxel(axis, c, u, v)));
    match (below != 0, here != 0) {
        (true, false) => Some((true, below)),
        (false, true) => Some((false, here)),
        _ => None,
    }
}

/// Is a point where the outline of some face type turns (or pinches) among the 4 cells around it?
fn is_corner(t: [CellType; 4]) -> bool {
    t.iter().flatten().any(|&ty| {
        let same = t.map(|x| x == Some(ty));
        let n = same.iter().filter(|&&s| s).count();
        n == 1 || n == 3 || (n == 2 && ((same[0] && same[3]) || (same[1] && same[2])))
    })
}

/// A point that must be a mesh vertex: an outline corner of an exposed face in any of the three axis planes
/// through it. It only depends on the 2x2x2 voxels around the point.
pub fn is_required_vertex(at: &impl Fn(Coords) -> MaterialId, p: Coords) -> bool {
    (0..3).any(|a| {
        let (c, u, v) = (p[a], p[(a + 1) % 3], p[(a + 2) % 3]);
        is_corner([
            cell_type(at, a, c, u - 1, v - 1),
            cell_type(at, a, c, u - 1, v),
            cell_type(at, a, c, u, v - 1),
            cell_type(at, a, c, u, v),
        ])
    })
}

/// Two face-adjacent solid voxels of different materials among the 8 voxels around `p`. The mesher's wall
/// bookkeeping cannot tell such a hidden interface from a visible boundary, so it may keep a vertex here.
pub fn touches_material_interface(at: &impl Fn(Coords) -> MaterialId, p: Coords) -> bool {
    for d in 0..8 {
        let a = [p[0] - (d & 1), p[1] - ((d >> 1) & 1), p[2] - ((d >> 2) & 1)];
        for axis in 0..3 {
            let mut b = a;
            b[axis] += 1;
            let (ma, mb) = (at(a), at(b));
            if ma != 0 && mb != 0 && ma != mb {
                return true;
            }
        }
    }
    false
}

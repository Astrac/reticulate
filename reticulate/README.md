# reticulate

`reticulate` turns voxel chunks stored in a **sparse voxel octree** (SVO) into
triangle meshes. It has two halves:

- **Octrees** that store a 32×32×32 chunk of voxels sparsely, plus a *walker*
  abstraction to traverse them in Morton order.
- A **mesher** that walks a chunk twice and emits a *greedy*, *watertight*
  (no T-junctions) triangle mesh, one flat region of coplanar faces at a time,
  using nothing but bit operations on small bitfields.

The crate has no rendering code: the output is a stream of `Triangle`s that a
caller can upload wherever it likes.

- [Quick start](#quick-start)
- [Conventions](#conventions)
- [Sparse voxel octrees](#sparse-voxel-octrees)
- [The mesher](#the-mesher)
- [Module map](#module-map)
- [Portability](#portability)
- [Testing](#testing)
- [Known limitations](#known-limitations)

## Quick start

```rust
use reticulate::{CompactOctree, Coords3D, Fixed8Octree, meshing::Triangulator};

// A 2x1x1 bar of stone (material 1) inside a chunk.
let mut tree = Fixed8Octree::new();
tree.update(Coords3D { x: 2, y: 3, z: 4 }, Some(1u16));
tree.update(Coords3D { x: 3, y: 3, z: 4 }, Some(1u16));

// Optional: a denser, read-only representation.
let compact = CompactOctree::from_fixed8(&tree);

let mut triangulator = Triangulator::new();
let mut triangles = Vec::new();
triangulator.triangulate(
    compact,
    |data| *data,                 // map the payload to a MaterialId (0 = empty)
    |triangle| triangles.push(triangle),
);

assert_eq!(triangles.len(), 12); // six merged quads
```

The toolchain must support `uN::lowest_one` / `isolate_lowest_one` in `const`
contexts. Rust 1.94 stable does not; a recent nightly does (developed and
tested with a 1.101 nightly). The crate uses edition 2024.

## Conventions

Everything else in this document relies on a handful of conventions.

### Chunks and depth

A chunk is a cube of `32³` voxels: `TREE_DEPTH = 5` and
`PLANE_SIDE = 1 << TREE_DEPTH`. Coordinates are `ChunkCoord = u8`.

A tree node has a `Depth`: the root is depth 0 and covers the whole chunk, each
level halves the side, and depth 5 (`Depth::MAX`) is a single voxel. A node at
depth `d` covers a cube of `32 >> d` voxels per side and a quadtree-projected
area of `1024 >> 2d` cells.

### Morton codes

`Morton3D { code, depth }` identifies a node by the path of octants from the
root, three bits per level. Within each triplet bit 0 is `x`, bit 1 is `y`
and bit 2 is `z`, so the code of a voxel interleaves its coordinates as
`…z₁y₁x₁ z₀y₀x₀`. Walks visit nodes depth-first in increasing Morton order,
which, crucially for the mesher, visits the voxels of any axis-aligned column in
increasing order.

`Morton2D` is the same idea for a plane (`u` in even bits, `v` in odd bits).
`Coords3D` / `Coords2D` are the plain coordinate forms; `Morton3D` converts
between them (`from_xyz_*`, `Coords3D::from_morton`, `projected_axis_*`).

### Axes, planes and faces

A *face plane* at `layer` along an axis sits between voxel slab `layer - 1` and
slab `layer`. The plane's 2D coordinates are `(u, v)` — `u` is the **row** and
`v` the **column**:

| Plane axis | row `u` | column `v` |
|:-----------|:--------|:-----------|
| X          | y       | z          |
| Y          | z       | x          |
| Z          | x       | y          |

`SurfaceLayer { axis, layer }` names one such plane. `Face` is an `Axis` plus a
direction (`Positive` or `Negative`), i.e. which way the face's normal points.

## Sparse voxel octrees

Both octrees are generic over a payload `Data`. The mesher treats a voxel as
*solid* when it has a payload and *empty* otherwise.

### `Fixed8Octree`

The editable representation. Nodes live in one `Vec`; a `Branch` holds the index
of its eight children, which are stored contiguously, and a `Leaf` holds an
`Option<Data>` (`None` is empty space).

- `Fixed8Octree::new()` is a single empty root leaf.
- `update(coords, Some(data))` descends to the voxel, splitting any leaf on the
  way into eight children carrying the leaf's old value. `update(coords, None)`
  clears a voxel.
- `get(coords)` returns the payload at a voxel.
- Nothing is ever merged back: it trades memory for simple, cheap edits.

### `CompactOctree`

A read-optimized representation built with `CompactOctree::from_fixed8`. Nodes
are stored depth-first in a flat `Vec<CompactOctreeNode>`:

- `Branch(mask)` — bit `i` of `mask` says octant `i` is present. Only present
  children follow, in octant order.
- `Leaf(data)` — a solid node.

Empty subtrees are pruned, and a branch whose eight children are all leaves with
equal data collapses into one leaf, so a solid 4×4×4 block becomes a single
depth-3 leaf. The mesher benefits twice: fewer nodes to visit, and larger nodes
mean fewer, bigger bit operations.

### Walking

Trees are consumed through *walkers*, callback objects driven in Morton order:

```text
Walker<Data>:       on_branch_start(morton, children_mask)
                    on_branch_end(morton)
                    on_leaf(morton, &data)
DenseWalker<Data>:  the same, plus on_empty(morton)
```

`Walk::walk` reports only solid leaves. `DenseWalk::walk_dense` also reports
every **empty** node, so the leaves and empties together tile the whole chunk
with no gaps: a dense walk of any valid tree covers exactly `32³` voxels.
The mesher needs this because a face exists exactly where solid meets empty.

Both octrees implement both traits (`walking/`). `PrintWalker` and
`DensePrintWalker` (in `utils`) dump a walk to stdout for debugging.

## The mesher

```rust
Triangulator::triangulate(tree, to_material, on_triangle)
```

takes any `DenseWalk`, a closure mapping the payload to a `MaterialId` (`u16`,
`0` meaning empty), and a callback receiving each `Triangle`:

```text
Triangle { face: Face, material: MaterialId, vertices: [Coords3D; 3] }
```

Vertices are in chunk-local voxel corner coordinates (`0..=32`). The stored
winding is the same for every face; `outward_vertices()` returns them ordered so
the normal points away from the solid.

### The idea

Instead of looking at each voxel's six neighbours, the mesher works on whole
**planes** with 32-bit rows:

1. Record every place where the material changes along each axis (*parity*).
2. For each solid/empty boundary, flood a whole connected, coplanar region of
   faces with bit operations, and triangulate it as one piece (*greedy meshing*).
3. Remember which face cells were emitted (*claims*) so nothing is emitted
   twice.

Large octree nodes are handled as a unit, so uniform space costs almost nothing.

### Bitfields (`meshing::bitfields`)

All scratch state lives in one `BitfieldsStorage` (reused between chunks) and is
viewed through `Bitfields::from_storage` as three independent structures, each
with one instance per axis:

- **`Material`** — a `32×32` plane of 16-bit material ids per axis. Cells are
  stored in **Morton2D order**, so every quadtree node's area is one contiguous
  range. `fill` writes a node's area in one go, and `iterate_materials` walks an
  area as a sequence of uniform, power-of-four-aligned runs. A plane cell holds
  the material of the most recently visited voxel of that column.
- **`Parity`** — for each axis, one `u32` per `(u, v)` position whose bit `k` is
  set when the material at coordinate `k` along the axis differs from the one at
  `k - 1`. It is the sparse "a face starts here" map, and it also stores the
  *walls* between coplanar faces of different materials.
- **`Claims`** — for each axis and layer, 32 `u32` rows (row `u`, bit `v`) of
  face cells that have already been emitted.

### Pass 1: gathering (`GatheringWalker`)

The first dense walk fills in `Parity`. For every node, on each axis, it reads
the materials already present in the node's projected area (what lies below, by
Morton order), sets a transition bit at the node's first layer wherever the
material differs, and then fills the area with the node's own material. Because
Morton order visits each column in increasing order, "already present" always
means "directly below".

The pass has to finish before any surface is extracted: a region's boundary
depends on transitions gathered from nodes that are visited later.

### Pass 2: scanning (`SurfaceWalker` and `SurfaceScanner`)

`Material` is cleared and the tree is walked again. For each node (in Morton
order) and each axis, `SurfaceScanner::scan` compares the node's material with
the material below it in the projected area. A face is wanted only between a
solid and empty space: equal materials and solid/solid pairs are skipped (the
shared face of stone against dirt is occluded, though the material change still
splits coplanar faces around it). A solid node therefore scans its `Negative`
faces, and an empty node scans the `Positive` faces of the solids below it. The
emitted face takes the solid's material.

For every not-yet-claimed starting cell the scanner:

1. **Fills** a region (`triangulation::filler::Filler`). Starting from the seed
   cell, it expands each row to the run between walls, then grows up and down to
   neighbouring rows until nothing changes. Walls come from `Edges`: the parity
   of the perpendicular axis on this slab *and* the previous one (a face lies
   between the two slabs). Parity guards between rows stop growth across
   material transitions, and claimed cells are avoided. The result is one
   `u32` row mask per row.
2. **Weaves** it into triangles (`triangulation::weaver::Weaver`). The weaver
   sweeps the region row by row. `Runs`/`Run` track the horizontal stretches of
   cells in the current row, `Stepper` advances from one row to the next, and
   `Zipper` joins the vertex chain along the bottom of a strip to the one along
   its top with a triangle strip. `Weaver::partition_corners` adds vertices
   wherever a neighbouring region has a corner on this region's edge, so the
   mesh has no T-junctions.
3. **Claims** the region's cells and emits the triangles, lifted to 3D by
   `Triangle::from_strip_triangle`.

### Bit-level helpers (`utils::bitmasks`)

Everything above is built from a few branch-free helpers: Morton
interleave/deinterleave, `make_ones_mask`, `run_starts` / `run_ends` /
`segment_boundaries`, and `boundary_changes` / `boundary_points`, which find the
columns where a row's boundary differs from the row below it.

## Module map

| Path                                  | Contents                                                       |
|:--------------------------------------|:---------------------------------------------------------------|
| `axis`, `face`, `depth`               | `Axis`, `Face`, `Depth`                                        |
| `coords_2d`, `coords_3d`              | plain coordinates and plane projections                        |
| `morton_2d`, `morton_3d`              | Morton codes, including the BMI2 fast path                     |
| `octree/`                             | `Fixed8Octree`, `CompactOctree`, `TREE_DEPTH`                  |
| `walking/`                            | `Walker`/`DenseWalker` traits and their octree implementations |
| `meshing/triangulator`                | `Triangulator`, the two-pass driver                            |
| `meshing/gathering_walker`            | pass 1: transitions into `Parity`                              |
| `meshing/surface_walker`, `surface_scanner` | pass 2: find, fill, weave and claim faces                |
| `meshing/bitfields/`                  | `Material`, `Parity`, `Claims`, `BitfieldsStorage`             |
| `meshing/triangulation/`              | `filler`, `edges`, `runs`/`run`/`run_mask`, `stepper`, `zipper`, `weaver` |
| `meshing/triangle`, `surface_layer`   | output `Triangle`, `SurfaceLayer`                              |
| `utils/bitmasks`                      | branch-free bit helpers                                        |
| `utils/*print_walker`, `debug_assertions` | debugging aids                                             |
| `utils/testing`                       | shared test helpers (`cfg(test)` only)                         |

## Portability

The crate targets x86_64 and aarch64. The only architecture-specific code is in
`morton_2d`, where Morton encoding and decoding use BMI2 `pdep`/`pext` when the
CPU has them — detected at run time on x86_64, or at compile time with
`-C target-feature=+bmi2` — and otherwise fall back to a portable bit-twiddling
implementation. aarch64 and every other target always use the portable path
(the BMI2 code is `cfg(target_arch = "x86_64")`), and on x86_64 the unit tests
check that both paths give identical results. The test suite was run on x86_64;
the aarch64 path is the same portable code that is exercised there.

## Testing

```sh
cargo +nightly test -p reticulate
```

The unit tests in `src/tests/` mirror the source tree: the tests for
`src/meshing/triangulation/stepper.rs` are in
`src/tests/meshing/triangulation/stepper.rs`. Shared fixtures and assertions are
in `src/utils/testing/`:

- `fixtures` — `VoxelGrid` and `Fixture`: a dense chunk plus its parity and
  claims, built by brute force, so the mesher's own bit-twiddling is checked
  against a simple reference.
- `geometry` / `mesh` — coverage, winding and T-junction checks, and
  `assert_mesh_matches`, which verifies a mesh against the voxel grid alone
  (every exposed face covered exactly once, with the right material and
  orientation).
- `walkers`, `weaving`, `rng`, `logging` — a recording walker, ASCII-art regions,
  a seedable RNG and idempotent logger setup.

Tests that document known gaps are `#[ignore]`d with a reason;
`cargo test -- --ignored` runs them.

## Known limitations

- **Chunk boundary.** Faces on the far side of a chunk (plane 32) and regions
  that touch column 31 are not supported yet; a row of 32 bits cannot hold a wall
  at column 32. Neighbouring chunks are not consulted.
- **`Triangulator` reuse.** `Triangulator::triangulate` does not reset the claims,
  parity and material state left by a previous call, so a `Triangulator` currently
  gives correct results for one chunk only. Create a new one per chunk.
- **`Face::is_positive`** returns `true` for `Face::Negative`; it really means
  "the stored winding already faces outward". Use `Triangle::outward_vertices`
  rather than branching on it.
- **`Fixed8Octree`** children masks are only maintained at the lowest level and
  are ignored by walks.

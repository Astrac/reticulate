# Reticulate

This project is a [Sparse Voxel Octree](https://en.wikipedia.org/wiki/Sparse_voxel_octree)
mesher with an optional integration into the bevy game engine.

- [Features](#features)
- [Benchmarks](#benchmarks)
- [Demo](#demo)
- [Components](#components)
- [License](./LICENSE.md)
- [Contributing](./CONTRIBUTING.md.md)

## Features

These are the high-level features currently implemented:

- Sparse Voxel Octree data structure variants for reading and editing
- Mesher component supporting materials up to 16bits
- Criterion benchmarks for chunk meshing
- Small bevy demo

Guaranteed features of the generate mesh:

- The mesh is built from greedy triangles
- The number of triangles in a single-material mesh is guaranteed minimal
- The number of triangles in a multi-material mesh is [very close to minimal](#minimality)
- The mesh is watertight with no t-junctions of any kind

On the roadmap:

- The machinery to manage multiple chunks. Moreover, the last layer of the chunk
  along each axis is not currently rendereda and that will be fixed by seeding the
  next chunk.
- Raycasting to determine voxel coordinates from screen coordinates
- Shaders with texture atlas support
- More demos

## Benchmarks

The mesher is tested with several scenario for both the fixed-8 and the compact
variant of the tree. To run the benchmarks run this command:

```sh
cargo bench -p reticulate-bench --bench chunk
```

If you want to have an extra report of metrics enable the `metrics-recording`
feature from the command line:

```sh
cargo bench -p reticulate-bench --bench chunk --features "metrics_recording"
```

Enabling metrics has a noticeable impact on performance. Please note that the
numbers provided in the report won't probably match with the ones provided by
criterion as there is no statistical normalization behind them.

This is a summary of the Criterion `time` results from provided scenarios
running on a MacBook Pro M4:

```
chunk_checkerboard_single_material_compact:    [506.78 µs 507.28 µs 507.74 µs]
chunk_checkerboard_single_material_fixed_8:    [681.79 µs 796.82 µs 904.50 µs]
chunk_ortho_labyrinth_single_material_compact: [230.68 µs 231.02 µs 231.39 µs]
chunk_ortho_labyrinth_single_material_fixed_8: [274.62 µs 301.50 µs 326.51 µs]
chunk_sphere_single_material_compact:          [57.010 µs 57.092 µs 57.173 µs]
chunk_sphere_single_material_fixed_8:          [99.453 µs 101.97 µs 104.30 µs]
chunk_terrain_layered_material_compact:        [96.105 µs 96.265 µs 96.423 µs]
chunk_terrain_layered_material_fixed_8:        [144.33 µs 152.06 µs 159.38 µs]
chunk_terrain_single_material_compact:         [28.415 µs 28.458 µs 28.502 µs]
chunk_terrain_single_material_fixed_8:         [103.32 µs 106.10 µs 108.70 µs]
```

This is the same summary running against a Intel Core i7-13700K on Windows 11:

```
chunk_checkerboard_single_material_compact:    [550.46 µs 551.53 µs 552.56 µs]
chunk_checkerboard_single_material_fixed_8:    [521.80 µs 522.59 µs 523.49 µs]
chunk_ortho_labyrinth_single_material_compact: [258.90 µs 259.47 µs 260.02 µs]
chunk_ortho_labyrinth_single_material_fixed_8: [254.87 µs 255.34 µs 255.84 µs]
chunk_sphere_single_material_compact:          [105.18 µs 105.43 µs 105.68 µs]
chunk_sphere_single_material_fixed_8:          [171.26 µs 171.58 µs 171.91 µs]
chunk_terrain_layered_material_compact:        [215.98 µs 216.52 µs 217.04 µs]
chunk_terrain_layered_material_fixed_8:        [268.52 µs 269.06 µs 269.64 µs]
chunk_terrain_single_material_compact:         [32.031 µs 32.113 µs 32.196 µs]
chunk_terrain_single_material_fixed_8:         [135.94 µs 136.44 µs 136.97 µs]
```

## Demo

A demo built with bevy is available at this address:

## Components

This is a summary of the components provided by the `reticulate` library:

- [The Mesher](#the-mesher)
   * [Mesher - the idea](#mesher---the-idea)
   * [Mesher - bitfields (`meshing::bitfields`)](#mesher---bitfields-meshingbitfields)
   * [Mesher - pass 1: gathering (`GatheringWalker`)](#mesher---pass-1-gathering-gatheringwalker)
   * [Mesher - pass 2: scanning (`SurfaceWalker` and `SurfaceScanner`)](#mesher---pass-2-scanning-surfacewalker-and-surfacescanner)
- [Linear octree](#linear-octree)
   * [`Fixed8Octree`](#fixed8octree)
   * [`CompactOctree`](#compactoctree)
- [Walking](#walking)
- [Morton codes](#morton-codes)
- [Axes, planes and faces](#axes-planes-and-faces)

### The Mesher

```rust
Mesher::make_triangles(tree, to_material, on_triangle)
```

takes any `DenseWalk`, a closure mapping the payload to a `MaterialId` (`u16`,
`0` meaning empty), and a callback receiving each `Triangle`:

```text
Triangle { face: Face, material: MaterialId, vertices: [Coords3D; 3] }
```

Vertices are in chunk-local voxel corner coordinates (`0..=32`). The stored
winding is the same for every face; `outward_vertices()` returns them ordered so
the normal points away from the solid.

#### Mesher - the idea

Instead of looking at each voxel's six neighbours, the mesher works on whole
**planes** with 32-bit rows:

1. Record every place where the material changes along each axis (*parity*).
2. For each solid/empty boundary, flood a whole connected, coplanar region of
   faces with bit operations, and triangulate it as one piece (*greedy meshing*).
3. Remember which face cells were emitted (*claims*) so nothing is emitted
   twice.

Large octree nodes are handled as a unit, so uniform space costs almost nothing.

#### Mesher - bitfields (`meshing::bitfields`)

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

#### Mesher - pass 1: gathering (`GatheringWalker`)

The first dense walk fills in `Parity`. For every node, on each axis, it reads
the materials already present in the node's projected area (what lies below, by
Morton order), sets a transition bit at the node's first layer wherever the
material differs, and then fills the area with the node's own material. Because
Morton order visits each column in increasing order, "already present" always
means "directly below".

The pass has to finish before any surface is extracted: a region's boundary
depends on transitions gathered from nodes that are visited later.

#### Mesher - pass 2: scanning (`SurfaceWalker` and `SurfaceScanner`)

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

#### Minimality

Single-material meshes will have the minimum number of triangles required by the
topology. Multi-material meshes could have some extra vertices due to the fact
that the parity approach can only tell about tranitions but not occupancy of a
particular cell.

As a result, if a transition is buried under a uniform wall the wall will get an
extra vertex that could actually have been simplified. This should happen in a very
small number of non-degenerate cases.

### Linear octree

This crate implements a **Linear octree** data-structure represented in memory as a flat
vector on the heap without pointers. Each octree represents a 32x32x32 chunks of voxels.
The octree comes in two possible variants:

- Compact: where only the minimal necessary data is stored, optimized for memory/meshing
- Fixed 8-slot: where all the branches of the octree imply that all the 8 child slots
  have a (potentially empty) data-point in the vector; this is best for editing data

Both octrees are generic over a payload `Data`. The mesher treats a voxel as
*solid* when it has a payload and *empty* otherwise. When performing triangulation,
a closure can be provided to translate `Data` into a `MaterialId`, i.e. a `u16` value.

#### `Fixed8Octree`

The editable representation. Nodes live in one `Vec`; a `Branch` holds the index
of its eight children, which are stored contiguously, and a `Leaf` holds an
`Option<Data>` (`None` is empty space).

- `let tree = Fixed8Octree::new()` is a single empty root leaf.
- `tree.update(coords, Some(data))` descends to the voxel, splitting any leaf on the
  way into eight children carrying the leaf's old value. `update(coords, None)`
  clears a voxel.
- `tree.get(coords)` returns the payload at a voxel.

Nothing is ever merged back, this implementation trades memory for simple, cheap edits
and pre-allocated slots guaranteed for writes to the tree.

#### `CompactOctree`

A read-optimized representation built with `CompactOctree::from_fixed8`. Nodes
are stored depth-first in a flat `Vec<CompactOctreeNode>`:

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

### Morton codes

Several data types that enable this work such as **Morton** 2d/3d codes data-structures
and functions to operate on them are provided with this implementation.

Coordinates are represented in 3D using a `Coords3D` struct with `x`, `y` and `z`
components and in 2D using a `Coords2D` struct with `u` and `v` components.

These coordinates are coupled to implementations of [Morton](https://en.wikipedia.org/wiki/Z-order_curve)
coding, one for 3D and one for 2D. The `Morton3D` code is represented as
a `zyx...` bit interleave, the `Morton2D` code is represented as a `vu...`
bit interleave; the both are LSB-aligned within rust `u32` values.

Morton codes are coupled to `Depth` values to univocally identify which area/volume
of the quadtree/octree they represent. At `Depth = 0` there is only one valid
`Morton` code, i.e. `0`, and it represents the full area/volume. Each `Depth`
increase up to `5` represent a subdivision of the 2D area by `4` or 3D volume
by `8`. At `Depth = 5` the `Morton` index will identify a single 3D voxel or
2D texel.

### Axes, planes and faces

A *face plane* at `layer` along an axis sits between voxel slab `layer - 1` and
slab `layer`. The plane `Coords2D` values are ordered so that `u` is the **row** and
`v` the **column** in that particular context. When looking at planes perpendicular
to a particular axis, these are the specific mappings of `u` and `v` to the 3D context.

| Plane axis | Row `u` | Column `v` |
|:-----------|:--------|:-----------|
| X          | Y       | Z          |
| Y          | Z       | X          |
| Z          | X       | Y          |

`SurfaceLayer { axis, layer }` names one such plane. `Face` is an `Axis` plus a
direction (`Positive` or `Negative`), i.e. which way the face's normal points.

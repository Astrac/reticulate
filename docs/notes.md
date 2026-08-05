 Decouple scale from depth - the mesh needs to be built at a scale that is dependent from the content, not from the structure
   - Scale goes deeper when traversing branches
   - Scale goes shallower when all outer faces of a completed branch are uniform
      * Maybe better
          - Minimum scale of an octant is influenced by its preceding octants
          - Real scale of an octant is influenced by its preceding octants
   - Scale governs the resolution of the cube we carve out
   - A face-mask at scale X is interleaved with itself when going one level down
      * ex: 100110 -> 110000111100
      * Scale of a face_mask: biggest possible interleaving?
      * DATA STRUCTURE: (level, face_id, bitmask)
      * Going up is only possible if all xx pairs of bits are uniform (NOT (A XOR B))
        - WILD: Could it be possible to cull across boundaries if this is true?
   - A node needs to provide 2 ^ (scale - depth) bitmask per outer face at its depth
     * Max 2 ^ 6 = 32bits
     * Each node has max 6 * 32bits = 24 bytes
     * When a leaf it is all 1
     * When a branch ...
     * At any time I have X node states in memory:
       - If compressed X = depth (Max stack bytes = 24 * 6 = 144bytes)
       - If uncompressed X = depth + interleaved-branches-penalty
     * Optimizing scale - depth seems unnecessary, we could always draw at max scale on that mask?
       - How not to lose the merged blocks this way?
   - When xoring masks a branch is xored with the bits of its neighbours at the correct scale!
   - The ideal algorithm:
     * Has stack limited to open branches
     * Completes branches strictly one by one (closes one before opening the next)

 When opening a branch:
   - Push current state to the stack
   - Set face_mask = project_from_neighbours

  When visiting a leaf:
   - Scale = derive_scale(face_mask) - Explicitly tracking might be better
   - Set (level, face_id, bitmask) according to octant occlusion
      * bitmask = bitmask ^ occluded_faces(leaf_octant)

 When closing a branch:
   - Get current outer-faces face_mask
   - Pop previous state from the stack
   - Set face_mask = face_mask ^ project_into_neighbours
 Where
   project_from_neighbours:
     - Copy inner faces of current branch into the outer faces of a new mask
   project_into_neighbours:
     - Copy outer faces of current branch into the inner faces of a new mask

 64 Vertices in an octant
   Each voxel has 8 vertices
   Each octant has 8 voxels

 DECISION:
  - Use BitArr! to allocate a bit matrix of everything: BitArr![for 12 * 64 * 64, in u64, Lsb0]
  - Could it be worth to store resolution in the bit-string itself?
  - Map projections into the matrix
  - Do we really need full resolution tracking for inner?

 NOTICE:
  - Only one set of bitmasks is being worked on at any moment

 IMPORTANT!
  - Define Upsampling = max(Depth, Resolution - Depth)
  - Resolution should be a property of an individual face
      * Could we get away with tracking only resolution?
  - Upsampling is a property of the current iteration
    i.e. relationship between boundary faces and current depth
  - Depth is how many branches we traversed from the root to the point
    we are evaluating. Tracked by size of the stack.

  When opening the octree:
      - Set resolution = 0 for all boundary faces

  When opening a branch:
      - Project boundaries
      - Push stack

  When closing a branch:
      - Apply children masks
      - Pop stack
      - Project boundaries inverse

  When visiting a leaf:
    - Determine upsampling
    - If upsampling == 0 apply leaf to mask (<= 3?)
    - If upsampling > 0 simulate a branch of solid leaves

 It is not true that we can just store u64s and have completely scale-invariant boundaries. The
 root node cannot keep face masks at full resolution as each mask is actually 64x64 bits (4096),
 bringing the total size of the stack to a possibly concerning 4096 * 12 = about 6KB. Moreover,
 operations at that scale (i.e. 4096 per mask) would require many instructions for any face
 combination, rendering the whole thing much slower.

 We do need to:

    - Track depth
    - Track upscaling (how many levels deeper than depth is there boundary detail?)
    - Filter and upscale the masks when we project from parent to child
    - Downscale if possible when progressing from child to parent
    - What do we do if a parent needs to track a boundary at an upscaling deeper than 8x8 (the
      capability of the current u64 bitmask)?

 Reasoning about granularity...
    - We can keep a 8x8 mask for each face of the boundary (i.e. upscaling 3 wrt the current level)
    - What happens when we need to visit a new child and we are already at upscaling 3?
    - Can we make boundaries a tree in some way when we need to go deeper than 3?
      - E.g. a boundary node could keep a mask at maximum upscaling and mark the beginning of a new
             mask
      - Is it possible to cull such a tree with a mask or another tree efficiently?
      - How do we use such data structure in meshing?
    - Is culling right-associative??

 Giving up resolution-invariance (maybe recover part of it later as optimization)
    - The stack keeps a bitmasks together with theire resolution
    - When visiting a branch we check each relevant face's resolution
    - The minimum resolution of each octant is decided by that
    - Bitmask + resolution + morton on face

      * 9 masks 64x64
        - 3 axis * 2 sides outer
        - 3 axis inner
        - 00001 => Outer Negative x
        - 00010 => Outer Negative y
        - 00100 => Outer Negative z
        - 01001 => Outer Positive x
        - 01010 => Outer Positive y
        - 01100 => Outer Positive z
        - 10001 => Inner Negative x
        - 10010 => Inner Negative y
        - 10100 => Inner Negative z
      * Even are outer zyxpuv0 => 0pzyx
      * Odds are inner zyx0uv1 => 10zyx

  Important:
    - When visiting a leaf check needed upscaling on touching faces
    - When joining a boundary upscaling changes must include the joined area
      * Maybe I should prove this?

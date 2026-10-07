use crate::{
    Axis, Coords2D, Morton2D, Morton3D,
    meshing::bitfields::{MaterialId, PLANE_AREA},
};

// TODO - See if we can make planes private
#[repr(transparent)]
pub struct Material<'a> {
    pub planes: [&'a mut [MaterialId; PLANE_AREA]; 3],
}

impl<'a> Material<'a> {
    #[inline(always)]
    pub fn iterate_materials(
        &self,
        axis: Axis,
        morton: Morton2D,
        mut on_material: impl FnMut(MaterialId, Coords2D, usize),
    ) {
        let plane: &[MaterialId; PLANE_AREA] = &*self.planes[axis.as_usize()];
        let mut offset = morton.as_usize();
        let end = offset + morton.depth().as_area();

        // Cells in offset..run_end are known to equal run_material.
        let mut run_end = offset;
        let mut run_material: MaterialId = 0;

        while offset < end {
            let max_len = quadtree_alignment(offset).min(end - offset);

            if offset >= run_end {
                run_material = plane[offset];
                run_end = offset + run_length(plane, offset, run_material, max_len);
            }

            let len = quadtree_truncate((run_end - offset).min(max_len));
            on_material(
                run_material,
                Morton2D::from_code_max_depth(offset as u32).as_coords(),
                len,
            );
            offset += len;
        }
    }

    #[inline]
    pub fn fill(&mut self, axis: Axis, morton: Morton3D, material: MaterialId) {
        let (morton_2d, _) = morton.projected_axis_morton(axis);
        let area = PLANE_AREA >> (morton.depth.as_usize() << 1);
        let start = morton_2d.as_usize();
        debug_assert!(start % area == 0 && start + area <= PLANE_AREA);

        let plane = &mut *self.planes[axis.as_usize()];
        if area == 1 {
            plane[start] = material; // single store, no RMW
        } else {
            plane[start..start + area].fill(material); // vectorizes to wide stores
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        for p in &mut self.planes {
            p.fill(0);
        }
    }
}

/// Length of the uniform run starting at `start`, capped at `max_len`.
/// `max_len` is a power of 4 and `start` is a multiple of it.
#[inline(always)]
fn run_length(
    plane: &[MaterialId; PLANE_AREA],
    start: usize,
    material: MaterialId,
    max_len: usize,
) -> usize {
    debug_assert!(max_len.is_power_of_two() && start % max_len == 0);

    if max_len < 4 {
        return max_len; // a single cell, nothing to scan
    }

    let splat = splat64(material);
    for (i, quad) in plane[start..start + max_len].chunks_exact(4).enumerate() {
        let word = bytemuck::cast::<[u16; 4], u64>(quad.try_into().unwrap());
        let diff = word ^ splat;
        if diff != 0 {
            // lowest differing 16-bit lane (little-endian)
            return i * 4 + (diff.trailing_zeros() / 16) as usize;
        }
    }
    max_len
}

/// Largest power of 4 dividing `offset`; PLANE_AREA for offset 0.
#[inline(always)]
const fn quadtree_alignment(offset: usize) -> usize {
    1 << ((offset | PLANE_AREA).trailing_zeros() & !1)
}

/// Largest power of 4 that is <= `len` (`len` must be > 0).
#[inline(always)]
const fn quadtree_truncate(len: usize) -> usize {
    debug_assert!(len > 0);
    1 << ((usize::BITS - 1 - len.leading_zeros()) & !1)
}

#[inline(always)]
const fn splat64(material: u16) -> u64 {
    let material_u64 = material as u64;
    material_u64 | (material_u64 << 16) | (material_u64 << 32) | (material_u64 << 48)
}

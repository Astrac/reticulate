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

        let start = morton.as_usize() & (PLANE_AREA - 1);
        let area = morton.depth().as_area();

        if area == 1 {
            on_material(plane[start], morton.as_coords(), 1);
            return;
        }

        let mut local = 0;
        let mut run_end = 0;
        let mut run_material: MaterialId = 0;

        while local < area {
            // Largest power of 4 dividing `local`; area if local == 0.
            let max_len = 1usize << ((local | area).trailing_zeros() & !1);

            if local >= run_end {
                run_material = plane[start + local];
                run_end = local + run_length(plane, start + local, run_material, max_len);
            }

            let len = quadtree_truncate((run_end - local).min(max_len));
            on_material(
                run_material,
                Morton2D::from_code_max_depth((start + local) as u32).as_coords(),
                len,
            );
            local += len;
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

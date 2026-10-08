use crate::{
    Axis, Coords2D, Morton2D,
    meshing::bitfields::{MaterialId, PLANE_AREA},
};

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
    pub fn fill_projected(&mut self, axis: Axis, morton: Morton2D, material: MaterialId) {
        let area = PLANE_AREA >> (morton.depth().as_usize() << 1);
        let start = morton.as_usize();
        debug_assert!(start % area == 0 && start + area <= PLANE_AREA);

        let plane = &mut *self.planes[axis.as_usize()];
        if area == 1 {
            plane[start] = material;
        } else {
            plane[start..start + area].fill(material);
        }
    }

    // TODO - Remove this
    #[cfg(test)]
    #[inline(always)]
    pub fn fill(&mut self, axis: Axis, morton: crate::Morton3D, material: MaterialId) {
        self.fill_projected(axis, morton.projected_axis_morton(axis).0, material);
    }

    #[inline(always)]
    pub fn replace_projected(
        &mut self,
        axis: Axis,
        node: Morton2D,
        material: MaterialId,
        mut on_material: impl FnMut(MaterialId, Coords2D, usize),
    ) {
        if node.depth().as_area() == 1 {
            let start = node.as_usize() & (PLANE_AREA - 1);
            let old = core::mem::replace(&mut self.planes[axis.as_usize()][start], material);
            on_material(old, node.as_coords(), 1);
            return;
        }
        self.iterate_materials(axis, node, &mut on_material);
        self.fill_projected(axis, node, material);
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
        return max_len;
    }

    let quads: &[[MaterialId; 4]; PLANE_AREA / 4] = bytemuck::cast_ref(plane);
    let want = [material; 4];
    let first = start >> 2;
    for (i, quad) in quads[first..first + (max_len >> 2)].iter().enumerate() {
        if *quad != want {
            return i * 4 + quad.iter().position(|&c| c != material).unwrap();
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

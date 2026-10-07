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
    /// Reads the materials present in an area
    #[inline(always)]
    pub fn iterate_materials(
        &self,
        axis: Axis,
        morton: Morton2D,
        mut on_material: impl FnMut(MaterialId, Coords2D, usize),
    ) {
        let materials = self.material_plane(axis);
        let index = morton.as_usize();
        let plane_size = morton.depth().as_area();
        let final_index = index + plane_size;

        let mut current_offset = index;
        let mut current_grid = quadtree_alignment(current_offset);

        while current_offset < final_index {
            let max_length = current_grid.min(final_index - current_offset);
            let (material, scanned_length) = next_material(materials, current_offset, max_length);

            on_material(
                material,
                Morton2D::from_code_max_depth(current_offset as u32).as_coords(),
                scanned_length,
            );

            current_offset += scanned_length;
            current_grid = quadtree_alignment(current_offset);
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

    #[inline]
    fn material_plane(&self, axis: Axis) -> &[MaterialId; PLANE_AREA] {
        self.planes[axis.as_usize()]
    }
}

#[inline(always)]
fn next_material(materials: &[u16], start_offset: usize, max_length: usize) -> (u16, usize) {
    debug_assert!(max_length > 0);
    debug_assert!(materials.len() > start_offset);

    let mut current_offset = start_offset;
    let final_offset = start_offset + max_length;
    let material = materials[current_offset];

    while current_offset < final_offset {
        if material != materials[current_offset] {
            break;
        }

        current_offset += 1;
    }

    (material, quadtree_truncate(current_offset - start_offset))
}

/// Returns the biggest power of 4 that is smaller of the given value
#[inline(always)]
const fn quadtree_truncate(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        let msb_index = usize::BITS - 1 - value.leading_zeros();
        let even_index = msb_index & !1;
        1 << even_index
    }
}

/// Returns the biggest power of 4 that can divide the offset or PLANE_AREA if the offset is 0
#[inline(always)]
const fn quadtree_alignment(offset: usize) -> usize {
    if offset == 0 {
        PLANE_AREA
    } else {
        1 << (offset.trailing_zeros() & !1)
    }
}

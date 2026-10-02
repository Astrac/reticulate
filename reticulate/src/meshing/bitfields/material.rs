use core::panic;

use crate::{
    Axis, Coords2D, Morton2D, Morton3D,
    meshing::bitfields::{MATERIAL_BITS, MaterialId, PLANE_AREA, STORAGE_WORD, Slice},
};

#[repr(transparent)]
pub struct Material<'a> {
    pub cells: [&'a mut Slice; 3],
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

    /// Fills an area with the given material
    #[inline(always)]
    pub fn fill(&mut self, axis: Axis, morton: Morton3D, material: u16) {
        let (morton_2d, _) = morton.projected_axis_morton(axis);
        let plane_size = (PLANE_AREA >> (morton.depth.as_usize() << 1)) * MATERIAL_BITS;
        let material_splat64 = splat64(material);
        let index = morton_2d.as_usize() * MATERIAL_BITS;
        let plane_bits = &mut self.cells[axis.as_usize()][index..index + plane_size];

        match plane_bits.domain_mut() {
            bitvec::domain::Domain::Enclave(mut v) => {
                v.store_value(material_splat64);
            }
            bitvec::domain::Domain::Region {
                mut head,
                body,
                mut tail,
            } => {
                head.iter_mut().for_each(|h| {
                    h.store_value(material_splat64);
                });
                body.iter_mut().for_each(|v| *v = material_splat64);
                tail.iter_mut().for_each(|h| {
                    h.store_value(material_splat64);
                });
            }
        };
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|c| c.fill(false));
    }

    #[inline(always)]
    fn material_plane(&self, axis: Axis) -> &[u16] {
        let plane_bits = &self.cells[axis.as_usize()];

        match plane_bits.domain() {
            bitvec::domain::Domain::Enclave(_) => {
                panic!("A material plane must have 1024 bits")
            }
            bitvec::domain::Domain::Region { head, body, tail } => {
                debug_assert!(
                    head.is_none(),
                    "A material plane must be aligned with {}bit grid; found unaligned head.",
                    STORAGE_WORD
                );
                debug_assert!(
                    tail.is_none(),
                    "A material plane must be aligned with {}bit grid; found unaligned tail.",
                    STORAGE_WORD
                );
                bytemuck::cast_slice(body)
            }
        }
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

#[inline(always)]
const fn splat64(material: u16) -> u64 {
    let material_u64 = material as u64;
    material_u64 | (material_u64 << 16) | (material_u64 << 32) | (material_u64 << 48)
}

use crate::{
    Coords2D,
    axis::*,
    meshing::bitfields::{PLANE_AREA, PLANE_SIDE, STORAGE_WORD, Slice},
};

#[repr(transparent)]
pub struct Occlusion<'a> {
    pub masks: [&'a mut Slice; 3],
}

impl<'a> Occlusion<'a> {
    pub fn mark_done(&mut self, axis: Axis, axis_coord: usize, shape: &[u32]) {
        let index = axis_coord * PLANE_AREA;
        match self.masks[axis.as_usize()][index..index + PLANE_AREA].domain_mut() {
            bitvec::domain::Domain::Enclave(_) => {
                panic!("A parity transition matrix must be of 4KB",)
            }
            bitvec::domain::Domain::Region { head, body, tail } => {
                debug_assert!(
                    head.is_none(),
                    "A parity transition matrix must be aligned with {}bit grid; found unaligned head.",
                    STORAGE_WORD
                );
                debug_assert!(
                    tail.is_none(),
                    "A parity transition matrix must be aligned with {}bit grid; found unaligned tail.",
                    STORAGE_WORD
                );
                bytemuck::cast_slice_mut::<_, u32>(body).copy_from_slice(shape);
            }
        }
    }

    pub fn is_pending(&self, axis: Axis, axis_coord: usize, uv: Coords2D) -> bool {
        let index = axis_coord * PLANE_AREA + uv.v as usize * PLANE_SIDE + uv.u as usize;
        self.masks[axis.as_usize()][index]
    }
}

use crate::meshing::{bitfields::Parity, surface_layer::SurfaceLayer};

pub struct Edges<'a> {
    current: &'a [u32],
    previous: Option<&'a [u32]>,
}

impl<'a> Edges<'a> {
    #[inline(always)]
    pub fn from_parity(parity: &'a Parity<'a>, layer: SurfaceLayer) -> Edges<'a> {
        let rows_axis = layer.axis.next().next();
        Edges {
            current: parity.gather_edges(SurfaceLayer::new(rows_axis, layer.layer)),
            previous: (layer.layer > 0)
                .then(|| parity.gather_edges(SurfaceLayer::new(rows_axis, layer.layer - 1))),
        }
    }

    pub fn walls(&self, row: usize) -> u32 {
        self.current[row] | self.previous.map_or(0, |p| p[row])
    }
}

use super::*;

// #[derive(Copy, Clone, Debug)]
// pub struct Routing {
//     pub out_pos_x: usize,
//     pub out_pos_y: usize,
//     pub out_pos_z: usize,
//     pub out_neg_x: usize,
//     pub out_neg_y: usize,
//     pub out_neg_z: usize,
//     pub inn_pos_x: usize,
//     pub inn_pos_y: usize,
//     pub inn_pos_z: usize,
//     pub inn_neg_x: usize,
//     pub inn_neg_y: usize,
//     pub inn_neg_z: usize,
// }
#[derive(Copy, Clone, Debug)]
pub struct Routing {
    // 2 External Shell Registers per axis (Scalar indices)
    pub out_pos_x: usize,
    pub out_pos_y: usize,
    pub out_pos_z: usize,
    pub out_neg_x: usize,
    pub out_neg_y: usize,
    pub out_neg_z: usize,

    // 3 Generational Internal Registers per axis (Array of 3 slots)
    pub inn_pos_x: [usize; 3],
    pub inn_pos_y: [usize; 3],
    pub inn_pos_z: [usize; 3],
    pub inn_neg_x: [usize; 3],
    pub inn_neg_y: [usize; 3],
    pub inn_neg_z: [usize; 3],
}

impl Default for Routing {
    fn default() -> Self {
        Self {
            out_pos_x: OUT_POS_X_IDX,
            out_pos_y: OUT_POS_Y_IDX,
            out_pos_z: OUT_POS_Z_IDX,
            out_neg_x: OUT_NEG_X_IDX,
            out_neg_y: OUT_NEG_Y_IDX,
            out_neg_z: OUT_NEG_Z_IDX,
            inn_pos_x: [INN_POS_X_IDX, INN_POS_X_1_IDX, INN_POS_X_2_IDX],
            inn_pos_y: [INN_POS_Y_IDX, INN_POS_Y_1_IDX, INN_POS_Y_2_IDX],
            inn_pos_z: [INN_POS_Z_IDX, INN_POS_Z_1_IDX, INN_POS_Z_2_IDX],
            inn_neg_x: [INN_NEG_X_IDX, INN_NEG_X_1_IDX, INN_NEG_X_2_IDX],
            inn_neg_y: [INN_NEG_Y_IDX, INN_NEG_Y_1_IDX, INN_NEG_Y_2_IDX],
            inn_neg_z: [INN_NEG_Z_IDX, INN_NEG_Z_1_IDX, INN_NEG_Z_2_IDX],
        }
    }
}

// impl Routing {
//     #[inline(always)]
//     pub fn focus_octant(&self, octant: u8) -> Routing {
//         let is_pos_x = (octant & 1) as usize;
//         let is_pos_y = ((octant >> 1) & 1) as usize;
//         let is_pos_z = ((octant >> 2) & 1) as usize;
//
//         Routing {
//             out_pos_x: [self.inn_neg_x, self.out_pos_x][is_pos_x],
//             out_pos_y: [self.inn_neg_y, self.out_pos_y][is_pos_y],
//             out_pos_z: [self.inn_neg_z, self.out_pos_z][is_pos_z],
//
//             out_neg_x: [self.out_neg_x, self.inn_pos_x][is_pos_x],
//             out_neg_y: [self.out_neg_y, self.inn_pos_y][is_pos_y],
//             out_neg_z: [self.out_neg_z, self.inn_pos_z][is_pos_z],
//
//             inn_pos_x: [self.out_pos_x, self.inn_neg_x][is_pos_x],
//             inn_pos_y: [self.inn_pos_y, self.inn_neg_y][is_pos_y],
//             inn_pos_z: [self.inn_pos_z, self.inn_neg_z][is_pos_z],
//
//             inn_neg_x: [self.inn_pos_x, self.out_neg_x][is_pos_x],
//             inn_neg_y: [self.inn_neg_y, self.inn_pos_y][is_pos_y],
//             inn_neg_z: [self.inn_neg_z, self.inn_pos_z][is_pos_z],
//         }
//     }
// }

impl Routing {
    #[inline(always)]
    pub fn focus_octant(&self, octant: u8, current_depth: u8) -> Routing {
        let is_pos_x = (octant & 1) as usize;
        let is_pos_y = ((octant >> 1) & 1) as usize;
        let is_pos_z = ((octant >> 2) & 1) as usize;

        let p_idx = (current_depth % 3) as usize;
        let c_idx = ((current_depth + 1) % 3) as usize;

        let p_inn_pos_x = self.inn_pos_x[p_idx];
        let p_inn_neg_x = self.inn_neg_x[p_idx];
        let p_inn_pos_y = self.inn_pos_y[p_idx];
        let p_inn_neg_y = self.inn_neg_y[p_idx];
        let p_inn_pos_z = self.inn_pos_z[p_idx];
        let p_inn_neg_z = self.inn_neg_z[p_idx];

        let mut child_routing = Routing {
            out_pos_x: [p_inn_neg_x, self.out_pos_x][is_pos_x],
            out_pos_y: [p_inn_neg_y, self.out_pos_y][is_pos_y],
            out_pos_z: [p_inn_neg_z, self.out_pos_z][is_pos_z],

            out_neg_x: [self.out_neg_x, p_inn_pos_x][is_pos_x],
            out_neg_y: [self.out_neg_y, p_inn_pos_y][is_pos_y],
            out_neg_z: [self.out_neg_z, p_inn_pos_z][is_pos_z],

            inn_pos_x: self.inn_pos_x,
            inn_pos_y: self.inn_pos_y,
            inn_pos_z: self.inn_pos_z,
            inn_neg_x: self.inn_neg_x,
            inn_neg_y: self.inn_neg_y,
            inn_neg_z: self.inn_neg_z,
        };

        child_routing.inn_pos_x[c_idx] = [self.out_pos_x, p_inn_neg_x][is_pos_x];
        child_routing.inn_pos_y[c_idx] = [self.inn_pos_y[c_idx], p_inn_neg_y][is_pos_y];
        child_routing.inn_pos_z[c_idx] = [self.inn_pos_z[c_idx], p_inn_neg_z][is_pos_z];

        child_routing.inn_neg_x[c_idx] = [p_inn_pos_x, self.out_neg_x][is_pos_x];
        child_routing.inn_neg_y[c_idx] = [self.inn_neg_y[c_idx], p_inn_pos_y][is_pos_y];
        child_routing.inn_neg_z[c_idx] = [self.inn_neg_z[c_idx], p_inn_pos_z][is_pos_z];

        child_routing
    }
}

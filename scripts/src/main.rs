const LDB: u8 = 0; // Bit 0
const RDB: u8 = 1; // Bit 1
const LUB: u8 = 2; // Bit 2
const RUB: u8 = 3; // Bit 3
const LDF: u8 = 4; // Bit 4
const RDF: u8 = 5; // Bit 5
const LUF: u8 = 6; // Bit 6
const RUF: u8 = 7; // Bit 7

/// Returns a 24-bit mask representing the visible faces needed to wrap the solid blocks.
///
/// Face layout (4 bits each):
/// Bits 0-3: U, Bits 4-7: D, Bits 8-11: F, Bits 12-15: B, Bits 16-19: L, Bits 20-23: R
/// Quadrant layout within face: Bit+0: TL, Bit+1: TR, Bit+2: BL, Bit+3: BR (from front/top view)
fn compute_visible_faces(solidity_mask: u8) -> u32 {
    let mut visible_mask: u32 = 0;

    // Helper macro to check if a block is solid
    let is_solid = |block_bit: u8| -> bool {
        (solidity_mask & (1 << block_bit)) != 0
    };

    // Helper macro to set a specific face bit
    let mut set_facet = |face_offset: u32, quadrant: u32| {
        visible_mask |= 1 << (face_offset + quadrant);
    };

    // Face Bit Offsets
    let u_off = 0;
    let d_off = 4;
    let f_off = 8;
    let b_off = 12;
    let l_off = 16;
    let r_off = 20;

    // Evaluate each of the 8 blocks. A facet is only drawn if:
    // 1. The block itself IS solid.
    // 2. The neighbor block in that direction IS NOT solid (culling).

    // --- BLOCK 0: LDB ---
    if is_solid(LDB) {
        if !is_solid(LUB) { set_facet(d_off, 2); } // Hidden by Up? No -> Exposes Down face
        if !is_solid(LDF) { set_facet(b_off, 2); } // Hidden by Front? No -> Exposes Back face
        if !is_solid(RDB) { set_facet(l_off, 2); } // Hidden by Right? No -> Exposes Left face
    }

    // --- BLOCK 1: RDB ---
    if is_solid(RDB) {
        if !is_solid(RUB) { set_facet(d_off, 3); }
        if !is_solid(RDF) { set_facet(b_off, 3); }
        if !is_solid(LDB) { set_facet(r_off, 2); }
    }

    // --- BLOCK 2: LUB ---
    if is_solid(LUB) {
        if !is_solid(LDB) { set_facet(u_off, 0); } // Hidden by Down? No -> Exposes Up face
        if !is_solid(LUF) { set_facet(b_off, 0); }
        if !is_solid(RUB) { set_facet(l_off, 0); }
    }

    // --- BLOCK 3: RUB ---
    if is_solid(RUB) {
        if !is_solid(RDB) { set_facet(u_off, 1); }
        if !is_solid(RUF) { set_facet(b_off, 1); }
        if !is_solid(LUB) { set_facet(r_off, 0); }
    }

    // --- BLOCK 4: LDF ---
    if is_solid(LDF) {
        if !is_solid(LUF) { set_facet(d_off, 0); }
        if !is_solid(LDB) { set_facet(f_off, 2); } // Hidden by Back? No -> Exposes Front face
        if !is_solid(RDF) { set_facet(l_off, 3); }
    }

    // --- BLOCK 5: RDF ---
    if is_solid(RDF) {
        if !is_solid(RUF) { set_facet(d_off, 1); }
        if !is_solid(RDB) { set_facet(f_off, 3); }
        if !is_solid(LDF) { set_facet(r_off, 3); }
    }

    // --- BLOCK 6: LUF ---
    if is_solid(LUF) {
        if !is_solid(LDF) { set_facet(u_off, 2); }
        if !is_solid(LUB) { set_facet(f_off, 0); }
        if !is_solid(RUF) { set_facet(l_off, 1); }
    }

    // --- BLOCK 7: RUF ---
    if is_solid(RUF) {
        if !is_solid(RDF) { set_facet(u_off, 3); }
        if !is_solid(RUB) { set_facet(f_off, 1); }
        if !is_solid(LUF) { set_facet(r_off, 1); }
    }

    visible_mask
}

fn main() {
    println!("// Look-Up Table mapping 8-bit solidity mask to 24-bit visible face mask");
    println!("// Format: 0x[FaceMask] (Bits: RRRR_LLLL_BBBB_FFFF_DDDD_UUUU)");
    println!("pub static VISIBLE_FACES_LUT: [u32; 256] = [");

    for solidity_mask in 0..=255 {
        let face_mask = compute_visible_faces(solidity_mask);

        // Print 8 elements per line for clean formatting
        if solidity_mask % 8 == 0 {
            print!("    ");
        }

        print!("0x{:06X}, ", face_mask);

        if solidity_mask % 8 == 7 {
            println!("// masks {}-{:#04X}", solidity_mask - 7, solidity_mask);
        }
    }

    println!("];");
}

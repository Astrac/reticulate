mod dense_print_walker;
mod print_walker;

pub mod bitmasks;

pub use dense_print_walker::DensePrintWalker;
pub use print_walker::PrintWalker;

use crate::{
    axis::*,
    meshing::bitfields::{Material, MaterialId, Parity},
};

pub fn print_bitfield(bits: &[u32]) {
    println!("{}", "-".repeat(32));
    print_bits(bits);
    println!("{}", "-".repeat(32));
}

pub fn print_parity(parity: &Parity<'_>) {
    for axis in [AXIS_X /*, AXIS_Y, AXIS_Z*/] {
        for layer in 0..3 {
            println!("-- {}{layer:02} {}", axis.name(), "-".repeat(25));
            print_bits(parity.masks[axis.as_usize()].get_layer(layer));
            println!("{}", "-".repeat(32));
        }
    }
}

pub fn print_materials(materials: &Material<'_>) {
    for axis in [AXIS_X /*, AXIS_Y, AXIS_Z*/] {
        let data = &mut materials.cells[axis.as_usize()].domain().flat_map(|x| {
            bytemuck::cast::<_, [MaterialId; 8 / size_of::<MaterialId>()]>(x.to_le_bytes())
        });

        println!("-- {} {}", axis.name(), "-".repeat(27));
        for _ in 0..32 {
            for _ in 0..32 {
                let Some(m) = data.next() else { unreachable!() };
                print!("{m:04x} ");
            }
            print!("\n");
        }
        println!("{}", "-".repeat(32));
    }
}

fn print_bits(bits: &[u32]) {
    for row in bits {
        println!(
            "{}",
            format!("{row:032b}").replace("1", "x").replace("0", ".")
        );
    }
}

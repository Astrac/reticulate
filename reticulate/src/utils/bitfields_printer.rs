use std::ops::Range;

use crate::{
    Axis, MaterialId,
    meshing::bitfields::{Material, Parity},
};

pub fn print_bitfield_slice(bits: &[u32]) {
    println!("{}", "-".repeat(32));
    print_bits(bits);
    println!("{}", "-".repeat(32));
}

pub fn print_parity(parity: &Parity<'_>, axes: &[Axis], layers: Range<usize>) {
    for axis in axes {
        for layer in layers.clone() {
            println!("-- {}{layer:02} {axis:?}", "-".repeat(25));
            print_bits(parity.masks[axis.as_usize()].get_layer(layer));
            println!("{}", "-".repeat(32));
        }
    }
}

pub fn print_materials(materials: &Material<'_>, axes: &[Axis]) {
    for axis in axes {
        let data = &materials.planes[axis.as_usize()];

        println!("-- {} {axis:?}", "-".repeat(27));
        for row in 0..32 {
            for column in 0..32 {
                let m = data[row * 32 + column];
                print!("{m:04x} ");
            }
            println!();
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

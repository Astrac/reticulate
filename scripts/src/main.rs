mod gilbert;

use std::{fs::File, io::Write};

use bilge::prelude::*;
use bytemuck::{AnyBitPattern, NoUninit, bytes_of};

const BOUNDS: (i32, i32, i32) = (32, 64, 32);

#[bitsize(16)]
#[derive(Copy, Clone, FromBits)]
struct Coords {
    x: u5,
    y: u6,
    z: u5,
}

#[derive(Copy, Clone, Debug, NoUninit, AnyBitPattern)]
#[repr(transparent)]
struct StoredCoords(u16);

impl Coords {
    fn from_index(index: u16) -> Coords {
        let (x, y, z) = gilbert::gilbert::d2xyz(index as i32, BOUNDS);
        Coords::new(u5::from_(x), u6::from_(y), u5::from_(z))
    }
}

fn write_lut() -> std::io::Result<()> {
    let mut file = File::create("chunk_lut.bin")?;
    for index in 0..u16::MAX {
        let coords = Coords::from_index(index);
        let stored = StoredCoords(u16::from(coords));
        file.write_all(bytes_of(&stored))?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    write_lut()?;

    Ok(())
}

// fn to_coords(&self) -> Coords<Self::Coord> {
//     let _ = <Self as Key<W, H, D>>::KEY_ASSERTIONS;
//     // This is not bound to W/H/D!!!
//     // Bits 0..5
//     let x = (self & 0x3F) as u8;
//     // Bits 6..11
//     let y = ((self >> 6) & 0x3F) as u8;
//     // Bits 12..15
//     let z = (self >> 12) as u8;

//     Coords { x, y, z }
// }

// fn from_coords(value: Coords<Self::Coord>) -> Self {
//     let _ = <Self as Key<W, H, D>>::KEY_ASSERTIONS;
//     // This is not bound to W/H/D!!!
//     let x = (value.x as u16) & 0x3F;
//     let y = (value.y as u16) & 0x3F;
//     let z = (value.z as u16) & 0x0F;

//     x | (y << 6) | (z << 12)
// }

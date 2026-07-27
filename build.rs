mod magic_bitboards;
mod magic_generator;

use std::{env, fs::File, io::Write, path::Path};
use bytemuck::cast_slice;
use crate::{
    magic_bitboards::{BISHOP_MAGIC, ROOK_MAGIC},
    magic_generator::{BISHOP_OFFSETS, ROOK_OFFSETS, compute_blockers, compute_magic},
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=magic_bitboards.rs");
    println!("cargo:rerun-if-changed=magic_generator.rs");

    let out_dir = env::var("OUT_DIR").unwrap();

    let path = Path::new(&out_dir).join("bishop_blockers.bin");
    let mut file = File::create(path).unwrap();
    let bishop_blockers: [u64; 64] = compute_blockers(&BISHOP_OFFSETS);
    for value in bishop_blockers {
        file.write_all(&value.to_le_bytes()).unwrap();
    }

    let path = Path::new(&out_dir).join("bishop_attackers.bin");
    let mut file = File::create(path).unwrap();
    let data: [[u64; 512]; 64] = compute_magic(&BISHOP_OFFSETS, bishop_blockers, 9, BISHOP_MAGIC);
    file.write_all(cast_slice(&data)).unwrap();

    let path = Path::new(&out_dir).join("rook_blockers.bin");
    let mut file = File::create(path).unwrap();
    let rook_blockers: [u64; 64] = compute_blockers(&ROOK_OFFSETS);
    for value in rook_blockers {
        file.write_all(&value.to_le_bytes()).unwrap();
    }

    let path = Path::new(&out_dir).join("rook_attackers.bin");
    let mut file = File::create(path).unwrap();
    let data: [[u64; 4096]; 64] = compute_magic(&ROOK_OFFSETS, rook_blockers, 12, ROOK_MAGIC);
    file.write_all(cast_slice(&data)).unwrap();
}

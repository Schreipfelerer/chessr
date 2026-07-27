// src/lib.rs
pub mod board;
pub mod movegen;
pub mod magic;

#[path = "../magic_generator.rs"]
pub mod magic_generator;
#[path = "../magic_bitboards.rs"]
pub mod magic_bitboards;

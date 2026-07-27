#[path = "../magic_bitboards.rs"]
mod magic_bitboards;
use crate::{
    board::Sq64,
    magic::magic_bitboards::{BISHOP_MAGIC, ROOK_MAGIC},
};


#[repr(align(8))]
struct AlignedBytes<const N: usize>([u8; N]);

static BISHOP_BYTES_BLOCKER: AlignedBytes<512> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/bishop_blockers.bin"
)));
static BISHOP_BYTES_ATTACKS: AlignedBytes<262_144> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/bishop_attackers.bin"
)));
static ROOK_BYTES_BLOCKER: AlignedBytes<512> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/rook_blockers.bin"
)));
static ROOK_BYTES_ATTACKS: AlignedBytes<2_097_152> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/rook_attackers.bin"
)));

pub static BISHOP_ATTACKS: &[[u64; 512]; 64] =
    unsafe { &*(BISHOP_BYTES_ATTACKS.0.as_ptr() as *const [[u64; 512]; 64]) };
pub static ROOK_ATTACKS: &[[u64; 4096]; 64] =
    unsafe { &*(ROOK_BYTES_ATTACKS.0.as_ptr() as *const [[u64; 4096]; 64]) };
pub static BISHOP_BLOCKERS: &[u64; 64] =
    unsafe { &*(BISHOP_BYTES_BLOCKER.0.as_ptr() as *const [u64; 64]) };
pub static ROOK_BLOCKERS: &[u64; 64] =
    unsafe { &*(ROOK_BYTES_BLOCKER.0.as_ptr() as *const [u64; 64]) };

pub fn get_bishop_moves(sq: Sq64, bb: u64) -> u64 {
    let sq_ind = sq.0 as usize;
    BISHOP_ATTACKS[sq_ind][magic_index(sq.0, bb & BISHOP_BLOCKERS[sq_ind], 9, BISHOP_MAGIC)]
}
pub fn get_rook_moves(sq: Sq64, bb: u64) -> u64 {
    let sq_ind = sq.0 as usize;
    ROOK_ATTACKS[sq_ind][magic_index(sq.0, bb & ROOK_BLOCKERS[sq_ind], 12, ROOK_MAGIC)]
}
const fn magic_index(sq: u8, bb: u64, bits: u8, magic_table: [u64; 64]) -> usize {
    (bb.wrapping_mul(magic_table[sq as usize]) >> 64 - bits) as usize
}

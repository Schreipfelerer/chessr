use std::arch::x86_64::_pext_u64;

use crate::board::Sq64;

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
static BETWEEN_BYTES: AlignedBytes<32_768> =
    AlignedBytes(*include_bytes!(concat!(env!("OUT_DIR"), "/between.bin")));

pub static BISHOP_ATTACKS: &[[u64; 512]; 64] =
    unsafe { &*(BISHOP_BYTES_ATTACKS.0.as_ptr() as *const [[u64; 512]; 64]) };
pub static ROOK_ATTACKS: &[[u64; 4096]; 64] =
    unsafe { &*(ROOK_BYTES_ATTACKS.0.as_ptr() as *const [[u64; 4096]; 64]) };
pub static BISHOP_BLOCKERS: &[u64; 64] =
    unsafe { &*(BISHOP_BYTES_BLOCKER.0.as_ptr() as *const [u64; 64]) };
pub static ROOK_BLOCKERS: &[u64; 64] =
    unsafe { &*(ROOK_BYTES_BLOCKER.0.as_ptr() as *const [u64; 64]) };
pub static BETWEEN: &[[u64; 64]; 64] =
    unsafe { &*(BETWEEN_BYTES.0.as_ptr() as *const [[u64; 64]; 64]) };

#[inline(always)]
pub fn get_bishop_moves(sq: Sq64, bb: u64) -> u64 {
    BISHOP_ATTACKS[sq.ind()][pext_index(bb, BISHOP_BLOCKERS[sq.ind()])]
}
#[inline(always)]
pub fn get_rook_moves(sq: Sq64, bb: u64) -> u64 {
    ROOK_ATTACKS[sq.ind()][pext_index(bb, ROOK_BLOCKERS[sq.ind()])]
}
#[inline(always)]
fn pext_index(bb: u64, mask: u64) -> usize {
    unsafe { _pext_u64(bb, mask) as usize }
}

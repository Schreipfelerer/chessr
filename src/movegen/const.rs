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
    "/bishop_attacks.bin"
)));
static ROOK_BYTES_BLOCKER: AlignedBytes<512> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/rook_blockers.bin"
)));
static ROOK_BYTES_ATTACKS: AlignedBytes<2_097_152> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/rook_attacks.bin"
)));
static PAWN_BYTES_ATTACKS: AlignedBytes<1024> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/pawn_attacks.bin"
)));
static KNIGHT_BYTES_ATTACKS: AlignedBytes<512> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/knight_attacks.bin"
)));
static KING_BYTES_ATTACKS: AlignedBytes<512> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/king_attacks.bin"
)));
static BETWEEN_BYTES: AlignedBytes<32_768> =
    AlignedBytes(*include_bytes!(concat!(env!("OUT_DIR"), "/between.bin")));

pub static PAWN_ATTACKS: &[[u64; 64]; 2] =
    unsafe { &*(PAWN_BYTES_ATTACKS.0.as_ptr() as *const [[u64; 64]; 2]) };
pub static KNIGHT_ATTACKS: &[u64; 64] =
    unsafe { &*(KNIGHT_BYTES_ATTACKS.0.as_ptr() as *const [u64; 64]) };
pub static KING_ATTACKS: &[u64; 64] =
    unsafe { &*(KING_BYTES_ATTACKS.0.as_ptr() as *const [u64; 64]) };
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
    BISHOP_ATTACKS[sq.ind()][unsafe { _pext_u64(bb, BISHOP_BLOCKERS[sq.ind()]) as usize }]
}
#[inline(always)]
pub fn get_rook_moves(sq: Sq64, bb: u64) -> u64 {
    ROOK_ATTACKS[sq.ind()][unsafe { _pext_u64(bb, ROOK_BLOCKERS[sq.ind()]) as usize }]
}

#[repr(align(8))]
struct AlignedBytes<const N: usize>([u8; N]);

static ZOBRIST_PIECES_BYTES: AlignedBytes<6_144> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/zobrist_pieces.bin"
)));
static ZOBRIST_CASTLING_BYTES: AlignedBytes<128> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/zobrist_castling.bin"
)));
static ZOBRIST_EP_BYTES: AlignedBytes<64> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/zobrist_ep.bin"
)));
static ZOBRIST_SIDE_BYTES: AlignedBytes<8> = AlignedBytes(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/zobrist_side.bin"
)));

pub static ZOBRIST_PIECES: &[[[u64; 64]; 6]; 2] =
    unsafe { &*ZOBRIST_PIECES_BYTES.0.as_ptr().cast::<[[[u64; 64]; 6]; 2]>() };
pub static ZOBRIST_CASTLING: &[u64; 16] =
    unsafe { &*ZOBRIST_CASTLING_BYTES.0.as_ptr().cast::<[u64; 16]>() };
pub static ZOBRIST_EP: &[u64; 8] =
    unsafe { &*ZOBRIST_EP_BYTES.0.as_ptr().cast::<[u64; 8]>() };
pub static ZOBRIST_SIDE: &u64 =
    unsafe { &*ZOBRIST_SIDE_BYTES.0.as_ptr().cast::<u64>() };

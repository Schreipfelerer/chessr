use crate::board::Sq64;

pub const BISHOP_BLOCKERS: [u64; 64] = compute_blockers(&BISHOP_OFFSETS);
#[allow(long_running_const_eval)]
pub const BISHOP_ATTACKS: [[u64; 512]; 64] =
    compute_magic(&BISHOP_OFFSETS, BISHOP_BLOCKERS, 9, BISHOP_MAGIC);
pub const BISHOP_OFFSETS: [i8; 4] = [15, 17, -15, -17];

pub const ROOK_BLOCKERS: [u64; 64] = compute_blockers(&ROOK_OFFSETS);
#[allow(long_running_const_eval)]
pub const ROOK_ATTACKS: [[u64; 4096]; 64] =
    compute_magic(&ROOK_OFFSETS, ROOK_BLOCKERS, 12, ROOK_MAGIC);
pub const ROOK_OFFSETS: [i8; 4] = [1, -1, 16, -16];

pub const fn get_bishop_moves(sq: Sq64, bb: u64) -> u64 {
    let sq_ind = sq.0 as usize;
    BISHOP_ATTACKS[sq_ind][magic_index(sq.0, bb & BISHOP_BLOCKERS[sq_ind], 9, BISHOP_MAGIC)]
}

pub const fn get_rook_moves(sq: Sq64, bb: u64) -> u64 {
    let sq_ind = sq.0 as usize;
    ROOK_ATTACKS[sq_ind][magic_index(sq.0, bb & ROOK_BLOCKERS[sq_ind], 12, ROOK_MAGIC)]
}

const fn compute_magic<const N: usize>(
    offsets: &[i8; 4],
    blockers: [u64; 64],
    bits: u8,
    magic_table: [u64; 64],
) -> [[u64; N]; 64] {
    let mut table = [[0u64; N]; 64];
    let mut sq = 0u8;
    while sq < 64 {
        table[sq as usize] = compute_magic_square(sq, offsets, blockers, bits, magic_table);
        sq += 1;
    }
    table
}

const fn compute_magic_square<const N: usize>(
    sq: u8,
    offsets: &[i8; 4],
    blockers: [u64; 64],
    bits: u8,
    magic_table: [u64; 64],
) -> [u64; N] {
    let bb = blockers[sq as usize];
    let mut table = [0u64; N];
    let mut current_mask = bb;
    while current_mask != 0 {
        table[magic_index(sq, current_mask, bits, magic_table)] =
            compute_sliding_attacks(sq, current_mask, offsets);
        current_mask = current_mask.wrapping_sub(1) & bb
    }

    table[0] = compute_sliding_attacks(sq, 0u64, offsets);
    table
}

pub const fn compute_sliding_attacks(sq: u8, blockers: u64, offsets: &[i8]) -> u64 {
    let mut bb = 0u64;
    let sq_88 = Sq64(sq).to_sq88();
    let mut i = 0;
    while i < offsets.len() {
        let offset = offsets[i];
        let mut to_sq = sq_88.step(offset);
        while to_sq.is_on_board() {
            let to_sq_64 = to_sq.to_sq64();
            let mask = 1 << to_sq_64.0;
            bb |= mask;
            if blockers & mask != 0 {
                break;
            }
            to_sq = to_sq.step(offset)
        }
        i += 1;
    }
    bb
}

const fn magic_index(sq: u8, bb: u64, bits: u8, magic_table: [u64; 64]) -> usize {
    (bb.wrapping_mul(magic_table[sq as usize]) >> 64 - bits) as usize
}

const fn compute_blockers(offsets: &[i8; 4]) -> [u64; 64] {
    let mut table = [0u64; 64];
    let mut sq = 0u8;
    while sq < 64 {
        let sq_88 = Sq64(sq).to_sq88();
        let mut bb = 0u64;
        let mut i = 0;
        while i < 4 {
            let offset = offsets[i];
            let mut to_88 = sq_88.step(offset);
            let mut prev_sq: Option<Sq64> = None;
            while to_88.is_on_board() {
                if let Some(sq) = prev_sq {
                    bb |= 1u64 << sq.0;
                }
                prev_sq = Some(to_88.to_sq64());
                to_88 = to_88.step(offset);
            }
            i += 1;
        }
        table[sq as usize] = bb;
        sq += 1;
    }
    table
}
const BISHOP_MAGIC: [u64; 64] = [
    0x0C208C0148082004,
    0x01100031000A0800,
    0x000D00A004100804,
    0x0284040080044820,
    0x0090041800802020,
    0x0D08051004050010,
    0x20044A010040C004,
    0x0000808118004009,
    0x010040A008808020,
    0x2000010010E20005,
    0x0202419080C00380,
    0x1509020084082024,
    0xA00800B020140038,
    0x1109020904100040,
    0x0000001082024000,
    0x001010A014018421,
    0x4010424801004441,
    0x00011040A8800844,
    0xB9009A100821A10C,
    0x0010800140800203,
    0x0180A08400200002,
    0x102A004049002018,
    0x0008081020086200,
    0x0088281031002941,
    0x000E404030080040,
    0x0001110481242008,
    0x4024030230090020,
    0x0004040108012004,
    0x40060010A2005008,
    0x0002109000481000,
    0x4000401004D20620,
    0x4022002201088200,
    0x2000201002050440,
    0x0B04124040280100,
    0x0402521000804900,
    0x80A0820080080082,
    0x0240002088020080,
    0x0250004440408010,
    0x0009808400844400,
    0x29491010046060A2,
    0x0020708428243000,
    0x0100041001028808,
    0x0201000888101002,
    0x0009400441020080,
    0x080031046050A104,
    0x80280122080A0080,
    0x61100489260C0500,
    0x3004818031000200,
    0x4000800800E10208,
    0x8210130038108400,
    0x000000800C10A030,
    0x1082800090088040,
    0x4000000850110000,
    0x8D00281811806044,
    0x1240020088620098,
    0x0020120202012200,
    0x0051002020201000,
    0x002244820A081000,
    0x9400200200440A60,
    0x0040200004500420,
    0x00040000C0110460,
    0x7020101050040090,
    0x2208060220280200,
    0x0000A22C01220600,
];

const ROOK_MAGIC: [u64; 64] = [
    0x808001C002108020,
    0x0080400010006000,
    0x4020040010201800,
    0x8200040200201008,
    0x0408000834010404,
    0x0080140002002080,
    0x0A00104895020001,
    0x0200004204A88104,
    0x9014400024401082,
    0x0240800840008030,
    0x1000200820220410,
    0x000010040100300A,
    0x0400080430804200,
    0x0002012004005008,
    0x4038C04001180480,
    0x120080028812E041,
    0x0010200804051000,
    0x0109280502900600,
    0x00004A2000C80C00,
    0x8010011800020082,
    0x0280012010020120,
    0x0020080110340004,
    0x000080C00A080120,
    0x08104419020C5300,
    0x0415400280108000,
    0x0020211090040042,
    0x8088040020000808,
    0x0490010900021100,
    0x0068801000840824,
    0x0000101800600440,
    0x8000200180014240,
    0x0010204020004500,
    0x0000640140400012,
    0x0010802011010403,
    0x011200C808008220,
    0x0018080005400600,
    0x0000291242200804,
    0x00005C8010040200,
    0x4021C00043500200,
    0x000000B1008000C0,
    0x4050200010006000,
    0x2000400024103400,
    0x0000200010003900,
    0x0004201110001002,
    0x0040028C00101005,
    0x0060020108210C00,
    0x0010208002609104,
    0x2080024020801300,
    0x0000801300104B00,
    0x0000400025280218,
    0x22027040140812C0,
    0x0002010C46404200,
    0x00001448AA401201,
    0x4800045E00071080,
    0x04A0060002010201,
    0x60E0120100440D20,
    0x0410410820128202,
    0x1000108040240901,
    0x0106004020304682,
    0xC101424890200202,
    0x0040100800024401,
    0x0201220284200801,
    0x0080410200628904,
    0x4204010180402402,
];

use crate::board::Sq64;

pub const BISHOP_BLOCKERS: [u64; 64] = compute_blockers(&BISHOP_OFFSETS);
pub const BISHOP_ATTACKS: [[u64; 512]; 64] = compute_magic_bishops();
pub const BISHOP_MAGIC: [u64; 64] = [0x0C208C0148082004, 0x01100031000A0800, 0x000D00A004100804, 0x0284040080044820, 0x0090041800802020, 0x0D08051004050010, 0x20044A010040C004, 0x0000808118004009, 0x010040A008808020, 0x2000010010E20005, 0x0202419080C00380, 0x1509020084082024, 0xA00800B020140038, 0x1109020904100040, 0x0000001082024000, 0x001010A014018421, 0x4010424801004441, 0x00011040A8800844, 0xB9009A100821A10C, 0x0010800140800203, 0x0180A08400200002, 0x102A004049002018, 0x0008081020086200, 0x0088281031002941, 0x000E404030080040, 0x0001110481242008, 0x4024030230090020, 0x0004040108012004, 0x40060010A2005008, 0x0002109000481000, 0x4000401004D20620, 0x4022002201088200, 0x2000201002050440, 0x0B04124040280100, 0x0402521000804900, 0x80A0820080080082, 0x0240002088020080, 0x0250004440408010, 0x0009808400844400, 0x29491010046060A2, 0x0020708428243000, 0x0100041001028808, 0x0201000888101002, 0x0009400441020080, 0x080031046050A104, 0x80280122080A0080, 0x61100489260C0500, 0x3004818031000200, 0x4000800800E10208, 0x8210130038108400, 0x000000800C10A030, 0x1082800090088040, 0x4000000850110000, 0x8D00281811806044, 0x1240020088620098, 0x0020120202012200, 0x0051002020201000, 0x002244820A081000, 0x9400200200440A60, 0x0040200004500420, 0x00040000C0110460, 0x7020101050040090, 0x2208060220280200, 0x0000A22C01220600];
pub const BISHOP_OFFSETS: [i8; 4] = [15, 17, -15, -17];
//pub const ROOK_BLOCKERS: [u64; 64] = compute_blockers(&ROOK_OFFSETS);
//pub const ROOK_ATTACKS: [u64; 4096] = compute_magic_rooks();


pub const fn get_bishop_moves(sq: Sq64, bb: u64) -> u64{
    let sq_ind = sq.0 as usize;
    BISHOP_ATTACKS[sq_ind][magic_index_bushop(sq.0, bb & BISHOP_BLOCKERS[sq_ind])]
}

const fn compute_magic_bishops() -> [[u64; 512]; 64] {
    let mut table = [[0u64; 512]; 64];
    let mut sq = 0u8;
    while sq < 64{
        table[sq as usize] = compute_magic_bishop_square(sq);
        sq += 1;
    }
    table
}

const fn compute_magic_bishop_square(sq: u8) -> [u64; 512] {
    let bb = BISHOP_BLOCKERS[sq as usize];
    let mut table = [0u64; 512];
    let mut current_mask = bb;
    while current_mask != 0{
        table[magic_index_bushop(sq, current_mask)] = compute_sliding_attacks(sq, current_mask, &BISHOP_OFFSETS);
        current_mask = current_mask.wrapping_sub(1) & bb
    }

    table[0] = compute_sliding_attacks(sq, 0u64, &BISHOP_OFFSETS);
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
            if blockers & mask != 0{
                break;
            }
            to_sq = to_sq.step(offset)
        }
        i += 1;
    }
    bb
}

const fn magic_index_bushop(sq: u8, bb: u64) -> usize {
    (bb.wrapping_mul(BISHOP_MAGIC[sq as usize]) >> 64-9) as usize
}

const fn compute_blockers(offsets: &[i8; 4]) -> [u64; 64] {
    let mut table = [0u64; 64];
    let mut sq = 0u8;
    while sq < 64 {
        let from_0x88 = sq + (sq & !7); // same as Sq64::to_sq88
        let mut bb = 0u64;
        let mut i = 0;
        while i < 4 {
            let offset = offsets[i];
            let mut to_0x88 = (from_0x88 as i8).wrapping_add(offset) as u8;
            while to_0x88 & 0x88 == 0 {
                let to_64 = (to_0x88 + (to_0x88 & 7)) >> 1; // same as Sq88::to_sq64
                bb |= 1u64 << to_64;
                to_0x88 = to_0x88.wrapping_add(offset as u8);
            }
            i += 1;
        }
        table[sq as usize] = bb & 0x007E_7E7E_7E7E_7E00;
        sq += 1;
    }
    table
}

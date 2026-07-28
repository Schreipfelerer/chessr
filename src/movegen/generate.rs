use crate::{
    board::{Board, BoardState, Color, Move, MoveFlag, Piece, Sq64, StateInfo},
    movegen::magic::{BETWEEN, get_bishop_moves, get_rook_moves},
};
use arrayvec::ArrayVec;

// Direction offsets for sliding pieces
pub const KING_OFFSETS: [i8; 8] = [1, -1, 16, -16, 15, 17, -15, -17];
pub const KNIGHT_OFFSETS: [i8; 8] = [31, 33, 18, 14, -31, -33, -18, -14];
pub const KNIGHT_ATTACKS: [u64; 64] = compute_attacks(&KNIGHT_OFFSETS);
pub const KING_ATTACKS: [u64; 64] = compute_attacks(&KING_OFFSETS);
pub const PAWN_ATTACKS: [[u64; 64]; 2] = [compute_attacks(&[15, 17]), compute_attacks(&[-15, -17])];

const fn compute_attacks(offsets: &[i8]) -> [u64; 64] {
    let len = offsets.len();
    let mut table = [0u64; 64];
    let mut sq = 0u8;
    while sq < 64 {
        let from_0x88 = sq + (sq & !7); // same as Sq64::to_sq88
        let mut bb = 0u64;
        let mut i = 0;
        while i < len {
            let offset = offsets[i];
            let to_0x88 = (from_0x88 as i8).wrapping_add(offset) as u8;
            if to_0x88 & 0x88 == 0 {
                let to_64 = (to_0x88 + (to_0x88 & 7)) >> 1; // same as Sq88::to_sq64
                bb |= 1u64 << to_64;
            }
            i += 1;
        }
        table[sq as usize] = bb;
        sq += 1;
    }
    table
}

pub fn generate_moves(board_state: &BoardState) -> ArrayVec<Move, 256> {
    let board = &board_state.board;
    let state_info = &board_state.state_info;
    let c = state_info.active_color();
    let mut moves = ArrayVec::new();

    let attacked: u64 = compute_attacked(board, c);
    let checkers: u64 = compute_checkers(board, c);
    let pinned: ArrayVec<(Sq64, u64), 8> = compute_pins(board, c);

    match checkers.count_ones() {
        0 => {
            for from_sq in BitboardIter(board.get_friendly_occupancy(c)) {
                // If pinned
                let mut bb = !0u64;
                if let Some((_, pin_bb)) = pinned.iter().find(|(sq, _)| *sq == from_sq) {
                    bb &= pin_bb;
                }
                match board.get_piece_at(from_sq) {
                    Piece::Pawn => {
                        generate_pawn_moves(board, from_sq, c, &mut moves, state_info, bb)
                    }
                    Piece::Knight => generate_knight_moves(board, from_sq, c, &mut moves, bb),
                    Piece::Bishop => {
                        generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Bishop, bb)
                    }
                    Piece::Rook => {
                        generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Rook, bb)
                    }
                    Piece::Queen => {
                        generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Bishop, bb);
                        generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Rook, bb);
                    }
                    Piece::King => {
                        generate_king_moves(board, from_sq, c, &mut moves, attacked);
                        generate_castles(board, from_sq, c, &mut moves, state_info, attacked);
                    }
                }
            }
        }
        1 => {
            // Only inbetween or attacker or king moves
            let king_sq = board.find_king(c);
            let attacker_sq = checkers.trailing_zeros();
            let cbb = BETWEEN[king_sq.0 as usize][attacker_sq as usize] | checkers;
            for from_sq in BitboardIter(board.get_friendly_occupancy(c)) {
                let mut bb = cbb;
                // If pinned
                if let Some((_, pin_bb)) = pinned.iter().find(|(sq, _)| *sq == from_sq) {
                    bb &= pin_bb;
                }
                match board.get_piece_at(from_sq) {
                    Piece::Pawn => {
                        generate_pawn_moves(board, from_sq, c, &mut moves, state_info, bb)
                    }
                    Piece::Knight => generate_knight_moves(board, from_sq, c, &mut moves, bb),
                    Piece::Bishop => {
                        generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Bishop, bb)
                    }
                    Piece::Rook => {
                        generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Rook, bb)
                    }
                    Piece::Queen => {
                        generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Bishop, bb);
                        generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Rook, bb);
                    }
                    Piece::King => {
                        generate_king_moves(board, from_sq, c, &mut moves, attacked);
                    }
                }
            }
        }
        _ => {
            // Only King Movements
            let sq = board.find_king(c);
            generate_king_moves(board, sq, c, &mut moves, attacked);
        }
    }
    moves
}

fn compute_pins(board: &Board, c: Color) -> ArrayVec<(Sq64, u64), 8> {
    let sq = board.find_king(c);
    let mut pins = ArrayVec::new();
    //Rooks
    let bb = (board.get_piece_bitboard(c.flip(), Piece::Rook)
        | board.get_piece_bitboard(c.flip(), Piece::Queen))
        & get_rook_moves(sq, board.get_enemy_occupancy(c));
    for tsq in BitboardIter(bb) {
        let path = BETWEEN[sq.0 as usize][tsq.0 as usize];
        let path_blockers = path & board.get_friendly_occupancy(c);
        if path_blockers.count_ones() == 1 {
            // Found pin
            pins.push((
                Sq64(path_blockers.trailing_zeros() as u8),
                path | (1 << tsq.0),
            ));
        }
    }

    //Bishops
    let bb = (board.get_piece_bitboard(c.flip(), Piece::Bishop)
        | board.get_piece_bitboard(c.flip(), Piece::Queen))
        & get_bishop_moves(sq, board.get_enemy_occupancy(c));
    for tsq in BitboardIter(bb) {
        let path = BETWEEN[sq.0 as usize][tsq.0 as usize];
        let path_blockers = path & board.get_friendly_occupancy(c);
        if path_blockers.count_ones() == 1 {
            // Found pin
            pins.push((
                Sq64(path_blockers.trailing_zeros() as u8),
                path | (1 << tsq.0),
            ));
        }
    }

    pins
}

fn compute_checkers(board: &Board, c: Color) -> u64 {
    let mut bb = 0u64;
    let sq = board.find_king(c);
    let co = c.flip();
    bb |= PAWN_ATTACKS[c as usize][sq.0 as usize] & board.get_piece_bitboard(co, Piece::Pawn);
    bb |= KNIGHT_ATTACKS[sq.0 as usize] & board.get_piece_bitboard(co, Piece::Knight);
    bb |= get_bishop_moves(sq, board.get_occupany())
        & (board.get_piece_bitboard(co, Piece::Bishop)
            | board.get_piece_bitboard(co, Piece::Queen));
    bb |= get_rook_moves(sq, board.get_occupany())
        & (board.get_piece_bitboard(co, Piece::Rook) | board.get_piece_bitboard(co, Piece::Queen));
    bb
}

fn compute_attacked(board: &Board, c: Color) -> u64 {
    let mut bb = 0;
    for sq in BitboardIter(board.get_enemy_occupancy(c)) {
        let sq_ind = sq.0 as usize;
        let bb_no_king = board.get_occupany() ^ board.get_piece_bitboard(c, Piece::King);
        bb |= match board.get_piece_at(sq) {
            Piece::Pawn => PAWN_ATTACKS[c.flip() as usize][sq_ind],
            Piece::Knight => KNIGHT_ATTACKS[sq_ind],
            Piece::Bishop => get_bishop_moves(sq, bb_no_king),
            Piece::Rook => get_rook_moves(sq, bb_no_king),
            Piece::Queen => get_bishop_moves(sq, bb_no_king) | get_rook_moves(sq, bb_no_king),
            Piece::King => KING_ATTACKS[sq_ind],
        }
    }
    bb
}

fn generate_knight_moves(
    board: &Board,
    from_sq: Sq64,
    color: Color,
    moves: &mut ArrayVec<Move, 256>,
    valid_destinations: u64,
) {
    let bb = KNIGHT_ATTACKS[from_sq.0 as usize] & valid_destinations;
    generate_moves_bb(board, from_sq, bb, color, moves);
}

fn generate_king_moves(
    board: &Board,
    from_sq: Sq64,
    color: Color,
    moves: &mut ArrayVec<Move, 256>,
    attacked: u64,
) {
    let bb = KING_ATTACKS[from_sq.0 as usize] & !attacked;
    generate_moves_bb(board, from_sq, bb, color, moves);
}

pub fn generate_sliding_moves(
    board: &Board,
    sq: Sq64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    p: Piece,
    valid_destinations: u64,
) {
    let bb = match p {
        Piece::Rook => get_rook_moves(sq, board.get_occupany()),
        Piece::Bishop => get_bishop_moves(sq, board.get_occupany()),
        _ => unreachable!(),
    };
    generate_moves_bb(board, sq, bb & valid_destinations, c, moves);
}

pub fn generate_moves_bb(
    board: &Board,
    from_sq: Sq64,
    bb: u64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
) {
    let pbb = bb & !board.get_friendly_occupancy(c);
    let qbb = pbb & !board.get_enemy_occupancy(c);
    let cbb = pbb & board.get_enemy_occupancy(c);
    for to_sq in BitboardIter(qbb) {
        moves.push(Move::new(from_sq, to_sq));
    }
    for to_sq in BitboardIter(cbb) {
        moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::Capture));
    }
}

pub fn generate_pawn_moves(
    board: &Board,
    from_square: Sq64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    state_info: &StateInfo,
    valid_destinations: u64,
) {
    //Pushing
    let target_square = Sq64(match c {
        Color::White => from_square.0 + 8,
        Color::Black => from_square.0 - 8,
    });
    if !board.is_occupied(target_square) {
        if valid_destinations >> target_square.0 & 1 == 1 {
            if target_square.0 >> 3 == 7 || target_square.0 >> 3 == 0 {
                // Promotion
                for piece in Piece::PROMOTABLE {
                    moves.push(Move::new_flags(
                        from_square,
                        target_square,
                        MoveFlag::new_promotion(piece),
                    ));
                }
            } else {
                // Normal Pawn Push
                moves.push(Move::new(from_square, target_square));
            }
        }
        // Double Pawn Push
        if (from_square.0 >> 3 == 1 && c == Color::White)
            || (from_square.0 >> 3 == 6 && c == Color::Black)
        {
            let target_square = Sq64(match c {
                Color::White => from_square.0 + 16,
                Color::Black => from_square.0 - 16,
            });
            if valid_destinations >> target_square.0 & 1 == 1 {
                if !board.is_occupied(target_square) {
                    moves.push(Move::new_flags(
                        from_square,
                        target_square,
                        MoveFlag::DoublePawnPush,
                    ));
                }
            }
        }
    }
    // Taking
    let pa_bb = PAWN_ATTACKS[c as usize][from_square.0 as usize];
    for to_sq in BitboardIter(pa_bb & board.get_enemy_occupancy(c) & valid_destinations) {
        if to_sq.0 >> 3 == 7 || to_sq.0 >> 3 == 0 {
            //Promotion Capture
            for piece in Piece::PROMOTABLE {
                moves.push(Move::new_flags(
                    from_square,
                    to_sq,
                    MoveFlag::new_promotion_capture(piece),
                ));
            }
        } else {
            //Capture
            moves.push(Move::new_flags(from_square, to_sq, MoveFlag::Capture));
        }
    }
    // EP
    if let Some(ep_sq) = state_info.ep_square {
        let pawn_sq = match c {
            Color::White => ep_sq.0 - 8,
            Color::Black => ep_sq.0 + 8,
        };
        let bb = pa_bb & (0b1 << ep_sq.0);
        if bb & valid_destinations != 0 {
            // Check for double pin EdgeCase
            let king_sq = board.find_king(c);
            //Prefilter if king is in smae row
            if king_sq.0 & 0x38 == from_square.0 & 0x38 {
                let mask = 1 << from_square.0 | 1 << pawn_sq;
                let occupancy = board.get_occupany() & !mask;

                if get_rook_moves(king_sq, occupancy)
                    & (board.get_piece_bitboard(c.flip(), Piece::Rook)
                        | board.get_piece_bitboard(c.flip(), Piece::Queen))
                    == 0
                {
                    moves.push(Move::new_flags(from_square, ep_sq, MoveFlag::EnPassant));
                }
            } else {
                moves.push(Move::new_flags(from_square, ep_sq, MoveFlag::EnPassant));
            }
        }
        // Edgecase if taken pawn is putting king in check
        else if bb != 0 && (0b1 << pawn_sq) & valid_destinations != 0 {
            moves.push(Move::new_flags(from_square, ep_sq, MoveFlag::EnPassant));
        }
    }
}

pub fn generate_castles(
    board: &Board,
    from_sq: Sq64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    state_info: &StateInfo,
    attacked: u64,
) {
    //Check Short Castle
    if state_info.has_castle_rights(c, true) {
        let path_unoccupied = (board.get_occupany() & 0b110 << from_sq.0) == 0; // Pieces Between
        let path_safe = (attacked & 0b111 << from_sq.0) == 0; // King aswell

        if path_unoccupied && path_safe {
            let to_sq = Sq64(from_sq.0 + 2);
            moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::CastleKingside));
        }
    }

    //Check Long Castle
    if state_info.has_castle_rights(c, false) {
        let path_unoccupied = (board.get_occupany() & 0b111 << from_sq.0 - 3) == 0;
        let path_safe = (attacked & 0b111 << from_sq.0 - 2) == 0; // King aswell

        if path_unoccupied && path_safe {
            let to_sq = Sq64(from_sq.0 - 2);
            moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::CastleQueenside));
        }
    }
}

pub struct BitboardIter(pub u64);
impl Iterator for BitboardIter {
    type Item = Sq64;
    #[inline(always)]
    fn next(&mut self) -> Option<Sq64> {
        if self.0 == 0 {
            return None;
        }
        let sq = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        Some(Sq64(sq))
    }
}

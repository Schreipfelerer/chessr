use crate::{
    board::{Board, BoardState, Color, Move, Piece, Sq64, StateInfo, MoveFlag},
    movegen::consts::{
        BETWEEN, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, get_bishop_moves, get_rook_moves,
    },
};
use arrayvec::ArrayVec;

pub fn generate_moves(board_state: &BoardState) -> ArrayVec<Move, 256> {
    let board = &board_state.board;
    let state_info = &board_state.state_info;
    let c = state_info.active_color();
    let mut moves = ArrayVec::new();

    let checkers: u64 = compute_checkers(board, c);

    match checkers.count_ones() {
        0 => {
            let bb = !0u64;
            generate_all(board, state_info, c, &mut moves, bb, true);
        }
        1 => {
            // Only in between or attacker or king moves
            let king_sq = board.find_king(c);
            let attacker_sq = checkers.trailing_zeros();
            let bb = BETWEEN[king_sq.ind()][attacker_sq as usize] | checkers;
            generate_all(board, state_info, c, &mut moves, bb, false);
        }
        _ => {
            // Only King Movements
            let sq = board.find_king(c);
            generate_king_moves(board, sq, c, &mut moves);
        }
    }
    moves
}

fn generate_all(
    board: &Board,
    state_info: &StateInfo,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    valid_bb: u64,
    allow_castle: bool,
) {
    let (pin_bb, pin_bbs) = compute_pins(board, c);

    let unpinned_pawns = board.get_piece_bitboard(c, Piece::Pawn) & !pin_bb;
    batch_generate_pawn_moves(board, unpinned_pawns, c, moves, state_info, valid_bb);

    for from_sq in BitboardIter(board.occ_friendly(c) & !unpinned_pawns) {
        let pbb = valid_bb & pin_bbs[from_sq.ind()];
        match board.get_piece_at(from_sq) {
            Piece::Pawn => generate_pawn_moves(board, from_sq, c, moves, state_info, pbb),
            Piece::Knight => generate_knight_moves(board, from_sq, c, moves, pbb),
            Piece::Bishop => generate_sliding_moves(board, from_sq, c, moves, Piece::Bishop, pbb),
            Piece::Rook => generate_sliding_moves(board, from_sq, c, moves, Piece::Rook, pbb),
            Piece::Queen => {
                generate_sliding_moves(board, from_sq, c, moves, Piece::Bishop, pbb);
                generate_sliding_moves(board, from_sq, c, moves, Piece::Rook, pbb);
            }
            Piece::King => {
                generate_king_moves(board, from_sq, c, moves);
                if allow_castle {
                    generate_castles(board, from_sq, c, moves, state_info);
                }
            }
        }
    }
}

fn compute_pins(board: &Board, c: Color) -> (u64, [u64; 64]) {
    let mut pins = [u64::MAX; 64];
    let mut pbb = 0;
    let sq = board.find_king(c);
    //Rooks
    let bb = (board.get_piece_bitboard(c.flip(), Piece::Rook)
        | board.get_piece_bitboard(c.flip(), Piece::Queen))
        & get_rook_moves(sq, board.occ_enemy(c));
    push_pins(board, c, sq, &mut pins, &mut pbb, bb);
    //Bishops
    let bb = (board.get_piece_bitboard(c.flip(), Piece::Bishop)
        | board.get_piece_bitboard(c.flip(), Piece::Queen))
        & get_bishop_moves(sq, board.occ_enemy(c));
    push_pins(board, c, sq, &mut pins, &mut pbb, bb);
    (pbb, pins)
}

fn push_pins(board: &Board, c: Color, sq: Sq64, pins: &mut [u64; 64], pbb: &mut u64, bb: u64) {
    for tsq in BitboardIter(bb) {
        let path = BETWEEN[sq.ind()][tsq.ind()];
        let path_blockers = path & board.occ_friendly(c);
        if path_blockers.count_ones() == 1 {
            // Found pin
            *pbb |= path_blockers;
            pins[path_blockers.trailing_zeros() as usize] = path | tsq.mask();
        }
    }
}

fn compute_checkers(board: &Board, c: Color) -> u64 {
    let mut bb = 0u64;
    let sq = board.find_king(c);
    let co = c.flip();
    bb |= PAWN_ATTACKS[c as usize][sq.ind()] & board.get_piece_bitboard(co, Piece::Pawn);
    bb |= KNIGHT_ATTACKS[sq.ind()] & board.get_piece_bitboard(co, Piece::Knight);
    bb |= get_bishop_moves(sq, board.occ())
        & (board.get_piece_bitboard(co, Piece::Bishop)
            | board.get_piece_bitboard(co, Piece::Queen));
    bb |= get_rook_moves(sq, board.occ())
        & (board.get_piece_bitboard(co, Piece::Rook) | board.get_piece_bitboard(co, Piece::Queen));
    bb
}

fn generate_knight_moves(
    board: &Board,
    from_sq: Sq64,
    color: Color,
    moves: &mut ArrayVec<Move, 256>,
    valid_destinations: u64,
) {
    let bb = KNIGHT_ATTACKS[from_sq.ind()] & valid_destinations;
    generate_moves_bb(board, from_sq, bb, color, moves);
}

fn generate_king_moves(
    board: &Board,
    from_sq: Sq64,
    color: Color,
    moves: &mut ArrayVec<Move, 256>,
) {
    let bb = KING_ATTACKS[from_sq.ind()] & !board.occ_friendly(color);
    if bb != 0 {
        let occ_no_king = board.occ() & !board.get_piece_bitboard(color, Piece::King);
        let cbb = bb & board.occ_enemy(color);
        let qbb = bb & !board.occ_enemy(color);
        for to_sq in BitboardIter(qbb) {
            if !is_attacked(board, to_sq, color.flip(), occ_no_king) {
                moves.push(Move::new(from_sq, to_sq));
            }
        }
        for to_sq in BitboardIter(cbb) {
            if !is_attacked(board, to_sq, color.flip(), occ_no_king) {
                moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::Capture));
            }
        }
    }
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
        Piece::Rook => get_rook_moves(sq, board.occ()),
        Piece::Bishop => get_bishop_moves(sq, board.occ()),
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
    let pbb = bb & !board.occ_friendly(c);
    let qbb = pbb & !board.occ_enemy(c);
    let cbb = pbb & board.occ_enemy(c);
    for to_sq in BitboardIter(qbb) {
        moves.push(Move::new(from_sq, to_sq));
    }
    for to_sq in BitboardIter(cbb) {
        moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::Capture));
    }
}

pub fn batch_generate_pawn_moves(
    board: &Board,
    pawn_bb: u64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    state_info: &StateInfo,
    valid_destinations: u64,
) {
    if pawn_bb.count_ones() <= 1 {
        for pawn_sq in BitboardIter(pawn_bb) {
            generate_pawn_moves(board, pawn_sq, c, moves, state_info, valid_destinations);
        }
        return;
    }
    const BASE_RANKS: u64 = 0xFF00_0000_0000_00FF;
    const FILE_A: u64 = 0x0101010101010101;
    const FILE_H: u64 = 0x8080808080808080;

    let empty = !board.occ();
    let empty_valid = !board.occ() & valid_destinations;
    let occ_enemy_valid = board.occ_enemy(c) & valid_destinations;
    let rotate_shift = match c {
        Color::White => 8,
        Color::Black => 56,
    };
    let push_delta: i8 = match c {
        Color::White => 8,
        Color::Black => -8,
    };
    let push_bb = pawn_bb & empty_valid.rotate_right(rotate_shift);
    let doublepush_bb = pawn_bb
        & BASE_RANKS.rotate_left(rotate_shift)
        & empty.rotate_right(rotate_shift)
        & empty_valid.rotate_right(rotate_shift * 2);
    let cap_left = pawn_bb & !FILE_A & occ_enemy_valid.rotate_right(rotate_shift - 1);
    let cap_right = pawn_bb & !FILE_H & occ_enemy_valid.rotate_right(rotate_shift + 1);

    let prom_rank = BASE_RANKS.rotate_right(rotate_shift);
    let n_prom_rank = !prom_rank;

    for push_sq in BitboardIter(push_bb & n_prom_rank) {
        moves.push(Move::new(push_sq, push_sq + push_delta));
    }
    for double_push_sq in BitboardIter(doublepush_bb & BASE_RANKS.rotate_left(rotate_shift)) {
        moves.push(Move::new_flags(
            double_push_sq,
            double_push_sq + (push_delta * 2),
            MoveFlag::DoublePawnPush,
        ));
    }
    for promo_sq in BitboardIter(push_bb & prom_rank) {
        for p in Piece::PROMOTABLE {
            moves.push(Move::new_flags(
                promo_sq,
                promo_sq + push_delta,
                MoveFlag::new_promotion(p),
            ));
        }
    }
    for cap_left_sq in BitboardIter(cap_left & n_prom_rank) {
        moves.push(Move::new_flags(
            cap_left_sq,
            cap_left_sq + (push_delta - 1),
            MoveFlag::Capture,
        ));
    }
    for promo_cap_left_sq in BitboardIter(cap_left & prom_rank) {
        for p in Piece::PROMOTABLE {
            moves.push(Move::new_flags(
                promo_cap_left_sq,
                promo_cap_left_sq + (push_delta - 1),
                MoveFlag::new_promotion_capture(p),
            ));
        }
    }
    for cap_right_sq in BitboardIter(cap_right & n_prom_rank) {
        moves.push(Move::new_flags(
            cap_right_sq,
            cap_right_sq + (push_delta + 1),
            MoveFlag::Capture,
        ));
    }
    for promo_cap_right_sq in BitboardIter(cap_right & prom_rank) {
        for p in Piece::PROMOTABLE {
            moves.push(Move::new_flags(
                promo_cap_right_sq,
                promo_cap_right_sq + (push_delta + 1),
                MoveFlag::new_promotion_capture(p),
            ));
        }
    }
    if let Some(ep_sq) = state_info.ep_square {
        if (ep_sq - (push_delta + 1)).is_on_bb(pawn_bb & !FILE_H) {
            generate_ep_moves(
                board,
                ep_sq - (push_delta + 1),
                c,
                moves,
                state_info,
                valid_destinations,
                ep_sq.mask(),
            );
        }
        if (ep_sq - (push_delta - 1)).is_on_bb(pawn_bb & !FILE_A) {
            generate_ep_moves(
                board,
                ep_sq - (push_delta - 1),
                c,
                moves,
                state_info,
                valid_destinations,
                ep_sq.mask(),
            );
        }
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
    let target_square = match c {
        Color::White => from_square + 8,
        Color::Black => from_square - 8,
    };
    if !board.is_occupied(target_square) {
        if target_square.is_on_bb(valid_destinations) {
            if target_square.rank() == 7 || target_square.rank() == 0 {
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
        if (from_square.rank() == 1 && c == Color::White)
            || (from_square.rank() == 6 && c == Color::Black)
        {
            let target_square = match c {
                Color::White => from_square + 16,
                Color::Black => from_square - 16,
            };
            if target_square.is_on_bb(valid_destinations) {
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
    let pa_bb = PAWN_ATTACKS[c as usize][from_square.ind()];
    for to_sq in BitboardIter(pa_bb & board.occ_enemy(c) & valid_destinations) {
        if to_sq.rank() == 7 || to_sq.rank() == 0 {
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
    generate_ep_moves(
        board,
        from_square,
        c,
        moves,
        state_info,
        valid_destinations,
        pa_bb,
    );
}

fn generate_ep_moves(
    board: &Board,
    from_square: Sq64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    state_info: &StateInfo,
    valid_destinations: u64,
    pa_bb: u64,
) {
    // EP
    if let Some(ep_sq) = state_info.ep_square {
        let pawn_sq = match c {
            Color::White => ep_sq - 8,
            Color::Black => ep_sq + 8,
        };
        let bb = pa_bb & ep_sq.mask();
        if bb & valid_destinations != 0 {
            // Check for double pin EdgeCase
            let king_sq = board.find_king(c);
            //Prefilter if king is in same row
            if king_sq.rank() == from_square.rank() {
                let mask = from_square.mask() | pawn_sq.mask();
                let occupancy = board.occ() & !mask;

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
        // Edge-case if taken pawn is putting king in check
        else if bb != 0 && pawn_sq.mask() & valid_destinations != 0 {
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
) {
    //Check Short Castle
    if state_info.has_castle_rights(c, true) {
        let path_unoccupied = (board.occ() & 0b110 << from_sq.0) == 0; // Pieces Between

        if path_unoccupied {
            if !is_attacked(board, from_sq + 1, c.flip(), board.occ()) {
                if !is_attacked(board, from_sq + 2, c.flip(), board.occ()) {
                    let to_sq = from_sq + 2;
                    moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::CastleKingside));
                }
            }
        }
    }

    //Check Long Castle
    if state_info.has_castle_rights(c, false) {
        let path_unoccupied = (board.occ() & 0b111 << from_sq.0 - 3) == 0;
        if path_unoccupied {
            if !is_attacked(board, from_sq - 1, c.flip(), board.occ()) {
                if !is_attacked(board, from_sq - 2, c.flip(), board.occ()) {
                    let to_sq = from_sq - 2;
                    moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::CastleQueenside));
                }
            }
        }
    }
}

fn is_attacked(board: &Board, sq: Sq64, by_color: Color, occ_no_king: u64) -> bool {
    (KNIGHT_ATTACKS[sq.ind()] & board.get_piece_bitboard(by_color, Piece::Knight)) != 0
        || (KING_ATTACKS[sq.ind()] & board.get_piece_bitboard(by_color, Piece::King)) != 0
        || (PAWN_ATTACKS[by_color.flip() as usize][sq.ind()]
            & board.get_piece_bitboard(by_color, Piece::Pawn))
            != 0
        || (get_bishop_moves(sq, occ_no_king)
            & (board.get_piece_bitboard(by_color, Piece::Bishop)
                | board.get_piece_bitboard(by_color, Piece::Queen)))
            != 0
        || (get_rook_moves(sq, occ_no_king)
            & (board.get_piece_bitboard(by_color, Piece::Rook)
                | board.get_piece_bitboard(by_color, Piece::Queen)))
            != 0
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

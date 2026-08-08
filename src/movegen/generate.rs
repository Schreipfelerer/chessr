use crate::movegen::BitboardIter;
use crate::movegen::helpers::{compute_checkers, compute_pins, is_attacked};
use crate::{
    board::{Board, BoardState, Color, Move, MoveFlag, Piece, Sq64, StateInfo},
    movegen::r#const::{
        BETWEEN, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, get_bishop_moves, get_rook_moves,
    },
};
use arrayvec::ArrayVec;

#[must_use]
/// Generates all legal moves for the active side.
pub fn generate_moves(board_state: &BoardState, only_capture: bool) -> ArrayVec<Move, 256> {
    let board = &board_state.board;
    let state_info = &board_state.state_info;
    let c = state_info.active_color;

    let checkers: u64 = compute_checkers(board, c);

    match checkers.count_ones() {
        0 => {
            let bb = if only_capture {
                board.occ_enemy(c)
            } else {
                !0u64
            };
            generate_all(board, state_info, c, bb, true)
        }
        1 => {
            // Only in between or attacker or king moves
            let king_sq = board.find_king(c);
            let attacker_sq = checkers.trailing_zeros();
            let bb = BETWEEN[king_sq.ind()][attacker_sq as usize] | checkers;
            generate_all(board, state_info, c, bb, false)
        }
        _ => {
            // Only King Movements
            let mut moves = ArrayVec::new();
            let sq = board.find_king(c);
            generate_king_moves(board, sq, c, &mut moves);
            moves
        }
    }
}

#[must_use]
/// See if king is in check
pub fn is_check(board_state: &BoardState) -> bool {
    let board = &board_state.board;
    let state_info = &board_state.state_info;
    let c = state_info.active_color;

    let checkers: u64 = compute_checkers(board, c);
    checkers != 0
}

#[must_use]
/// Generates legal moves for moves that land on a valid square.
fn generate_all(
    board: &Board,
    state_info: &StateInfo,
    c: Color,
    valid_targets: u64,
    is_check: bool,
) -> ArrayVec<Move, 256> {
    let mut moves = ArrayVec::new();
    let (pin_bb, pin_bbs) = compute_pins(board, c);

    let unpinned_pawns = board.get_piece_bitboard(c, Piece::Pawn) & !pin_bb;
    batch_generate_pawn_moves(
        board,
        unpinned_pawns,
        c,
        &mut moves,
        state_info,
        valid_targets,
    );

    for from_sq in BitboardIter(board.occ_friendly(c) & !unpinned_pawns) {
        let pbb = valid_targets & pin_bbs[from_sq.ind()];
        match board.get_piece_at(from_sq) {
            Piece::Pawn => generate_pawn_moves(board, from_sq, c, &mut moves, state_info, pbb),
            Piece::Knight => generate_knight_moves(board, from_sq, c, &mut moves, pbb),
            Piece::Bishop => {
                generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Bishop, pbb);
            }
            Piece::Rook => generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Rook, pbb),
            Piece::Queen => {
                generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Bishop, pbb);
                generate_sliding_moves(board, from_sq, c, &mut moves, Piece::Rook, pbb);
            }
            Piece::King => {
                generate_king_moves(board, from_sq, c, &mut moves);
                if is_check {
                    generate_castles(board, from_sq, c, &mut moves, state_info);
                }
            }
        }
    }
    moves
}

fn generate_knight_moves(
    board: &Board,
    from_sq: Sq64,
    color: Color,
    moves: &mut ArrayVec<Move, 256>,
    valid_destinations: u64,
) {
    let bb = KNIGHT_ATTACKS[from_sq.ind()] & valid_destinations;
    generate_moves_to_bb(board, from_sq, bb, color, moves);
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
            // Quiet
            if !is_attacked(board, to_sq, !color, occ_no_king) {
                moves.push(Move::new(from_sq, to_sq));
            }
        }
        for to_sq in BitboardIter(cbb) {
            // Capture
            if !is_attacked(board, to_sq, !color, occ_no_king) {
                moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::Capture));
            }
        }
    }
}

/// Generates queen, rook, or bishop moves from a square.
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
    generate_moves_to_bb(board, sq, bb & valid_destinations, c, moves);
}

/// Converts destination bitboards into quiet and capturing moves.
pub fn generate_moves_to_bb(
    board: &Board,
    from_sq: Sq64,
    destination_squares: u64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
) {
    let non_blocked_squared = destination_squares & !board.occ_friendly(c);
    for quiet_sq in BitboardIter(non_blocked_squared & !board.occ_enemy(c)) {
        moves.push(Move::new(from_sq, quiet_sq));
    }
    for captured_sq in BitboardIter(non_blocked_squared & board.occ_enemy(c)) {
        moves.push(Move::new_flags(from_sq, captured_sq, MoveFlag::Capture));
    }
}

/// Generates pawn moves in batches.
pub fn batch_generate_pawn_moves(
    board: &Board,
    pawn_bb: u64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    state_info: &StateInfo,
    valid_destinations: u64,
) {
    const BASE_RANKS: u64 = 0xFF00_0000_0000_00FF;
    const FILE_A: u64 = 0x0101_0101_0101_0101;
    const FILE_H: u64 = 0x8080_8080_8080_8080;

    if pawn_bb.count_ones() <= 1 {
        for pawn_sq in BitboardIter(pawn_bb) {
            generate_pawn_moves(board, pawn_sq, c, moves, state_info, valid_destinations);
        }
        return;
    }

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
            generate_ep_move(
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
            generate_ep_move(
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

/// Generates all legal moves for one pawn.
pub fn generate_pawn_moves(
    board: &Board,
    from_sq: Sq64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    state_info: &StateInfo,
    valid_destinations: u64,
) {
    //Pushing
    let target_square = match c {
        Color::White => from_sq + 8,
        Color::Black => from_sq - 8,
    };
    if !board.is_occupied(target_square) {
        if target_square.is_on_bb(valid_destinations) {
            if target_square.rank() == 7 || target_square.rank() == 0 {
                // Promotion
                for piece in Piece::PROMOTABLE {
                    moves.push(Move::new_flags(
                        from_sq,
                        target_square,
                        MoveFlag::new_promotion(piece),
                    ));
                }
            } else {
                // Normal Pawn Push
                moves.push(Move::new(from_sq, target_square));
            }
        }
        // Double Pawn Push
        if (from_sq.rank() == 1 && c == Color::White) || (from_sq.rank() == 6 && c == Color::Black)
        {
            let target_square = match c {
                Color::White => from_sq + 16,
                Color::Black => from_sq - 16,
            };
            if target_square.is_on_bb(valid_destinations) && !board.is_occupied(target_square) {
                moves.push(Move::new_flags(
                    from_sq,
                    target_square,
                    MoveFlag::DoublePawnPush,
                ));
            }
        }
    }
    // Taking
    let pa_bb = PAWN_ATTACKS[c as usize][from_sq.ind()];
    for to_sq in BitboardIter(pa_bb & board.occ_enemy(c) & valid_destinations) {
        if to_sq.rank() == 7 || to_sq.rank() == 0 {
            //Promotion Capture
            for piece in Piece::PROMOTABLE {
                moves.push(Move::new_flags(
                    from_sq,
                    to_sq,
                    MoveFlag::new_promotion_capture(piece),
                ));
            }
        } else {
            //Capture
            moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::Capture));
        }
    }
    generate_ep_move(
        board,
        from_sq,
        c,
        moves,
        state_info,
        valid_destinations,
        pa_bb,
    );
}

/// Adds an en passant move when it is legal for the given pawn.
fn generate_ep_move(
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
        if ep_sq.is_on_bb(pa_bb) {
            if ep_sq.is_on_bb(valid_destinations) {
                // Check for double pin EdgeCase
                let king_sq = board.find_king(c);
                // Prefilter if king is in same row
                if king_sq.rank() == from_square.rank() {
                    let mask = from_square.mask() | pawn_sq.mask();
                    let occupancy = board.occ() & !mask;

                    if get_rook_moves(king_sq, occupancy)
                        & (board.get_piece_bitboard(!c, Piece::Rook)
                            | board.get_piece_bitboard(!c, Piece::Queen))
                        == 0
                    {
                        moves.push(Move::new_flags(from_square, ep_sq, MoveFlag::EnPassant));
                    }
                } else {
                    moves.push(Move::new_flags(from_square, ep_sq, MoveFlag::EnPassant));
                }
            }
            // Edge-case if taken pawn is putting king in check
            else if pawn_sq.is_on_bb(valid_destinations) {
                moves.push(Move::new_flags(from_square, ep_sq, MoveFlag::EnPassant));
            }
        }
    }
}

/// Generates legal kingside and queenside castling moves.
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

        if path_unoccupied
            && !is_attacked(board, from_sq + 1, !c, board.occ())
            && !is_attacked(board, from_sq + 2, !c, board.occ())
        {
            let to_sq = from_sq + 2;
            moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::CastleKingside));
        }
    }

    //Check Long Castle
    if state_info.has_castle_rights(c, false) {
        let path_unoccupied = (board.occ() & 0b111 << (from_sq.0 - 3)) == 0;
        if path_unoccupied
            && !is_attacked(board, from_sq - 1, !c, board.occ())
            && !is_attacked(board, from_sq - 2, !c, board.occ())
        {
            let to_sq = from_sq - 2;
            moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::CastleQueenside));
        }
    }
}

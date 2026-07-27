use crate::{
    board::{Board, BoardState, Color, Move, MoveFlag, Piece, Sq64, StateInfo},
    movegen::magic::{get_bishop_moves, get_rook_moves},
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
    let color = state_info.active_color();
    let mut moves = ArrayVec::new();

    for piece in Piece::ALL {
        for from_sq in BitboardIter(board.get_piece_bitboard(color, piece)) {
            match piece {
                Piece::Pawn => generate_pawn_moves(board, from_sq, color, &mut moves, state_info),
                Piece::Knight => {
                    generate_direct_moves(board, from_sq, KNIGHT_ATTACKS, color, &mut moves)
                }
                Piece::Bishop => {
                    generate_sliding_moves(board, from_sq, color, &mut moves, Piece::Bishop)
                }
                Piece::Rook => {
                    generate_sliding_moves(board, from_sq, color, &mut moves, Piece::Rook)
                }
                Piece::Queen => {
                    generate_sliding_moves(board, from_sq, color, &mut moves, Piece::Bishop);
                    generate_sliding_moves(board, from_sq, color, &mut moves, Piece::Rook);
                }
                Piece::King => {
                    generate_direct_moves(board, from_sq, KING_ATTACKS, color, &mut moves);
                    generate_castles(board, from_sq, color, &mut moves, state_info);
                }
            }
        }
    }

    moves
}

pub fn generate_sliding_moves(
    board: &Board,
    sq: Sq64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    p: Piece,
) {
    let bb = match p {
        Piece::Rook => get_rook_moves(sq, board.get_occupany()),
        Piece::Bishop => get_bishop_moves(sq, board.get_occupany()),
        _ => unreachable!(),
    };
    let captures = bb & board.get_enemy_occupancy(c);
    for tsq in BitboardIter(captures) {
        moves.push(Move::new_flags(sq, tsq, MoveFlag::Capture));
    }

    let quiets = bb & !board.get_occupany();
    for tsq in BitboardIter(quiets) {
        moves.push(Move::new(sq, tsq));
    }
}

pub fn generate_direct_moves(
    board: &Board,
    from_sq: Sq64,
    attacks: [u64; 64],
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
) {
    let bb = attacks[from_sq.0 as usize] & !board.get_friendly_occupancy(c);

    for to_sq in BitboardIter(bb) {
        if board.is_occupied_enemy(to_sq, c) {
            moves.push(Move::new_flags(from_sq, to_sq, MoveFlag::Capture));
        } else {
            moves.push(Move::new(from_sq, to_sq));
        }
    }
}

pub fn generate_pawn_moves(
    board: &Board,
    from_square: Sq64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    state_info: &StateInfo,
) {
    //Pushing
    let target_square = Sq64(match c {
        Color::White => from_square.0 + 8,
        Color::Black => from_square.0 - 8,
    });
    if !board.is_occupied(target_square) {
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
            // Double Pawn Push
            if from_square.0 >> 3 == 1 || from_square.0 >> 3 == 6 {
                let target_square = Sq64(match c {
                    Color::White => from_square.0 + 16,
                    Color::Black => from_square.0 - 16,
                });
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
    for to_sq in BitboardIter(pa_bb & board.get_enemy_occupancy(c)) {
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
        if pa_bb & (1 << ep_sq.0) != 0 {
            moves.push(Move::new_flags(from_square, ep_sq, MoveFlag::EnPassant));
        }
    }
}

pub fn generate_castles(
    board: &Board,
    from_square: Sq64,
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
    state_info: &StateInfo,
) {
    //Check Short Castle
    if state_info.has_castle_rights(c, true) {
        let f_sq = Sq64(from_square.0 + 1);
        let g_sq = Sq64(from_square.0 + 2);

        let path_unoccupied = !board.is_occupied(f_sq) && !board.is_occupied(g_sq);
        let path_safe = !is_square_attacked_by(from_square, c.flip(), board)
            && !is_square_attacked_by(f_sq, c.flip(), board);

        if path_unoccupied && path_safe {
            moves.push(Move::new_flags(from_square, g_sq, MoveFlag::CastleKingside));
        }
    }

    //Check Long Castle
    if state_info.has_castle_rights(c, false) {
        let d_sq = Sq64(from_square.0 - 1);
        let c_sq = Sq64(from_square.0 - 2);
        let b_sq = Sq64(from_square.0 - 3);

        let path_unoccupied =
            !board.is_occupied(d_sq) && !board.is_occupied(c_sq) && !board.is_occupied(b_sq);
        let path_safe = !is_square_attacked_by(from_square, c.flip(), board)
            && !is_square_attacked_by(d_sq, c.flip(), board);

        if path_unoccupied && path_safe {
            moves.push(Move::new_flags(
                from_square,
                c_sq,
                MoveFlag::CastleQueenside,
            ));
        }
    }
}

pub fn is_square_attacked_by(sq64: Sq64, c: Color, board: &Board) -> bool {
    let sq = sq64.0 as usize;
    if PAWN_ATTACKS[c.flip() as usize][sq] & board.get_piece_bitboard(c, Piece::Pawn) != 0 {
        return true;
    }
    if KNIGHT_ATTACKS[sq] & board.get_piece_bitboard(c, Piece::Knight) != 0 {
        return true;
    }
    if KING_ATTACKS[sq] & board.get_piece_bitboard(c, Piece::King) != 0 {
        return true;
    }

    if get_bishop_moves(sq64, board.get_occupany())
        & (board.get_piece_bitboard(c, Piece::Bishop) | board.get_piece_bitboard(c, Piece::Queen))
        != 0
    {
        return true;
    }
    if get_rook_moves(sq64, board.get_occupany())
        & (board.get_piece_bitboard(c, Piece::Rook) | board.get_piece_bitboard(c, Piece::Queen))
        != 0
    {
        return true;
    }
    false
}

struct BitboardIter(u64);
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



#[cfg(test)]
mod perft_tests {
    use crate::{board::BoardState, movegen::perft::number_of_moves};

    #[test]
    fn start_pos() {
        let mut board =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        assert_eq!(number_of_moves(&mut board, 4), 197_281);
    }

    #[test]
    fn pos_wiki() {
        let pos2 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos2).unwrap(), 3),
            97_862
        );
        let pos3 = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos3).unwrap(), 4),
            43_238
        );
        let pos4 = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos4).unwrap(), 3),
            9_467
        );
        let pos5 = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos5).unwrap(), 3),
            62_379
        );
        let pos6 = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos6).unwrap(), 3),
            89_890
        );
    }

    #[test]
    fn en_passant_discovered_check() {
        let fen = "7k/8/8/K1pP3r/8/8/8/8 w - c6 0 1";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(fen).unwrap(), 5),
            44_848
        );
    }

    #[test]
    fn complicated_midgame() {
        let fen = "r3k2r/1bp2pP1/5n2/1P1Q4/1pPq4/5N2/1B1P2p1/R3K2R b KQkq c3 0 1";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(fen).unwrap(), 3),
            113_742
        );
    }
}

use std::time::Instant;

use crate::board::{Board, BoardState, Color, Move, Piece, Sq64, Sq88, StateInfo};
use arrayvec::ArrayVec;

// Direction offsets for sliding pieces
pub const ROOK_OFFSETS: [i8; 4] = [1, -1, 16, -16];
pub const BISHOP_OFFSETS: [i8; 4] = [15, 17, -15, -17];
pub const QUEEN_OFFSETS: [i8; 8] = [1, -1, 16, -16, 15, 17, -15, -17];
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
        let mut bb = board.get_piece_bitboard(color, piece);
        while bb != 0 {
            let sq = bb.trailing_zeros() as u8;
            bb &= bb - 1; // clear lsb
            let from_sq = Sq64(sq);
            let from_square_0x88 = from_sq.to_sq88();
            // dispatch to the right generator
            match piece {
                Piece::Pawn => generate_pawn_moves(
                    board,
                    from_sq,
                    color,
                    &mut moves,
                    state_info,
                ),
                Piece::Knight => {
                    generate_direct_moves(board, from_sq, KNIGHT_ATTACKS, color, &mut moves)
                }
                Piece::Bishop => generate_sliding_moves(
                    board,
                    from_square_0x88,
                    &BISHOP_OFFSETS,
                    color,
                    &mut moves,
                ),
                Piece::Rook => generate_sliding_moves(
                    board,
                    from_square_0x88,
                    &ROOK_OFFSETS,
                    color,
                    &mut moves,
                ),
                Piece::Queen => generate_sliding_moves(
                    board,
                    from_square_0x88,
                    &QUEEN_OFFSETS,
                    color,
                    &mut moves,
                ),
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
    from_sq: Sq88,
    offsets: &[i8],
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
) {
    for &offset in offsets {
        let mut to_sq = from_sq.step(offset);

        while to_sq.is_on_board() {
            let target_square = to_sq.to_sq64();
            if board.is_occupied(target_square) {
                if !board.is_occupied_firendly(target_square, c) {
                    moves.push(Move::new_flags(from_sq.to_sq64(), target_square, 0x4));
                }
                break;
            }
            moves.push(Move::new(from_sq.to_sq64(), target_square));

            to_sq = to_sq.step(offset)
        }
    }
}

pub fn generate_direct_moves(
    board: &Board,
    from_sq: Sq64,
    attacks: [u64; 64],
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
) {
    let mut bb = attacks[from_sq.0 as usize] & !board.get_friendly_occupancy(c);

    while bb != 0 {
        let sq = bb.trailing_zeros() as u8;
        bb &= bb - 1; // clear lsb
        let to_sq = Sq64(sq);
        if board.is_occupied_enemy(to_sq, c) {
            moves.push(Move::new_flags(from_sq, to_sq, 0x4));
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
                    8 + piece as u8 - 1,
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
                    moves.push(Move::new_flags(from_square, target_square, 1));
                }
            }
        }
    }
    // Taking
    let pa_bb = PAWN_ATTACKS[c as usize][from_square.0 as usize];
    let mut bb = pa_bb & board.get_enemy_occupancy(c);
    while bb != 0 {
        let sq = bb.trailing_zeros() as u8;
        bb &= bb - 1; // clear lsb
        let to_sq = Sq64(sq);
        //Can Take
        if to_sq.0 >> 3 == 7 || to_sq.0 >> 3 == 0 {
            //Promotion Capture
            for piece in Piece::PROMOTABLE {
                moves.push(Move::new_flags(from_square, to_sq, 0xC + piece as u8 - 1));
            }
        } else {
            //Capture
            moves.push(Move::new_flags(from_square, to_sq, 0x4));
        }
    }
    // EP
    if let Some(ep_sq) = state_info.ep_square {
        if pa_bb & (1 << ep_sq.0) != 0 {
            moves.push(Move::new_flags(from_square, ep_sq, 0x5));
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
        let path_safe = !is_square_attacked_by(from_square.to_sq88(), c.flip(), board)
            && !is_square_attacked_by(f_sq.to_sq88(), c.flip(), board);

        if path_unoccupied && path_safe {
            moves.push(Move::new_flags(from_square, g_sq, 0x2));
        }
    }

    //Check Long Castle
    if state_info.has_castle_rights(c, false) {
        let d_sq = Sq64(from_square.0 - 1);
        let c_sq = Sq64(from_square.0 - 2);
        let b_sq = Sq64(from_square.0 - 3);

        let path_unoccupied =
            !board.is_occupied(d_sq) && !board.is_occupied(c_sq) && !board.is_occupied(b_sq);
        let path_safe = !is_square_attacked_by(from_square.to_sq88(), c.flip(), board)
            && !is_square_attacked_by(d_sq.to_sq88(), c.flip(), board);

        if path_unoccupied && path_safe {
            moves.push(Move::new_flags(from_square, c_sq, 0x3));
        }
    }
}

pub fn is_square_attacked_by(sq_0x88: Sq88, c: Color, board: &Board) -> bool {
    let sq = sq_0x88.to_sq64().0 as usize;
    if PAWN_ATTACKS[c.flip() as usize][sq] & board.get_piece_bitboard(c, Piece::Pawn) != 0 {
        return true;
    }
    if KNIGHT_ATTACKS[sq] & board.get_piece_bitboard(c, Piece::Knight) != 0 {
        return true;
    }
    if KING_ATTACKS[sq] & board.get_piece_bitboard(c, Piece::King) != 0 {
        return true;
    }
    for bo in BISHOP_OFFSETS {
        let mut nsq = sq_0x88.step(bo);
        while nsq.is_on_board() {
            if board.is_piece(nsq.to_sq64(), c, Piece::Bishop)
                || board.is_piece(nsq.to_sq64(), c, Piece::Queen)
            {
                return true;
            }
            if board.is_occupied(nsq.to_sq64()) {
                break;
            }
            nsq = nsq.step(bo);
        }
    }
    for ro in ROOK_OFFSETS {
        let mut nsq = sq_0x88.step(ro);
        while nsq.is_on_board() {
            if board.is_piece(nsq.to_sq64(), c, Piece::Rook)
                || board.is_piece(nsq.to_sq64(), c, Piece::Queen)
            {
                return true;
            }
            if board.is_occupied(nsq.to_sq64()) {
                break;
            }
            nsq = nsq.step(ro);
        }
    }
    false
}

pub fn number_of_moves(board_state: &mut BoardState, depth: u8) -> u32 {
    if depth == 0 {
        return 1;
    }
    let mut move_nunmber = 0;
    let color = board_state.state_info.active_color();
    let other_color = color.flip();
    let moves = generate_moves(board_state);
    for m in moves {
        let undo = board_state.make_move(m);
        if !is_square_attacked_by(
            board_state.board.find_king(color).to_sq88(),
            other_color,
            &board_state.board,
        ) {
            move_nunmber += number_of_moves(board_state, depth - 1);
        }
        board_state.undo_move(undo);
    }
    move_nunmber
}

pub fn perft(board_state: &mut BoardState, max_depth: u8) {
    for depth in 1..=max_depth {
        let start = Instant::now();
        let moves = number_of_moves(board_state, depth);
        let duration = start.elapsed();

        println!(
            "perft with depth {}: {:?}, moves: {}",
            depth, duration, moves
        );
    }
}

pub fn perft_devide(board_state: &mut BoardState, depth: u8) {
    let mut total = 0;
    for m in generate_moves(board_state) {
        let undo = board_state.make_move(m);
        if !is_square_attacked_by(
            board_state
                .board
                .find_king(board_state.state_info.active_color().flip())
                .to_sq88(),
            board_state.state_info.active_color(),
            &board_state.board,
        ) {
            let moves = number_of_moves(board_state, depth - 1);
            println!("  {}: {} moves", m, moves);
            total += moves;
        }
        board_state.undo_move(undo);
    }
    println!("Total moves: {}", total)
}

#[cfg(test)]
mod perft_tests {
    use crate::{board::BoardState, movegen::number_of_moves};

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

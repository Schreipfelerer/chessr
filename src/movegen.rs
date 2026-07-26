use std::time::Instant;

use crate::board::{Board, BoardState, Color, Move, Piece, Sq64, Sq88, StateInfo};
use arrayvec::ArrayVec;

// Direction offsets for sliding pieces
pub const ROOK_OFFSETS: [i8; 4] = [1, -1, 16, -16];
pub const BISHOP_OFFSETS: [i8; 4] = [15, 17, -15, -17];
pub const QUEEN_OFFSETS: [i8; 8] = [1, -1, 16, -16, 15, 17, -15, -17];
pub const KING_OFFSETS: [i8; 8] = [1, -1, 16, -16, 15, 17, -15, -17];
pub const KNIGHT_OFFSETS: [i8; 8] = [31, 33, 18, 14, -31, -33, -18, -14];

pub fn generate_moves(board_state: &BoardState) -> ArrayVec<Move, 256> {
    let board = &board_state.board;
    let state_info = &board_state.state_info;
    let color = state_info.active_color();
    let mut moves = ArrayVec::new();

    for sq in 0..64 {
        let from_sq = Sq64(sq);
        if !board.is_occupied_firendly(from_sq, color) {
            continue;
        }
        let from_square_0x88 = from_sq.to_sq88();
        for piece in Piece::ALL {
            if !board.is_piece(from_sq, color, piece) {
                continue;
            }
            match piece {
                Piece::Pawn => generate_pawn_moves(
                    board,
                    from_sq,
                    from_square_0x88,
                    color,
                    &mut moves,
                    state_info,
                ),
                Piece::Knight => generate_direct_moves(
                    board,
                    from_square_0x88,
                    &KNIGHT_OFFSETS,
                    color,
                    &mut moves,
                ),
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
                    generate_direct_moves(
                        board,
                        from_square_0x88,
                        &KING_OFFSETS,
                        color,
                        &mut moves,
                    );
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
    from_sq: Sq88,
    offsets: &[i8],
    c: Color,
    moves: &mut ArrayVec<Move, 256>,
) {
    for &offset in offsets {
        let to_sq = from_sq.step(offset);

        if !to_sq.is_on_board() {
            continue;
        }
        let target_square = to_sq.to_sq64();
        if board.is_occupied(target_square) {
            if !board.is_occupied_firendly(target_square, c) {
                moves.push(Move::new_flags(from_sq.to_sq64(), target_square, 0x4));
            }
        } else {
            moves.push(Move::new(from_sq.to_sq64(), target_square));
        }
    }
}

pub fn generate_pawn_moves(
    board: &Board,
    from_square: Sq64,
    from_sq: Sq88,
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
    let to_sqs = match c {
        Color::White => [Sq88(from_sq.0 + 17), Sq88(from_sq.0 + 15)],
        Color::Black => [Sq88(from_sq.0 - 17), Sq88(from_sq.0 - 15)],
    };
    for to_sq in to_sqs {
        if !to_sq.is_on_board() {
            continue;
        }
        let target_square = to_sq.to_sq64();
        if board.is_occupied_enemy(target_square, c) {
            //Can Take
            if target_square.0 >> 3 == 7 || target_square.0 >> 3 == 0 {
                //Promotion Capture
                for piece in Piece::PROMOTABLE {
                    moves.push(Move::new_flags(
                        from_square,
                        target_square,
                        0xC + piece as u8 - 1,
                    ));
                }
            } else {
                //Capture
                moves.push(Move::new_flags(from_square, target_square, 0x4));
            }
        } else {
            //EP.
            if state_info.ep_square == Some(target_square) {
                moves.push(Move::new_flags(from_square, target_square, 0x5));
            }
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
        //Not Occupied
        if ![from_square.0 + 1, from_square.0 + 2]
            .iter()
            .any(|sq| board.is_occupied(Sq64(*sq)))
            && [from_square.0, from_square.0 + 1, from_square.0 + 2]
                .iter()
                .any(|sq| is_square_attacked_by(Sq64(*sq).to_sq88(), c.flip(), board))
        {
            moves.push(Move::new_flags(from_square, Sq64(from_square.0 + 2), 0x2));
        }
    }

    //Check Long Castle
    if state_info.has_castle_rights(c, false) {
        //Not Occupied
        if ![from_square.0 - 1, from_square.0 - 2, from_square.0 - 3]
            .iter()
            .any(|sq| board.is_occupied(Sq64(*sq)))
            && [from_square.0, from_square.0 - 1, from_square.0 - 2]
                .iter()
                .any(|sq| is_square_attacked_by(Sq64(*sq).to_sq88(), c.flip(), board))
        {
            moves.push(Move::new_flags(from_square, Sq64(from_square.0 - 2), 0x2));
        }
    }
}

pub fn is_square_attacked_by(sq_0x88: Sq88, c: Color, board: &Board) -> bool {
    for po in [15, 17] {
        let nsq = match c {
            Color::White => sq_0x88.step(-po),
            Color::Black => sq_0x88.step(po),
        };
        if nsq.is_on_board() {
            if board.is_piece(nsq.to_sq64(), c, Piece::Pawn) {
                return true;
            }
        }
    }
    for no in KNIGHT_OFFSETS {
        let nsq = sq_0x88.step(no);
        if nsq.is_on_board() {
            if board.is_piece(nsq.to_sq64(), c, Piece::Knight) {
                return true;
            }
        }
    }
    for ko in KING_OFFSETS {
        let nsq = sq_0x88.step(ko);
        if nsq.is_on_board() {
            if board.is_piece(nsq.to_sq64(), c, Piece::King) {
                return true;
            }
        }
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

#[test]
fn test_perft() {
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    assert_eq!(
        number_of_moves(&mut BoardState::from_fen(start_fen).unwrap(), 5),
        4865609
    );
    let perft5 = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    assert_eq!(
        number_of_moves(&mut BoardState::from_fen(perft5).unwrap(), 3),
        62379
    );
}

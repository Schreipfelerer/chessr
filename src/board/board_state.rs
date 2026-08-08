use crate::board::{
    Board, Color, FenErr, Move, MoveFlag, Piece, StateInfo, Undo,
    zobrist::{ZOBRIST_CASTLING, ZOBRIST_EP, ZOBRIST_SIDE},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BoardState {
    pub board: Board,
    pub state_info: StateInfo,
    zobrist_hist: Vec<u64>,
    hash: u64,
}

impl BoardState {
    #[must_use]
    pub fn from_fen(fen: &str) -> Result<Self, FenErr> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(FenErr::InvalidFormat);
        }
        let mut bs = Self {
            board: Board::from_fen_part(parts[0])?,
            state_info: StateInfo::from_fen(&parts[1..])?,
            zobrist_hist: Vec::with_capacity(1000),
            hash: 0,
        };
        bs.update_zobrist();
        Ok(bs)
    }

    pub fn make_move(&mut self, m: Move) -> Undo {
        self.zobrist_hist.push(self.hash);

        let flags = m.flags();
        let from = m.source();
        let to = m.target();
        let color = self.state_info.active_color;
        let piece = self.board.get_piece_at(from);
        self.state_info.ep_square = None;
        self.board.remove_piece(from, color, piece);

        let mut undo = Undo::new(m, &self.state_info);

        if piece == Piece::King {
            self.state_info.remove_castle_rights(color);
        }
        if piece == Piece::Rook {
            self.state_info.clear_corner_castle_rights(from); // Remove Castle Rights
        }
        match flags {
            MoveFlag::Quiet => self.board.place_piece(to, color, piece),
            MoveFlag::DoublePawnPush => {
                self.board.place_piece(to, color, piece);
                self.state_info.ep_square = match color {
                    Color::White => Some(from + 8),
                    Color::Black => Some(from - 8),
                }
            }
            MoveFlag::CastleKingside => {
                self.board.place_piece(to, color, Piece::King);
                self.board.place_piece(from + 1, color, Piece::Rook);
                self.board.remove_piece(to + 1, color, Piece::Rook);
            }
            MoveFlag::CastleQueenside => {
                self.board.place_piece(to, color, Piece::King);
                self.board.place_piece(from - 1, color, Piece::Rook);
                self.board.remove_piece(to - 2, color, Piece::Rook);
            }
            MoveFlag::EnPassant => {
                self.board.place_piece(to, color, Piece::Pawn);
                undo.captured_piece = Some(Piece::Pawn);
                match color {
                    Color::White => self.board.remove_piece(to - 8, Color::Black, Piece::Pawn),
                    Color::Black => self.board.remove_piece(to + 8, Color::White, Piece::Pawn),
                }
            }
            _ if flags.is_capture() => {
                let cp = self.board.get_piece_at(to);
                undo.captured_piece = Some(cp);
                self.board.remove_piece(to, !color, cp);
                self.state_info.clear_corner_castle_rights(to); // Remove Castle Rights
                if flags == MoveFlag::Capture {
                    self.board.place_piece(to, color, piece);
                }
            }
            _ => (),
        }

        if flags.is_promotion() {
            let new_piece = flags.promoted_piece();
            self.board.place_piece(to, color, new_piece);
        }

        if piece == Piece::Pawn || flags.is_capture() {
            self.state_info.half_move_clock = 0;
        } else {
            self.state_info.half_move_clock += 1;
        }

        if color == Color::Black {
            self.state_info.full_move_number += 1;
        }
        self.state_info.active_color = !self.state_info.active_color;
        self.hash ^= ZOBRIST_SIDE;

        self.update_zobrist();
        undo
    }

    pub fn undo_move(&mut self, undo: &Undo) {
        let color = self.state_info.active_color;
        let prev_color = !color;
        let m = undo.r#move;
        let from = m.source();
        let to = m.target();
        let flag = m.flags();

        let piece = self.board.get_piece_at(to);
        self.board.remove_piece(to, prev_color, piece);

        if let Some(cp) = undo.captured_piece {
            if flag == MoveFlag::EnPassant {
                let csq = match color {
                    Color::White => to + 8,
                    Color::Black => to - 8,
                };
                self.board.place_piece(csq, color, cp);
            } else {
                self.board.place_piece(to, color, cp);
            }
        }

        if flag.is_promotion() {
            self.board.place_piece(from, prev_color, Piece::Pawn);
        } else {
            self.board.place_piece(from, prev_color, piece);
        }

        if flag == MoveFlag::CastleKingside {
            self.board.remove_piece(to - 1, prev_color, Piece::Rook);
            self.board.place_piece(to + 1, prev_color, Piece::Rook);
        }
        if flag == MoveFlag::CastleQueenside {
            self.board.remove_piece(to + 1, prev_color, Piece::Rook);
            self.board.place_piece(to - 2, prev_color, Piece::Rook);
        }

        self.state_info.undo(undo);
        self.zobrist_hist.pop();
        self.update_zobrist();
    }

    #[must_use]
    pub fn start_pos() -> Self {
        let mut bs = Self {
            board: Board::start_pos(),
            state_info: StateInfo::start_pos(),
            zobrist_hist: Vec::with_capacity(1000),
            hash: 0,
        };
        bs.update_zobrist();
        bs
    }

    fn update_zobrist(&mut self) {
        let mut zobrist = self.board.hash;
        zobrist ^= ZOBRIST_CASTLING[self.state_info.castle_rights as usize];
        if self.state_info.active_color == Color::Black {
            zobrist ^= ZOBRIST_SIDE;
        }
        if let Some(sq) = self.state_info.ep_square {
            zobrist ^= ZOBRIST_EP[sq.file() as usize]
        }

        self.hash = zobrist;
    }

    pub fn is_repetition(&self) -> bool {
        let clock = self.state_info.half_move_clock as usize;
        self.zobrist_hist
            .iter()
            .rev()
            .take(clock)
            .filter(|&&h| h == self.hash)
            .count()
            >= 2
    }
}

#[cfg(test)]
mod tests {
    use crate::board::BoardState;

    #[test]
    fn test_start_pos() {
        const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert_eq!(
            BoardState::start_pos(),
            BoardState::from_fen(START_FEN).unwrap()
        );
    }
}

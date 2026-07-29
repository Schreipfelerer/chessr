use crate::board::{Color, Sq64, Undo};

const WHITE_KINGSIDE: u8 = 0b1000;
const WHITE_QUEENSIDE: u8 = 0b0100;
const BLACK_KINGSIDE: u8 = 0b0010;
const BLACK_QUEENSIDE: u8 = 0b0001;

#[derive(Debug, Copy, Clone)]
pub struct StateInfo {
    pub castle_rights: u8, // Bit 0-3 Unused, White Short, White Long, Black Short, Black Long
    pub active_color: Color,
    pub half_move_clock: u8,
    pub full_move_number: u32,
    pub ep_square: Option<Sq64>,
}
impl StateInfo {
    pub fn from_fen(fen_parts: &[&str]) -> Result<Self, FenErr> {
        Ok(Self {
            active_color: match fen_parts[0] {
                "w" => Color::White,
                "b" => Color::Black,
                _ => return Err(FenErr::InvalidSideToMove),
            },
            castle_rights: castle_rights(fen_parts[1])?,
            ep_square: match fen_parts[2] {
                "-" => None,
                sq => Some(square_from_algebratic(sq)?),
            },
            half_move_clock: fen_parts[3]
                .parse()
                .map_err(|_| FenErr::InvalidHalfmoveClock)?,
            full_move_number: fen_parts[4]
                .parse()
                .map_err(|_| FenErr::InvalidFullmoveNumber)?,
        })
    }
    #[must_use]
    pub fn has_castle_rights(&self, color: Color, is_short: bool) -> bool {
        let mut offset = match color {
            Color::Black => 0,
            Color::White => 2,
        };
        if is_short {
            offset += 1;
        }
        self.castle_rights >> offset & 0x1 == 1
    }
    pub fn clear_corner_castle_rights(&mut self, sq: Sq64) {
        let mask = match sq.0 {
            0 => WHITE_QUEENSIDE,
            7 => WHITE_KINGSIDE,
            56 => BLACK_QUEENSIDE,
            63 => BLACK_KINGSIDE,
            _ => 0,
        };
        self.castle_rights &= !mask;
    }
    pub fn remove_castle_rights(&mut self, color: Color) {
        let offset = match color {
            Color::Black => 0,
            Color::White => 2,
        };
        self.castle_rights &= !(0b0011 << offset);
    }

    pub fn undo(&mut self, undo: &Undo) {
        self.ep_square = undo.prev_ep_square;
        self.castle_rights = undo.prev_castling_rights;
        self.half_move_clock = undo.prev_halfmove_clock;
        if self.active_color == Color::White {
            self.full_move_number -= 1;
        }
        self.active_color = !self.active_color;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FenErr {
    InvalidFormat,               // Doesnt have exactly 6 Spaces
    InvalidCharInPiecePlacement, // Illegal Piece in Placement
    InvalidRankLength,           // Each Rank must have 8 pieces
    InvalidRankCount,            // Must have 8 ranks
    InvalidSideToMove,           // Side to Move must be 'w' or 'b'
    InvalidHalfmoveClock,        // HalfMoveClock should be 0..49
    InvalidFullmoveNumber,       // FullMoveNumber should be a number
    InvalidSquare,               // EnPassantSquare should be -/[a-h][3/6]
    InvalidCastleRights,         // Castle Rights should be -/[KQkq](1..4)
}

fn castle_rights(rights: &str) -> Result<u8, FenErr> {
    if rights == "-" {
        return Ok(0);
    }

    let mut cr = 0;
    for c in rights.as_bytes() {
        match c {
            b'K' => cr |= 8,
            b'Q' => cr |= 4,
            b'k' => cr |= 2,
            b'q' => cr |= 1,
            _ => return Err(FenErr::InvalidCastleRights),
        }
    }
    Ok(cr)
}

fn square_from_algebratic(s: &str) -> Result<Sq64, FenErr> {
    let [file, rank] = s.as_bytes() else {
        return Err(FenErr::InvalidSquare);
    };

    if !(b'a'..=b'h').contains(file) || !(b'1'..b'8').contains(rank) {
        return Err(FenErr::InvalidSquare);
    }
    let file = file - b'a';
    let rank = rank - b'1';
    Ok(Sq64(rank * 8 + file))
}

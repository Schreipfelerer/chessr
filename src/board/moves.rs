use crate::board::{BoardState, Color, Piece, Sq64, StateInfo};
use std::fmt;

// Bit 0-5 Source Square
// Bit 6-11 Target Square
// Bit 12-15 Special Flags (Promotion Flag, Castle Flag, Special Flags)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move(pub u16);

impl Move {
    #[inline(always)]
    #[must_use]
    pub fn new(from: Sq64, to: Sq64) -> Self {
        Move((from.0 as u16) | ((to.0 as u16) << 6))
    }

    #[inline(always)]
    #[must_use]
    pub fn new_flags(from: Sq64, to: Sq64, flags: MoveFlag) -> Self {
        Move((from.0 as u16) | ((to.0 as u16) << 6) | ((flags as u16) << 12))
    }

    #[inline(always)]
    #[must_use]
    pub fn flags(self) -> MoveFlag {
        unsafe { std::mem::transmute::<u8, MoveFlag>((self.0 >> 12) as u8) }
    }
    #[inline(always)]
    #[must_use]
    pub fn target(self) -> Sq64 {
        Sq64(((self.0 >> 6) & 0x3F) as u8)
    }
    #[inline(always)]
    #[must_use]
    pub fn source(self) -> Sq64 {
        Sq64((self.0 & 0x3F) as u8)
    }

    #[must_use]
    pub fn from_notation(bs: &BoardState, notation: &str) -> Option<Self> {
        let color = bs.state_info.active_color;
        if notation == "0-0" {
            return Some(match color {
                Color::White => Move(0o04 | 0o06 << 6 | 2 << 12),
                Color::Black => Move(0o74 | 0o76 << 6 | 2 << 12),
            });
        }
        if notation == "0-0-0" {
            return Some(match color {
                Color::White => Move(0o04 | 0o02 << 6 | 3 << 12),
                Color::Black => Move(0o74 | 0o72 << 6 | 3 << 12),
            });
        }

        let bytes = notation.as_bytes();

        if bytes.len() != 4 && bytes.len() != 5 {
            return None;
        }

        let mut flag = 0;
        if bytes.len() == 5 {
            flag = match bytes[5] {
                b'n' => 8,
                b'b' => 9,
                b'r' => 10,
                b'q' => 11,
                _ => return None,
            };
        }

        let from_sq = Sq64::from_notation(&bytes[..2])?;
        let to_sq = Sq64::from_notation(&bytes[2..4])?;

        if bs.board.is_occupied_enemy(to_sq, color) {
            flag += 4;
        }
        Some(Move(from_sq.0 as u16 | (to_sq.0 as u16) << 6 | flag << 12))
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.source(), self.target())?;
        if self.flags().is_promotion() {
            let promo_string = match self.flags().promoted_piece() {
                Piece::Knight => "n",
                Piece::Bishop => "b",
                Piece::Rook => "r",
                Piece::Queen => "q",
                _ => unreachable!(),
            };
            write!(f, "{promo_string}")?;
        }
        Ok(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Quiet = 0,
    DoublePawnPush = 1,
    CastleKingside = 2,
    CastleQueenside = 3,
    Capture = 4,
    EnPassant = 5,
    PromoKnight = 8,
    PromoBishop = 9,
    PromoRook = 10,
    PromoQueen = 11,
    PromoKnightCapture = 12,
    PromoBishopCapture = 13,
    PromoRookCapture = 14,
    PromoQueenCapture = 15,
}

impl MoveFlag {
    #[must_use]
    pub const fn new_promotion(p: Piece) -> Self {
        match p {
            Piece::Knight => MoveFlag::PromoKnight,
            Piece::Bishop => MoveFlag::PromoBishop,
            Piece::Rook => MoveFlag::PromoRook,
            Piece::Queen => MoveFlag::PromoQueen,
            _ => unreachable!(),
        }
    }
    #[must_use]
    pub const fn new_promotion_capture(p: Piece) -> Self {
        match p {
            Piece::Knight => MoveFlag::PromoKnightCapture,
            Piece::Bishop => MoveFlag::PromoBishopCapture,
            Piece::Rook => MoveFlag::PromoRookCapture,
            Piece::Queen => MoveFlag::PromoQueenCapture,
            _ => unreachable!(),
        }
    }
    #[inline(always)]
    #[must_use]
    pub const fn is_promotion(self) -> bool {
        (self as u8) & 8 != 0
    }
    #[inline(always)]
    #[must_use]
    pub const fn is_capture(self) -> bool {
        (self as u8) & 4 != 0
    }
    #[must_use]
    pub const fn promoted_piece(self) -> Piece {
        match (self as u8) & 3 {
            0 => Piece::Knight,
            1 => Piece::Bishop,
            2 => Piece::Rook,
            3 => Piece::Queen,
            _ => unreachable!(),
        }
    }
}

pub struct Undo {
    pub(crate) r#move: Move,
    pub(crate) captured_piece: Option<Piece>,
    pub(crate) prev_halfmove_clock: u8,
    pub(crate) prev_castling_rights: u8,
    pub(crate) prev_ep_square: Option<Sq64>,
}

impl Undo {
    #[must_use]
    pub fn new(m: Move, state_info: &StateInfo) -> Self {
        Self {
            r#move: m,
            captured_piece: None,
            prev_halfmove_clock: state_info.half_move_clock,
            prev_castling_rights: state_info.castle_rights,
            prev_ep_square: state_info.ep_square,
        }
    }
}

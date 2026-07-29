use crate::board::{BoardState, Color, Move, Piece, Sq64, StateInfo};
use std::fmt;

pub struct Undo {
    pub(crate) r#move: Move,
    pub(crate) captured_piece: Option<Piece>,
    pub(crate) prev_halfmove_clock: u8,
    pub(crate) prev_castling_rights: u8,
    pub(crate) prev_ep_square: Option<Sq64>,
}

impl Undo {
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

impl Move {
    #[inline(always)]
    pub fn new(from: Sq64, to: Sq64) -> Self {
        Move((from.0 as u16) | ((to.0 as u16) << 6))
    }

    #[inline(always)]
    pub fn new_flags(from: Sq64, to: Sq64, flags: MoveFlag) -> Self {
        Move((from.0 as u16) | ((to.0 as u16) << 6) | ((flags as u16) << 12))
    }

    #[inline(always)]
    pub fn flags(self) -> MoveFlag {
        unsafe { std::mem::transmute::<u8, MoveFlag>((self.0 >> 12) as u8) }
    }
    #[inline(always)]
    pub fn target(self) -> Sq64 {
        Sq64(((self.0 >> 6) & 0x3F) as u8)
    }
    #[inline(always)]
    pub fn source(self) -> Sq64 {
        Sq64((self.0 & 0x3F) as u8)
    }

    pub fn from_notation(bs: &BoardState, notation: &str) -> Option<Self> {
        let color = bs.state_info.active_color();
        if notation == "0-0" {
            return Some(match color {
                Color::White => Move(4 | 6 << 6 | 2 << 12),
                Color::Black => Move(60 | 62 << 6 | 2 << 12),
            });
        }
        if notation == "0-0-0" {
            return Some(match color {
                Color::White => Move(4 | 2 << 6 | 3 << 12),
                Color::Black => Move(60 | 58 << 6 | 3 << 12),
            });
        }

        let bytes = notation.as_bytes();

        if bytes.len() != 4 && bytes.len() != 6 {
            return None;
        }

        let mut flag = 0;
        if bytes.len() == 6 {
            flag = match (bytes[4], bytes[5]) {
                (b'=', b'N' | b'K') => 8,
                (b'=', b'B') => 9,
                (b'=', b'R') => 10,
                (b'=', b'Q') => 11,
                (b'e', b'P') => 5,
                _ => return None,
            };
        }

        let from_sq = Sq64::from_notation(&bytes[..2])?;
        let to_sq = Sq64::from_notation(&bytes[2..4])?;

        if bs.board.is_occupied_enemy(to_sq, color) {
            flag += 4
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
            write!(f, "{}", promo_string)?;
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
    pub const fn new_promotion(p: Piece) -> Self {
        match p {
            Piece::Knight => MoveFlag::PromoKnight,
            Piece::Bishop => MoveFlag::PromoBishop,
            Piece::Rook => MoveFlag::PromoRook,
            Piece::Queen => MoveFlag::PromoQueen,
            _ => unreachable!(),
        }
    }
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
    pub const fn is_promotion(self) -> bool {
        (self as u8) & 8 != 0
    }
    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        (self as u8) & 4 != 0
    }
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
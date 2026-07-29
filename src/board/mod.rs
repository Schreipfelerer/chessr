pub use crate::board::r#move::{MoveFlag, Undo};

mod r#move;
mod color;
mod piece;
mod sq64;
mod board_state;

#[derive(Debug)]
pub struct Board {
    pieces: [[u64; 6]; 2], // [Color, PieceType]
    occupancy: [u64; 3],   // [White, Black, Both]
    mailbox: [Option<Piece>; 64],
}

#[derive(Debug)]
pub struct BoardState {
    pub board: Board,
    pub state_info: StateInfo,
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

#[derive(Debug)]
pub struct StateInfo {
    pub castle_rights: u8, // Bit 0-3 Unused, White Short, White Long, Black Short, Black Long
    pub is_whites_turn: bool,
    pub half_move_clock: u8,
    pub full_move_number: u32,
    pub ep_square: Option<Sq64>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

// Bit 0-5 Source Square
// Bit 6-11 Target Square
// Bit 12-15 Special Flags (Promotion Flag, Castle Flag, Special Flags)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sq64(pub u8);


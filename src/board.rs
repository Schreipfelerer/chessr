use std::{
    fmt,
    ops::{Add, Sub},
};

#[derive(Debug)]
pub struct Board {
    pieces: [[u64; 6]; 2], // [Color, PieceType]
    occupancy: [u64; 3],   // [White, Black, Both]
    mailbox: [Option<Piece>; 64],
}

impl Board {
    pub fn from_fen_part(fen_part: &str) -> Result<Self, FenErr> {
        let mut pieces: [[u64; 6]; 2] = [[0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0]];
        let mut rank: u32 = 7;
        let mut file = 0;
        let mut mailbox: [Option<Piece>; 64] = [None; 64];
        for c in fen_part.chars() {
            match c {
                '/' => {
                    if file != 8 {
                        return Err(FenErr::InvalidRankLength);
                    }
                    file = 0;
                    rank = rank.checked_sub(1).ok_or(FenErr::InvalidRankCount)?;
                }
                '1'..='8' => file += c.to_digit(10).unwrap(),
                _ => {
                    let color: Color = c.is_lowercase().into();
                    let piece: Piece = match c.to_ascii_lowercase() {
                        'p' => Piece::Pawn,
                        'b' => Piece::Bishop,
                        'n' => Piece::Knight,
                        'r' => Piece::Rook,
                        'q' => Piece::Queen,
                        'k' => Piece::King,
                        _ => return Err(FenErr::InvalidCharInPiecePlacement),
                    };
                    mailbox[(file + rank * 8) as usize] = Some(piece);
                    pieces[color as usize][piece as usize] |= 1u64 << file + rank * 8;
                    file += 1;
                }
            }
        }
        if file != 8 {
            return Err(FenErr::InvalidRankLength);
        }
        if rank != 0 {
            return Err(FenErr::InvalidRankCount);
        }

        let mut occupancies: [u64; 3] = [0, 0, 0];
        for color in Color::ALL {
            for piece in Piece::ALL {
                occupancies[color as usize] |= pieces[color as usize][piece as usize];
            }
        }
        occupancies[2] = occupancies[0] | occupancies[1];

        Ok(Self {
            pieces: pieces,
            occupancy: occupancies,
            mailbox: mailbox,
        })
    }
    pub fn get_piece_bitboard(&self, color: Color, piece: Piece) -> u64 {
        self.pieces[color as usize][piece as usize]
    }
    pub fn is_occupied(&self, sq: Sq64) -> bool {
        sq.is_on_bb(self.occupancy[2])
    }
    pub fn is_occupied_enemy(&self, sq: Sq64, color: Color) -> bool {
        sq.is_on_bb(self.occupancy[color.flip() as usize])
    }
    pub fn remove_piece(&mut self, sq: Sq64, color: Color, piece: Piece) {
        let mask = sq.mask();
        self.pieces[color as usize][piece as usize] ^= mask;
        self.occupancy[color as usize] ^= mask;
        self.occupancy[2] ^= mask;
        self.mailbox[sq.ind()] = None;
    }
    pub fn place_piece(&mut self, sq: Sq64, color: Color, piece: Piece) {
        let mask = sq.mask();
        self.pieces[color as usize][piece as usize] ^= mask;
        self.occupancy[color as usize] ^= mask;
        self.occupancy[2] ^= mask;
        self.mailbox[sq.ind()] = Some(piece);
    }

    pub fn get_piece_at(&self, sq: Sq64) -> Piece {
        self.mailbox[sq.ind()].unwrap()
    }

    fn get_piece_visual(&self, rank: u8, file: u8) -> char {
        let sq = file + rank * 8;
        let bit = 1u64 << sq;
        if self.get_occupancy() & bit == 0 {
            return '.';
        }
        let mut char = match self.get_piece_at(Sq64(sq)) {
            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
        };
        if self.occupancy[0] & bit == 0 {
            char = char.to_ascii_lowercase();
        }
        char
    }

    pub fn find_king(&self, color: Color) -> Sq64 {
        let bb = self.pieces[color as usize][Piece::King as usize];
        Sq64(bb.trailing_zeros() as u8)
    }

    pub fn get_friendly_occupancy(&self, c: Color) -> u64 {
        self.occupancy[c as usize]
    }

    pub fn get_enemy_occupancy(&self, c: Color) -> u64 {
        self.occupancy[c.flip() as usize]
    }

    pub fn get_occupancy(&self) -> u64 {
        self.occupancy[2]
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  -----------------")?;
        for rank in (0..8).rev() {
            write!(f, "{} ", rank + 1)?;
            for file in 0..8 {
                write!(f, "|{}", self.get_piece_visual(rank, file))?;
            }
            write!(f, "|\n")?;
            writeln!(f, "  -----------------")?;
        }
        writeln!(f, "   a b c d e f g h ")
    }
}

#[derive(Debug)]
pub struct BoardState {
    pub board: Board,
    pub state_info: StateInfo,
}

impl BoardState {
    pub fn from_fen(fen: &str) -> Result<Self, FenErr> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(FenErr::InvalidFormat);
        }
        Ok(Self {
            board: Board::from_fen_part(parts[0])?,
            state_info: StateInfo::from_fen(&parts[1..])?,
        })
    }

    pub fn make_move(&mut self, m: Move) -> Undo {
        let flags = m.flags();
        let from = m.source();
        let to = m.target();
        let color = self.state_info.active_color();
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
                self.board.remove_piece(to, color.flip(), cp);
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
            self.state_info.half_move_clock = 0
        } else {
            self.state_info.half_move_clock += 1
        }

        if color == Color::Black {
            self.state_info.full_move_number += 1
        }
        self.state_info.is_whites_turn = !self.state_info.is_whites_turn;

        undo
    }

    pub fn undo_move(&mut self, undo: Undo) {
        let color = self.state_info.active_color();
        let prev_color = color.flip();
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

        self.state_info.ep_square = undo.prev_ep_square;
        self.state_info.castle_rights = undo.prev_castling_rights;
        self.state_info.half_move_clock = undo.prev_halfmove_clock;
        if color == Color::White {
            self.state_info.full_move_number -= 1
        }
        self.state_info.is_whites_turn = !self.state_info.is_whites_turn;
    }
}

pub struct Undo {
    r#move: Move,
    captured_piece: Option<Piece>,
    prev_halfmove_clock: u8,
    prev_castling_rights: u8,
    prev_ep_square: Option<Sq64>,
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

#[derive(Debug, Clone, Copy)]
pub enum FenErr {
    InvalidFormat,               // Doesnt have exaktly 6 Spaces
    InvalidCharInPiecePlacement, // Illeagal Piece in Placement
    InvalidRankLength,           // Each Rankmust have 8 pieces
    InvalidRankCount,            // Must have 8 ranks
    InvalidSideToMove,           // Side to Move must be 'w' or 'b'
    InvalidHalfmoveClock,        // HalfMoveClock should be 0..49
    InvalidFullmoveNumber,       // FullMoveNumber should be a number
    InvalidSquare,               // EnPassantSquare should be -/[a-h][3/6]
    InvalidCastleRights,         // Castle Rights should be -/[KQkq](1..4)
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

#[derive(Debug)]
pub struct StateInfo {
    pub castle_rights: u8, // Bit 0-3 Unused, White Short, White Long, Black Short, Black Long
    pub is_whites_turn: bool,
    pub half_move_clock: u8,
    pub full_move_number: u32,
    pub ep_square: Option<Sq64>,
}

impl StateInfo {
    pub fn from_fen(fen_parts: &[&str]) -> Result<Self, FenErr> {
        Ok(Self {
            is_whites_turn: match fen_parts[0] {
                "w" => true,
                "b" => false,
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
    pub fn active_color(&self) -> Color {
        (!self.is_whites_turn).into()
    }
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
            0 => 0b0100,  // White queenside rook home
            7 => 0b1000,  // White kingside rook home
            56 => 0b0001, // Black queenside rook home
            63 => 0b0010, // Black kingside rook home
            _ => 0,
        };
        self.castle_rights &= !mask;
    }
    pub fn remove_castle_rights(&mut self, color: Color) {
        let offset = match color {
            Color::Black => 0,
            Color::White => 2,
        };
        self.castle_rights &= !(0b0011 << offset)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub const ALL: [Color; 2] = [Color::White, Color::Black];
    pub fn flip(self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

impl Into<Color> for bool {
    fn into(self) -> Color {
        match self {
            false => Color::White,
            true => Color::Black,
        }
    }
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

impl Piece {
    pub const ALL: [Piece; 6] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];
    pub const PROMOTABLE: [Piece; 4] = [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];
}

// Bit 0-5 Source Square
// Bit 6-11 Target Square
// Bit 12-15 Special Flags (Promotion Flag, Castle Flag, Special Flags)
// 0000: Quiet move
// 0001: Double pawn push
// 0010: King castle / 0011: Queen castle
// 0100: Capture
// 0101: EP capture
// 1000–1011: Promotions (Knight, Bishop, Rook, Queen)
// 1100–1111: Capture + Promotions (Knight, Bishop, Rook, Queen)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(pub u16);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sq64(pub u8);
impl Sq64 {
    #[inline(always)]
    pub fn from_notation(notation: &[u8]) -> Option<Self> {
        if notation.len() != 2 {
            return None;
        }
        if !(b'a'..=b'h').contains(&notation[0]) {
            return None;
        }
        if !(b'1'..=b'8').contains(&notation[1]) {
            return None;
        }
        Some(Sq64(notation[0] - b'a' + ((notation[1] - b'1') * 8)))
    }
    #[inline(always)]
    pub fn rank(self) -> u8 {
        self.0 >> 3
    }
    #[inline(always)]
    pub fn file(self) -> u8 {
        self.0 & 7
    }
    #[inline(always)]
    pub fn mask(self) -> u64 {
        1 << self.0
    }
    #[inline(always)]
    pub fn is_on_bb(self, bb: u64) -> bool {
        bb >> self.0 & 1 == 1
    }
    #[inline(always)]
    pub fn ind(self) -> usize {
        self.0 as usize
    }
}
impl Add<i8> for Sq64 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: i8) -> Self::Output {
        Sq64(self.0.wrapping_add_signed(rhs))
    }
}
impl Sub<i8> for Sq64 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: i8) -> Self::Output {
        Sq64(self.0.wrapping_sub_signed(rhs))
    }
}

impl fmt::Display for Sq64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            (b'a' + self.file()) as char,
            (b'1' + self.rank()) as char
        )
    }
}

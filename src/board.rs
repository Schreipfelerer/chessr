use std::fmt;

pub struct Board {
    pieces: [[u64; 6]; 2], // [Color, PieceType]
    occupancy: [u64; 3],   // [White, Black, Both]
}

impl Board {
    pub fn from_fen_part(fen_part: &str) -> Result<Self, FenErr> {
        let mut pieces: [[u64; 6]; 2] = [[0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0]];
        let mut rank = 7;
        let mut file = 0;
        for c in fen_part.chars() {
            match c {
                '/' => {
                    if file != 8 {
                        return Err(FenErr::InvaidRankLength);
                    }
                    file = 0;
                    rank -= 1;
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
                    pieces[color as usize][piece as usize] |= 1u64 << file + rank * 8;
                    file += 1;
                }
            }
        }
        if file != 8 {
            return Err(FenErr::InvaidRankLength);
        }
        if rank != 0 {
            return Err(FenErr::InvaidRankCount);
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
        })
    }
    pub fn get_piece_bitboard(&self, color: Color, piece: Piece) -> u64 {
        self.pieces[color as usize][piece as usize]
    }
    pub fn get_color_occupancy(self, color: Color) -> u64 {
        self.occupancy[color as usize]
    }
    pub fn get_occupancy(self) -> u64 {
        self.occupancy[2]
    }

    fn get_piece_visual(&self, rank: u8, file: u8) -> char {
        let bit = 1u64 << file + rank * 8;
        if self.get_piece_bitboard(Color::White, Piece::Pawn) & bit != 0 {
            return 'P';
        }
        if self.get_piece_bitboard(Color::White, Piece::Knight) & bit != 0 {
            return 'N';
        }
        if self.get_piece_bitboard(Color::White, Piece::Bishop) & bit != 0 {
            return 'B';
        }
        if self.get_piece_bitboard(Color::White, Piece::Rook) & bit != 0 {
            return 'R';
        }
        if self.get_piece_bitboard(Color::White, Piece::Queen) & bit != 0 {
            return 'Q';
        }
        if self.get_piece_bitboard(Color::White, Piece::King) & bit != 0 {
            return 'K';
        }
        if self.get_piece_bitboard(Color::Black, Piece::Pawn) & bit != 0 {
            return 'p';
        }
        if self.get_piece_bitboard(Color::Black, Piece::Knight) & bit != 0 {
            return 'n';
        }
        if self.get_piece_bitboard(Color::Black, Piece::Bishop) & bit != 0 {
            return 'b';
        }
        if self.get_piece_bitboard(Color::Black, Piece::Rook) & bit != 0 {
            return 'r';
        }
        if self.get_piece_bitboard(Color::Black, Piece::Queen) & bit != 0 {
            return 'q';
        }
        if self.get_piece_bitboard(Color::Black, Piece::King) & bit != 0 {
            return 'k';
        }
        return '.';
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

pub struct BoardState {
    board: Board,
    state_info: StateInfo,
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
    pub fn board(&self) -> &Board {
        &self.board
    }
}

#[derive(Debug)]
pub enum FenErr {
    InvalidFormat,               // Doesnt have exaktly 6 Spaces
    InvalidCharInPiecePlacement, // Illeagal Piece in Placement
    InvaidRankLength,            // Each Rankmust have 8 pieces
    InvaidRankCount,             // Must have 8 ranks
    InvalidSideToMove,           // Side to Move must be 'w' or 'b'
    InvalidHalfmoveClock,        // HalfMoveClock should be 0..49
    InvalidFullmoveNumber,       // FullMoveNumber should be a number
    InvalidSquare,               // EnPassantSquare should be -/[a-h][3/6]
    InvalidCastleRights,         // Castle Rights should be -/[KQkq](1..4)
}

fn square_from_algebratic(s: &str) -> Result<u8, FenErr> {
    let [file, rank] = s.as_bytes() else {
        return Err(FenErr::InvalidSquare);
    };

    if !(b'a'..=b'h').contains(file) || !(b'1'..b'8').contains(rank) {
        return Err(FenErr::InvalidSquare);
    }
    let file = file - b'a';
    let rank = rank - b'1';
    Ok(rank * 8 + file)
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

struct StateInfo {
    has_castle_rights: u8, // Bit 0-3 Unused, White Short, White Long, Black Short, Black Long
    is_white_to_move: bool,
    half_move_clock: u8,
    full_move_number: u32,
    ep_square: Option<u8>,
}

impl StateInfo {
    pub fn from_fen(fen_parts: &[&str]) -> Result<Self, FenErr> {
        Ok(Self {
            is_white_to_move: match fen_parts[0] {
                "w" => true,
                "b" => false,
                _ => return Err(FenErr::InvalidSideToMove),
            },
            has_castle_rights: castle_rights(fen_parts[1])?,
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
}

struct Move {
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
    r#move: u16,
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub const ALL: [Color; 2] = [Color::White, Color::Black];
}

impl Into<Color> for bool {
    fn into(self) -> Color {
        match self {
            false => Color::White,
            true => Color::Black,
        }
    }
}

#[derive(Copy, Clone)]
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
}

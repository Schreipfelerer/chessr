use crate::board::{Color, FenErr, Piece, Sq64, zobrist::ZOBRIST_PIECES};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Board {
    pieces: [[u64; 6]; 2], // [Color, PieceType]
    occupancy: [u64; 3],   // [White, Black, Both]
    mailbox: [Option<Piece>; 64],
    pub(crate) hash: u64,
}

impl Board {
    #[must_use]
    pub fn from_fen_part(fen_part: &str) -> Result<Self, FenErr> {
        let mut pieces: [[u64; 6]; 2] = [[0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0]];
        let mut rank: u32 = 7;
        let mut file = 0;
        let mut mailbox: [Option<Piece>; 64] = [None; 64];
        let mut hash = 0;
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
                    let sq = file + rank * 8;
                    mailbox[sq as usize] = Some(piece);
                    pieces[color as usize][piece as usize] |= 1u64 << sq;
                    hash ^= ZOBRIST_PIECES[color as usize][piece as usize][sq as usize];
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

        let mut occupancy: [u64; 3] = [0, 0, 0];
        for color in Color::ALL {
            for piece in Piece::ALL {
                occupancy[color as usize] |= pieces[color as usize][piece as usize];
            }
        }
        occupancy[2] = occupancy[0] | occupancy[1];

        Ok(Self {
            pieces,
            occupancy,
            mailbox,
            hash,
        })
    }
    #[must_use]
    pub const fn start_pos() -> Self {
        let mut hash = 0;
        let pawn_bb = 0xFF00;
        let knight_bb = 0x42;
        let bishop_bb = 0x24;
        let rook_bb = 0x81;
        let queen_bb = 0x8;
        let king_bb = 0x10;
        let pieces = [
            [pawn_bb, knight_bb, bishop_bb, rook_bb, queen_bb, king_bb],
            [
                pawn_bb << (8 * 5),
                knight_bb << (8 * 7),
                bishop_bb << (8 * 7),
                rook_bb << (8 * 7),
                queen_bb << (8 * 7),
                king_bb << (8 * 7),
            ],
        ];
        let mut mailbox = [None; 64];

        // White
        mailbox[0] = Some(Piece::Rook);
        hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::Rook as usize][0];
        mailbox[1] = Some(Piece::Knight);
        hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::Knight as usize][1];
        mailbox[2] = Some(Piece::Bishop);
        hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::Bishop as usize][2];
        mailbox[3] = Some(Piece::Queen);
        hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::Queen as usize][3];
        mailbox[4] = Some(Piece::King);
        hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::King as usize][4];
        mailbox[5] = Some(Piece::Bishop);
        hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::Bishop as usize][5];
        mailbox[6] = Some(Piece::Knight);
        hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::Knight as usize][6];
        mailbox[7] = Some(Piece::Rook);
        hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::Rook as usize][7];

        let mut i = 8;
        while i < 16 {
            mailbox[i] = Some(Piece::Pawn);
            hash ^= ZOBRIST_PIECES[Color::White as usize][Piece::Pawn as usize][i];
            i += 1;
        }

        // Black
        i = 48;
        while i < 56 {
            mailbox[i] = Some(Piece::Pawn);
            hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::Pawn as usize][i];
            i += 1;
        }

        mailbox[56] = Some(Piece::Rook);
        hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::Rook as usize][56];
        mailbox[57] = Some(Piece::Knight);
        hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::Knight as usize][57];
        mailbox[58] = Some(Piece::Bishop);
        hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::Bishop as usize][58];
        mailbox[59] = Some(Piece::Queen);
        hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::Queen as usize][59];
        mailbox[60] = Some(Piece::King);
        hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::King as usize][60];
        mailbox[61] = Some(Piece::Bishop);
        hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::Bishop as usize][61];
        mailbox[62] = Some(Piece::Knight);
        hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::Knight as usize][62];
        mailbox[63] = Some(Piece::Rook);
        hash ^= ZOBRIST_PIECES[Color::Black as usize][Piece::Rook as usize][63];
        Self {
            pieces,
            occupancy: [0xFFFF, 0xFFFF_0000_0000_0000, 0xFFFF_0000_0000_FFFF],
            mailbox,
            hash,
        }
    }
    #[must_use]
    pub fn get_piece_bitboard(&self, color: Color, piece: Piece) -> u64 {
        self.pieces[color as usize][piece as usize]
    }
    #[must_use]
    pub fn is_occupied(&self, sq: Sq64) -> bool {
        sq.is_on_bb(self.occupancy[2])
    }
    #[must_use]
    pub fn is_occupied_enemy(&self, sq: Sq64, color: Color) -> bool {
        sq.is_on_bb(self.occupancy[!color as usize])
    }
    pub fn remove_piece(&mut self, sq: Sq64, color: Color, piece: Piece) {
        let mask = sq.mask();
        self.pieces[color as usize][piece as usize] ^= mask;
        self.occupancy[color as usize] ^= mask;
        self.occupancy[2] ^= mask;
        self.mailbox[sq.ind()] = None;
        self.hash ^= ZOBRIST_PIECES[color as usize][piece as usize][sq.ind()];
    }
    pub fn place_piece(&mut self, sq: Sq64, color: Color, piece: Piece) {
        let mask = sq.mask();
        self.pieces[color as usize][piece as usize] ^= mask;
        self.occupancy[color as usize] ^= mask;
        self.occupancy[2] ^= mask;
        self.mailbox[sq.ind()] = Some(piece);
        self.hash ^= ZOBRIST_PIECES[color as usize][piece as usize][sq.ind()];
    }

    /// Panics if sq is empty
    /// caller must check if the square is occupied
    #[must_use]
    pub fn get_piece_at(&self, sq: Sq64) -> Piece {
        self.mailbox[sq.ind()].unwrap()
    }

    #[must_use]
    fn get_piece_visual(&self, rank: u8, file: u8) -> char {
        let sq = file + rank * 8;
        let bit = 1u64 << sq;
        if self.occ() & bit == 0 {
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

    #[must_use]
    pub fn find_king(&self, color: Color) -> Sq64 {
        let bb = self.pieces[color as usize][Piece::King as usize];
        Sq64(bb.trailing_zeros() as u8)
    }

    #[must_use]
    pub fn occ_friendly(&self, c: Color) -> u64 {
        self.occupancy[c as usize]
    }

    #[must_use]
    pub fn occ_enemy(&self, c: Color) -> u64 {
        self.occupancy[!c as usize]
    }

    #[must_use]
    pub fn occ(&self) -> u64 {
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
            writeln!(f, "|")?;
            writeln!(f, "  -----------------")?;
        }
        writeln!(f, "   a b c d e f g h ")
    }
}

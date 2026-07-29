use crate::board::Piece;

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
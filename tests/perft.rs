#[cfg(test)]
mod perft_tests {
    use chessr::{board::BoardState, movegen::perft::number_of_moves};

    #[test]
    fn start_pos() {
        let mut board =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        assert_eq!(number_of_moves(&mut board, 4), 197_281);
    }

    #[test]
    fn pos_wiki() {
        let pos2 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos2).unwrap(), 3),
            97_862
        );
        let pos3 = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos3).unwrap(), 4),
            43_238
        );
        let pos4 = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos4).unwrap(), 3),
            9_467
        );
        let pos5 = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos5).unwrap(), 3),
            62_379
        );
        let pos6 = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(pos6).unwrap(), 3),
            89_890
        );
    }

    #[test]
    fn en_passant_discovered_check() {
        let fen = "7k/8/8/K1pP3r/8/8/8/8 w - c6 0 1";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(fen).unwrap(), 5),
            44_848
        );
    }

    #[test]
    fn complicated_midgame() {
        let fen = "r3k2r/1bp2pP1/5n2/1P1Q4/1pPq4/5N2/1B1P2p1/R3K2R b KQkq c3 0 1";
        assert_eq!(
            number_of_moves(&mut BoardState::from_fen(fen).unwrap(), 3),
            113_742
        );
    }
}

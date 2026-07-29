#[cfg(test)]
mod perft_tests {
    use chessr::{board::BoardState, movegen::perft::count_moves};

    struct FenTest<'a> {
        fen: &'a str,
        depth: u8,
        actual_moves: u32,
        explanation: &'a str,
    }
    impl FenTest<'_> {
        fn test(self) {
            let mut board = BoardState::from_fen(self.fen).unwrap();
            assert_eq!(
                count_moves(&mut board, self.depth),
                self.actual_moves,
                "{}",
                self.explanation
            );
        }
    }

    #[test]
    fn start_pos() {
        FenTest {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            depth: 4,
            actual_moves: 197_281,
            explanation: "Starting Position",
        }
        .test();
    }

    #[test]
    fn pos_wiki() {
        FenTest {
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0",
            depth: 3,
            actual_moves: 97_862,
            explanation: "Pos 2",
        }
        .test();
        FenTest {
            fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            depth: 4,
            actual_moves: 43_238,
            explanation: "Pos 3",
        }
        .test();
        FenTest {
            fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            depth: 3,
            actual_moves: 9_467,
            explanation: "Pos 4",
        }
        .test();
        FenTest {
            fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            depth: 3,
            actual_moves: 62_379,
            explanation: "Pos 5",
        }
        .test();
        FenTest {
            fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            depth: 3,
            actual_moves: 89_890,
            explanation: "Pos 6",
        }
        .test();
    }

    #[test]
    fn en_passant_test() {
        FenTest {
            fen: "4k3/1ppp4/8/2P2PPp/p1p2Pp1/8/1PPP4/4K3 b - f3 0 1",
            depth: 4,
            actual_moves: 28_127,
            explanation: "General EP position",
        }
        .test();
        FenTest {
            fen: "7k/8/8/K1pP3r/8/8/8/8 w - c6 0 1",
            depth: 5,
            actual_moves: 44_848,
            explanation: "EP with double pin",
        }
        .test();
        FenTest {
            fen: "7k/8/8/3Pp3/5K2/8/8/8 w - e6 0 1",
            depth: 1,
            actual_moves: 9,
            explanation: "EP to take checker",
        }
        .test();
        FenTest {
            fen: "7k/8/2r3K1/3Pp3/8/8/8/8 w - e6 0 1",
            depth: 1,
            actual_moves: 7,
            explanation: "EP to block check",
        }
        .test();
    }

    #[test]
    fn complicated_positions() {
        FenTest {
            fen: "r3k2r/1bp2pP1/5n2/1P1Q4/1pPq4/5N2/1B1P2p1/R3K2R b KQkq c3 0 1",
            depth: 3,
            actual_moves: 113_742,
            explanation: "Complicated Midgame",
        }
        .test();
        FenTest {
            fen: "k7/4r3/1b5q/2Q1Pp2/5R2/1rR1KP1q/3RBN2/2q1q1b1 w - - 0 1",
            depth: 4,
            actual_moves: 360_098,
            explanation: "Pin Position",
        }
        .test();
        FenTest {
            fen: "1nnn1b2/2k3rR/2q1rB2/8/4R2Q/8/4N1K1/8 w - - 0 1",
            depth: 4,
            actual_moves: 690_360,
            explanation: "Check Position",
        }
        .test();
    }

    #[test]
    fn castle_tests() {
        FenTest {
            fen: "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            depth: 2,
            actual_moves: 568,
            explanation: "Can Castle",
        }
        .test();
        FenTest {
            fen: "r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1",
            depth: 2,
            actual_moves: 482,
            explanation: "Cannot Castle without rights",
        }
        .test();
        FenTest {
            fen: "r3k2r/8/8/4q1Q1/8/8/8/R3K2R w KQkq - 0 1",
            depth: 2,
            actual_moves: 218,
            explanation: "Cannot castle from check",
        }
        .test();
        FenTest {
            fen: "r3k2r/6p1/5Q2/8/8/3q4/2P5/R3K2R w KQkq - 0 1",
            depth: 2,
            actual_moves: 1_660,
            explanation: "Cannot castle trough check",
        }
        .test();
        FenTest {
            fen: "r3k2r/8/8/2q3Q1/8/8/8/R3K2R w KQkq - 0 1",
            depth: 2,
            actual_moves: 1_588,
            explanation: "Cannot castle into check",
        }
        .test();
        FenTest {
            fen: "r3k2r/8/1q6/R6r/8/8/7Q/R3K2R w KQkq - 0 1",
            depth: 2,
            actual_moves: 1_899,
            explanation: "Can castle if rook or b-file is attacked",
        }
        .test();
        FenTest {
            fen: "rN2k1br/8/8/8/8/8/8/R1N1Kb1R b KQkq - 0 1",
            depth: 2,
            actual_moves: 832,
            explanation: "Cant castle through pieces",
        }
        .test();
        FenTest {
            fen: "r7/8/2b3b1/6B1/8/8/1k6/R3K2R w KQ - 0 1",
            depth: 4,
            actual_moves: 940_814,
            explanation: "Weird Castle Position",
        }
        .test();
    }
}

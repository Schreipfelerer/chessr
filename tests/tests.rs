#[cfg(test)]
mod full_tests {
    use std::sync::{Arc, atomic::AtomicBool};

    use chessr::{
        board::{BoardState, Move, Sq64},
        search::{TranspositionTable, iterative_deepening},
    };

    #[test]
    fn avoid_draw() {
        let mut bs =
            BoardState::from_fen("8/Q4ppk/b1r1p2p/p5r1/P2P1NPq/2P1PP2/5R1P/R5K1 b - - 0 32").unwrap();
        // moves h6h5 f4g2 h4h3 g2f4 h3h4 f4g2 h4h3
        bs.make_move(Move::new(Sq64(0o57), Sq64(0o47)));
        bs.make_move(Move::new(Sq64(0o35), Sq64(0o16)));
        bs.make_move(Move::new(Sq64(0o37), Sq64(0o27)));
        bs.make_move(Move::new(Sq64(0o16), Sq64(0o35)));
        bs.make_move(Move::new(Sq64(0o27), Sq64(0o37)));
        bs.make_move(Move::new(Sq64(0o35), Sq64(0o16)));
        bs.make_move(Move::new(Sq64(0o37), Sq64(0o27)));

        let mut tt = TranspositionTable::default();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let (m, _) = iterative_deepening(&mut bs, 4, None, &stop_flag, &mut tt);

        assert_ne!(m, Move::new(Sq64(0o16), Sq64(0o35)));
    }
}

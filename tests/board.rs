#[cfg(test)]
mod board_tests {
    use chessr::board::{BoardState, Color, Sq64};

    #[test]
    fn fen_err_test() {
        assert!(BoardState::from_fen("").is_err(), "Empty String");
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .is_ok(),
            "Starting Pos"
        );
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKRBNR w KQkq - 0 1")
                .is_err(),
            "Row too long"
        );
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKRBN/8 w KQkq - 0 1")
                .is_err(),
            "Too many Rows"
        );
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1").is_err(),
            "Too little Rows"
        );
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPEP/RNBQKRBN w KQkq - 0 1")
                .is_err(),
            "unknown Piece"
        );
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPEP/RNBQKRBN x KQkq - 0 1")
                .is_err(),
            "unknown colors turn"
        );
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPEP/RNBQKRBN w Kkkq - 0 1")
                .is_err(),
            "castle_rights double"
        );
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPEP/RNBQKRBN w Klkq - 0 1")
                .is_err(),
            "castle_rights unknown"
        );
        assert!(
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPEP/RNBQKRBN w KQkq -").is_err(),
            "too little pieces"
        );
    }
    #[test]
    fn fen_flag_test() {
        let board =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        assert_eq!(
            board.state_info.active_color,
            Color::White,
            "active color"
        );
        assert_eq!(board.state_info.ep_square, None, "ep_square none");
        assert_eq!(board.state_info.half_move_clock, 0, "Half Move Clock");
        assert_eq!(board.state_info.full_move_number, 1, "Full moves");
        assert_eq!(
            board.state_info.castle_rights, 0b00001111,
            "castle_rights"
        );

        let board =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b - e6 0 1")
                .unwrap();
        assert_eq!(
            board.state_info.active_color,
            Color::Black,
            "active color"
        );
        assert_eq!(board.state_info.ep_square, Some(Sq64(44)), "ep_square e6");
        assert_eq!(
            board.state_info.castle_rights, 0,
            "castle_rights"
        );
    }
}

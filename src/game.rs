use std::fmt;
use std::str::FromStr;
use std::collections::HashMap;
use chess::{Board, MoveGen, ChessMove, BoardStatus, Square, Piece, Color};

#[derive(Clone, Debug)]
pub struct Game {
    board: Board,
    positions: HashMap<u64, u32>,
}



#[derive(Debug, PartialEq)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw
}

impl Game {
    pub fn new() -> Self {
        // Default board is the standard chess starting position
        let mut g = Game {
            board: Board::default(),
            positions: HashMap::new(),
        };
        g.increment_position_count();
        g
    }

    pub(crate) fn get_hash(&self) -> u64 {
        self.board.get_hash()
    }


    pub fn make_move(&mut self, move_str: &str) -> Result<Self, String> {
        let parsed_move = self.parse_move(move_str)?;
        let new_board = self.board.make_move_new(parsed_move);

        self.board = new_board;
        self.increment_position_count();

        Ok(Self {
            board: self.board,
            positions: self.positions.clone(),
        })
    }

    pub fn is_threefold_repetition(&self) -> bool {
        self.positions.values().any(|&count| count >= 3)
    }

    pub fn is_terminal(&self) -> bool {
        // If threefold repetition detected, it's terminal (draw)
        if self.is_threefold_repetition() ||
            has_insufficient_material(&self.board) {
            return true;
        }

        let status = self.board.status();
        if status != chess::BoardStatus::Ongoing {
            return true;
        }

        // Check if no legal moves and not in check => stalemate
        if MoveGen::new_legal(&self.board).count() == 0 {
            return true
        }

        false
    }

    pub fn legal_moves(&self) -> Vec<String> {
        if self.is_terminal() {
            return Vec::new();
        }
        // Generate all legal moves from current position
        let movegen = MoveGen::new_legal(&self.board);
        movegen.map(|m| m.to_string()).collect()
    }

    fn parse_move(&self, move_str: &str) -> Result<ChessMove, String> {
        if move_str.len() < 4 {
            return Err("Move string too short".to_string());
        }

        let from_str = &move_str[0..2];
        let to_str = &move_str[2..4];

        let from_square = Square::from_str(from_str)
            .map_err(|_| format!("Invalid from-square: {}", from_str))?;
        let to_square = Square::from_str(to_str)
            .map_err(|_| format!("Invalid to-square: {}", to_str))?;

        let promotion_piece = if move_str.len() > 4 {
            // The fifth character in the move_str might represent the promotion piece.
            // For example, 'q' for queen, 'r' for rook, 'b' for bishop, 'n' for knight.
            match &move_str[4..5] {
                "q" => Some(Piece::Queen),
                "r" => Some(Piece::Rook),
                "b" => Some(Piece::Bishop),
                "n" => Some(Piece::Knight),
                _ => return Err(format!("Invalid promotion piece: {}", &move_str[4..5])),
            }
        } else {
            None
        };

        let mv = ChessMove::new(from_square, to_square, promotion_piece);
        if self.board.legal(mv) {
            Ok(mv)
        } else {
            Err(format!("Illegal move: {}", move_str))
        }
    }

    pub fn current_player(&self) -> &str {
        if self.board.side_to_move() == chess::Color::White {
            "White"
        } else {
            "Black"
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        match fen.parse::<Board>() {
            Ok(board) => {
                let mut game = Game {
                    board,
                    positions: HashMap::new(),
                };
                game.increment_position_count();
                Ok(game)
            },
            Err(e) => Err(format!("Invalid FEN: {}", e))
        }
    }

    fn increment_position_count(&mut self) {
        let key = self.board.get_hash();
        *self.positions.entry(key).or_insert(0) += 1;
    }

    pub fn encode(&self) -> Vec<f32> {
        let mut encoded = Vec::with_capacity(8*8*6);

        // Match Python indexing: row=0 = rank0 (a1 row), row=7 = rank7 (a8 row)
        // column=0 = file a, column=7 = file h
        for row in 0..8 {
            for column in 0..8 {
                let sq = chess::Square::make_square(
                    chess::Rank::from_index(row),
                    chess::File::from_index(column)
                );
                let piece_vec = self.encode_piece(sq);
                encoded.extend_from_slice(&piece_vec);
            }
        }

        encoded
    }

    fn encode_piece(&self, sq: chess::Square) -> [f32; 6] {
        if let Some(piece) = self.board.piece_on(sq) {
            let color = self.board.color_on(sq).unwrap();
            match piece {
                chess::Piece::Pawn => {
                    if color == chess::Color::White { [1.0,0.0,0.0,0.0,0.0,0.0] }
                    else { [-1.0,0.0,0.0,0.0,0.0,0.0] }
                }
                chess::Piece::Bishop => {
                    if color == chess::Color::White { [0.0,1.0,0.0,0.0,0.0,0.0] }
                    else { [0.0,-1.0,0.0,0.0,0.0,0.0] }
                }
                chess::Piece::Knight => {
                    if color == chess::Color::White { [0.0,0.0,1.0,0.0,0.0,0.0] }
                    else { [0.0,0.0,-1.0,0.0,0.0,0.0] }
                }
                chess::Piece::Rook => {
                    if color == chess::Color::White { [0.0,0.0,0.0,1.0,0.0,0.0] }
                    else { [0.0,0.0,0.0,-1.0,0.0,0.0] }
                }
                chess::Piece::Queen => {
                    if color == chess::Color::White { [0.0,0.0,0.0,0.0,1.0,0.0] }
                    else { [0.0,0.0,0.0,0.0,-1.0,0.0] }
                }
                chess::Piece::King => {
                    if color == chess::Color::White { [0.0,0.0,0.0,0.0,0.0,1.0] }
                    else { [0.0,0.0,0.0,0.0,0.0,-1.0] }
                }
            }
        } else {
            [0.0; 6]
        }
    }

    pub fn get_game_result(&self) -> Option<GameResult> {
        if !self.is_terminal() {
            return None;
        }

        // Check for threefold repetition
        if self.is_threefold_repetition() {
            return Some(GameResult::Draw);
        }

        // Check for insufficient material
        if has_insufficient_material(&self.board) {
            return Some(GameResult::Draw);
        }

        // If there are no legal moves
        if MoveGen::new_legal(&self.board).count() == 0 {
            // If the current player is in check, it's checkmate
            if self.board.checkers().popcnt() > 0 {
                // If White is in check, Black wins and vice versa
                return Some(if self.board.side_to_move() == Color::White {
                    GameResult::BlackWin
                } else {
                    GameResult::WhiteWin
                });
            } else {
                // Stalemate
                return Some(GameResult::Draw);
            }
        }

        Some(GameResult::Draw) // Default to draw for other terminal positions
    }

    pub fn result_value(&self) -> f32 {
        let status = self.get_game_result();
        match status {
            //white win
            Some(GameResult::WhiteWin) => 1.0,
            Some(GameResult::BlackWin) => -1.0,
            Some(GameResult::Draw) => 0.0,
            _ => 0.0
        }
    }

}

fn has_insufficient_material(board: &chess::Board) -> bool {
    use chess::Piece;
    // Count pieces by type
    let mut white_pieces = Vec::new();
    let mut black_pieces = Vec::new();

    for sq in chess::ALL_SQUARES {
        if let Some(piece) = board.piece_on(sq) {
            let color = board.color_on(sq).unwrap();
            if color == chess::Color::White {
                white_pieces.push(piece);
            } else {
                black_pieces.push(piece);
            }
        }
    }

    // Combine all pieces for easier logic
    let all_pieces: Vec<chess::Piece> = white_pieces.iter().chain(black_pieces.iter()).copied().collect();

    // Check if any piece that can generate mate scenarios exists
    // Pawns, Queens, and Rooks always mean sufficient material (unless no other conditions are met).
    if all_pieces.iter().any(|&p| p == Piece::Pawn || p == Piece::Rook || p == Piece::Queen) {
        return false;
    }

    // Now we have only Kings, possibly Bishops, and/or Knights
    let num_knights = all_pieces.iter().filter(|&&p| p == Piece::Knight).count();
    let num_bishops = all_pieces.iter().filter(|&&p| p == Piece::Bishop).count();
    // We know there are exactly two kings on the board (if the board is valid).

    // Cases:
    // 1. K vs K
    if all_pieces.len() == 2 {
        return true; // Only kings
    }

    // 2. K+N vs K
    if num_knights == 1 && num_bishops == 0 && all_pieces.len() == 3 {
        return true;
    }
    // K+N vs K+N is also insufficient (no forced mate), but that's a rare known theoretical position.
    // If you'd like to consider K+N vs K+N as insufficient:
    if num_knights == 2 && num_bishops == 0 && all_pieces.len() == 4 {
        return true;
    }

    // 3. K+B vs K
    if num_bishops == 1 && num_knights == 0 && all_pieces.len() == 3 {
        return true;
    }

    // 4. K+B vs K+B both on same color
    if num_bishops == 2 && num_knights == 0 && all_pieces.len() == 4 {
        // Find the squares of the bishops
        let mut bishop_squares = Vec::new();
        for sq in chess::ALL_SQUARES {
            if let Some(piece) = board.piece_on(sq) {
                if piece == Piece::Bishop {
                    bishop_squares.push(sq);
                }
            }
        }

        // Check bishop color complexes
        // Light squares have (file + rank) even sum, dark squares have odd.
        let colors: Vec<bool> = bishop_squares.iter().map(|&sq| {
            let file = sq.get_rank().to_index();
            let rank = sq.get_file().to_index();
            ((file + rank) % 2) == 0
        }).collect();

        // If all bishops are on the same color squares
        if colors.iter().all(|&c| c == colors[0]) {
            return true;
        }
    }

    // If none of the above conditions met, assume material is sufficient
    false
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_initial_moves_count() {
        let game = Game::new();
        let legal_moves = game.legal_moves();
        // In standard chess, from the initial position, there are 20 legal moves.
        assert_eq!(legal_moves.len(), 20, "Should have exactly 20 moves at the start.");
    }

    #[test]
    fn test_make_move() {
        let mut game = Game::new();
        let initial_moves = game.legal_moves();

        // "e2e4" is a known standard chess opening move.
        assert!(initial_moves.contains(&"e2e4".to_string()), "Initial moves must include 'e2e4'.");

        // Make the move and get a new game state.
        game = game.make_move("e2e4").expect("Move e2e4 should be legal and succeed.");

        let new_moves = game.legal_moves();
        // After the move, the position should no longer be the same as the initial position.
        // We can test this in several ways, but one simple test is that the set of legal moves changes.
        assert_ne!(initial_moves, new_moves, "Legal moves should differ after making a move.");
    }

    #[test]
    fn test_checkmate_detection() {
        let mut game = Game::new();
        // 1. White: f2f3
        game = game.make_move("f2f3").expect("f2f3 should be legal");
        assert!(!game.is_terminal(), "Game should not be terminal after f2f3.");

        // 2. Black: e7e5
        game = game.make_move("e7e5").expect("e7e5 should be legal");
        assert!(!game.is_terminal(), "Game should not be terminal after e7e5.");

        // 3. White: g2g4
        game = game.make_move("g2g4").expect("g2g4 should be legal");
        assert!(!game.is_terminal(), "Game should not be terminal after g2g4.");

        // 4. Black: d8h4 (Checkmate move)
        game = game.make_move("d8h4").expect("d8h4 should be legal");
        assert!(game.is_terminal(), "Game should be terminal (checkmate) after d8h4.");
    }

    #[test]
    fn test_current_player_switches() {
        let mut game = Game::new();
        let initial_moves = game.legal_moves();

        // Initial position: White to move.
        let current_player = game.current_player();
        assert_eq!(current_player, "White", "Game should start with White to move.");

        // Make one White move, e.g. "e2e4".
        assert!(initial_moves.contains(&"e2e4".to_string()), "Initial moves must include 'e2e4'.");
        game = game.make_move("e2e4").expect("e2e4 should be legal");

        // Now it should be Black's turn.
        let current_player = game.current_player();
        assert_eq!(current_player, "Black", "After White moves, it should be Black's turn.");

        // Make one Black move, e.g. "e7e5".
        let moves_after_white = game.legal_moves();
        assert!(moves_after_white.contains(&"e7e5".to_string()), "Moves must include 'e7e5' for Black.");
        game = game.make_move("e7e5").expect("e7e5 should be legal");

        // Now it should be White's turn again.
        let current_player = game.current_player();
        assert_eq!(current_player, "White", "After Black moves, it should be White's turn again.");
    }

    #[test]
    fn test_stalemate_detection_korchnoi_karpov() {
        let stalemate_fen = "7k/5Q2/8/8/8/8/8/7K b - - 0 1";
        let game = Game::from_fen(stalemate_fen)
            .expect("Should be able to create a board from a legal stalemate FEN");

        // Print the initial game state for debugging
        println!("{:?}", game);

        assert!(game.is_terminal(), "Game should be terminal due to stalemate.");
        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), 0, "No legal moves should be available in stalemate.");
    }

    #[test]
    fn test_insufficient_material() {
        let fen = "8/8/8/8/8/6K1/8/2k5 w - - 0 1";
        let game = Game::from_fen(fen).expect("Should parse fen for kings only");
        println!("{:?}", game);
        // Now you must ensure your `is_terminal()` recognizes insufficient material as a terminal state.
        // Check the crate's `BoardStatus` enum. Insufficient material should lead to a `Draw` status.
        assert!(game.is_terminal(), "Game should be terminal due to insufficient material.");
        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), 0, "No legal moves expected due to insufficient material.");

    }

    #[test]
    fn test_threefold_repetition() {
        let mut game = Game::new();
        // Start from initial position.
        // Sequence of moves:
        // 1. White: g1f3 Black: b8c6
        // 2. White: f3g1 Black: c6b8  (position repeats)
        // 3. White: g1f3 Black: b8c6
        // 4. White: f3g1 Black: c6b8  (position repeats again, now third time total)

        let moves = [
            "g1f3", "b8c6",   // first cycle
            "f3g1", "c6b8",   // back to start (second occurrence)
            "g1f3", "b8c6",   // third cycle begins
            "f3g1", "c6b8",   // back to start (third occurrence)
        ];

        for mov in &moves {
            game = game.make_move(mov).expect("Moves should be legal");
        }

        assert!(game.is_terminal(), "Game should be terminal due to threefold repetition.");
        assert!(game.is_threefold_repetition(), "Threefold repetition should be detected.");
        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), 0, "No legal moves should be available due to draw.");
    }

    #[test]
    fn test_fifty_move_rule() {
        let mut game = Game::new();
        // We'll just move knights back and forth without any pawns or captures.
        // For simplicity, start from the initial position and just shuffle knights.
        //
        // Initial position halfmove clock is 0.
        // Each full move (one by White and one by Black) increments this by 2 if no captures/pawn moves occur.
        // After 50 full moves (100 half-moves), we should detect a draw.

        // Make a sequence of 50 moves by White and Black that don't involve pawns or captures.
        // For example, knights can be moved out and back to their squares:
        // White: g1f3, Black: b8c6
        // White: f3g1, Black: c6b8
        // Repeat this pattern 25 times. That gives us 100 half-moves.
        assert!(game.is_terminal() == false, "Initial position should not be terminal.");
        for _ in 0..25 {
            game = game.make_move("g1f3").expect("Move should be legal");
            game = game.make_move("b8c6").expect("Move should be legal");
            game = game.make_move("f3g1").expect("Move should be legal");
            game = game.make_move("c6b8").expect("Move should be legal");
        }

        assert!(game.is_terminal(), "Game should be terminal due to the fifty-move rule draw.");
        let legal_moves = game.legal_moves();
        println!("{:?}", game);
        assert_eq!(legal_moves.len(), 0, "No legal moves should be available because the game is a draw.");
    }

    #[test]
    fn test_en_passant() {
        let fen = "rnbqkbnr/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 1";
        // This FEN represents a position after White has played e4 and Black has replied with ...f5,
        // giving White the opportunity to capture en passant on f5-e6.

        // Create the game from this fen:
        let mut game = Game::from_fen(fen).expect("Should parse fen");

        // En passant should be legal here: White pawn on e4 can capture on f5.
        let legal_moves = game.legal_moves();
        assert!(legal_moves.contains(&"e4f5".to_string()), "En passant capture should be legal.");

        // Make the en passant capture:
        game = game.make_move("e4f5").expect("En passant should work.");

        // After this move, verify that the board state has the white pawn on f5 and the black pawn removed.
        // (Implement assertions as needed once you have accessors for board state.)
    }

    #[test]
    fn test_promotion() {
        // Set up a position where White's pawn on seventh rank is ready to promote:
        // For instance, a White pawn on g7 and it's White to move.
        let fen = "8/6P1/8/8/8/8/8/4K2k w - - 0 1";
        let mut game = Game::from_fen(fen).expect("Should parse fen");

        // Check that the promotion move is legal (g7-g8=Q)
        let legal_moves = game.legal_moves();
        //print the legal moves
        println!("{:?}", legal_moves);
        assert!(legal_moves.contains(&"g7g8q".to_string()), "Promotion to queen should be legal.");
        assert!(legal_moves.contains(&"g7g8r".to_string()), "Promotion to rook should be legal.");
        assert!(legal_moves.contains(&"g7g8b".to_string()), "Promotion to bishop should be legal.");
        assert!(legal_moves.contains(&"g7g8n".to_string()), "Promotion to knight should be legal.");

        // Choose one promotion, say to a queen:
        game = game.make_move("g7g8q").expect("Pawn promotion to queen should be allowed.");

    }
    #[test]
    fn test_castling() {
        // Set up a FEN where White can legally castle kingside.
        // For simplicity: white king on e1, white rook on h1, no pieces in between,
        // black pieces placed in a way that doesn't put white in check.
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let mut game = Game::from_fen(fen).expect("Should parse fen");

        // In this position, White should be able to castle kingside (e1g1) and queenside (e1c1).
        let legal_moves = game.legal_moves();
        assert!(legal_moves.contains(&"e1g1".to_string()), "White should be able to castle kingside.");
        assert!(legal_moves.contains(&"e1c1".to_string()), "White should be able to castle queenside.");

        // Make a castling move, say kingside:
        game = game.make_move("e1g1").expect("Should be able to castle kingside");


    }

/*
    #[test]
    fn test_castling_rights_lost_after_king_moves() {
        // Standard starting position, White can castle both sides initially
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut game = Game::from_fen(fen).expect("Should parse fen");

        // Initially, White should be able to castle kingside and queenside.
        let legal_moves = game.legal_moves();
        assert!(legal_moves.contains(&"e1g1".to_string()), "White should be able to castle kingside initially.");
        assert!(legal_moves.contains(&"e1c1".to_string()), "White should be able to castle queenside initially.");

        // Move the king forward and then back.
        // e1e2 is a legal move, and then we move back e2e1.
        // Even though the king returns to e1, it has moved, so castling rights are lost.
        game = game.make_move("e1e2").expect("King should be able to move up one rank.");
        game = game.make_move("e7e5").expect("Black makes a random move to pass the turn.");
        game = game.make_move("e2e1").expect("King moves back to original square.");

        // Now check that castling rights are gone.
        let legal_moves_after = game.legal_moves();
        assert!(!legal_moves_after.contains(&"e1g1".to_string()), "White should no longer be able to castle kingside after king has moved.");
        assert!(!legal_moves_after.contains(&"e1c1".to_string()), "White should no longer be able to castle queenside after king has moved.");
    }

    */

    #[test]
    fn test_game_results() {
        // Test checkmate (Scholar's mate)
        let mut game = Game::new();
        game.make_move("e2e4").unwrap();
        game.make_move("e7e5").unwrap();
        game.make_move("f1c4").unwrap();
        game.make_move("b8c6").unwrap();
        game.make_move("d1h5").unwrap();
        game.make_move("g8f6").unwrap();
        game.make_move("h5f7").unwrap();

        assert!(game.is_terminal());
        assert_eq!(game.get_game_result(), Some(GameResult::WhiteWin));


        // Test threefold repetition
        let mut game = Game::new();
        // Make moves that repeat the position three times
        game.make_move("g1f3").unwrap();
        game.make_move("g8f6").unwrap();
        game.make_move("f3g1").unwrap();
        game.make_move("f6g8").unwrap();
        game.make_move("g1f3").unwrap();
        game.make_move("g8f6").unwrap();
        game.make_move("f3g1").unwrap();
        game.make_move("f6g8").unwrap();

        assert!(game.is_terminal());
        assert_eq!(game.get_game_result(), Some(GameResult::Draw));
    }

    #[test]
    fn test_encoding_initial_position() {
        // Standard initial chess position:
        // 8 | r n b q k b n r
        // 7 | p p p p p p p p
        // 6 | . . . . . . . .
        // 5 | . . . . . . . .
        // 4 | . . . . . . . .
        // 3 | . . . . . . . .
        // 2 | P P P P P P P P
        // 1 | R N B Q K B N R
        //     a b c d e f g h
        //
        // Using the indexing scheme row=0 => rank1 (bottom), column=0 => file a
        // So encoded[0..6] corresponds to a1, [6..12] to b1, etc.

        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let game = Game::from_fen(fen).expect("Should parse FEN");

        let encoded = game.encode();
        assert_eq!(encoded.len(), 384, "Encoded vector should have length 384 (8*8*6).");

        // Check a few known squares:
        // a1 should have a White Rook: [0,0,0,1,0,0]
        let a1_index = 0; // row=0, col=0, channels=6
        let a1_slice = &encoded[a1_index..a1_index+6];
        assert_eq!(a1_slice, &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0], "a1 should be a white rook");

        // e1 should have a White King: e1 = row=0, col=4
        // Each square has 6 floats, so index = (row*8 + column)*6
        // index = (0*8 + 4)*6 = 24
        let e1_index = 24;
        let e1_slice = &encoded[e1_index..e1_index+6];
        assert_eq!(e1_slice, &[0.0,0.0,0.0,0.0,0.0,1.0], "e1 should be a white king");

        // a8 should have a Black Rook: a8 = row=7 (top), col=0
        // index = (7*8 + 0)*6 = 56*6 = 336
        let a8_index = 336;
        let a8_slice = &encoded[a8_index..a8_index+6];
        assert_eq!(a8_slice, &[0.0,0.0,0.0,-1.0,0.0,0.0], "a8 should be a black rook");

        // e8 should have a Black King: e8 = row=7, col=4
        // index = (7*8+4)*6 = (56+4)*6 = 60*6 = 360
        let e8_index = 360;
        let e8_slice = &encoded[e8_index..e8_index+6];
        assert_eq!(e8_slice, &[0.0,0.0,0.0,0.0,0.0,-1.0], "e8 should be a black king");

        // Check an empty square, e.g., e4 is empty in the starting position:
        // e4 = row=3 (since row=0 = rank1, row=3 = rank4), col=4
        // index = (3*8+4)*6 = (24+4)*6 = 28*6 = 168
        let e4_index = 168;
        let e4_slice = &encoded[e4_index..e4_index+6];
        assert_eq!(e4_slice, &[0.0,0.0,0.0,0.0,0.0,0.0], "e4 should be empty");

        // If we reach here, the test passes for these checks.
    }
}
use std::fmt;
use std::str::FromStr;
use chess::{Board, MoveGen, ChessMove, BoardStatus, Square};


pub struct Game {
    board: Board,
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the board using its Display representation
        write!(
            f,
            "Game {{\nBoard:\n{}\nSide to Move: {:?}\nStatus: {:?}\n}}",
            self.board,                       // Board layout
            self.board.side_to_move(),        // Side to move (White or Black)
            self.board.status()               // Current status (Ongoing, Stalemate, Checkmate)
        )
    }
}

impl Game {
    pub fn new() -> Self {
        // Default board is the standard chess starting position
        Game {
            board: Board::default(),
        }
    }

    pub fn is_terminal(&self) -> bool {
        let status = self.board.status();

        if status == chess::BoardStatus::Ongoing {
            // Manually check for legal moves as a fallback
            let legal_moves = MoveGen::new_legal(&self.board);
            if legal_moves.count() == 0 {
                let king_square = self.board.king_square(self.board.side_to_move());
                // Use bitwise operation to check if the king is in check
                if self.board.checkers() & chess::BitBoard::from_square(king_square) == chess::BitBoard::new(0) {
                    println!("Detected stalemate manually.");
                    return true; // Stalemate detected
                }
            }
            return false; // Game is ongoing
        }

        // Return true if the status is Checkmate or Stalemate
        status == chess::BoardStatus::Stalemate || status == chess::BoardStatus::Checkmate
    }

    pub fn legal_moves(&self) -> Vec<String> {
        // Generate all legal moves from current position
        let movegen = MoveGen::new_legal(&self.board);
        movegen.map(|m| m.to_string()).collect()
    }

    pub fn make_move(&mut self, move_str: &str) -> Result<Self, String> {
        // Parse the move string
        // "e2e4" -> from = e2, to = e4
        // chess::ChessMove::new expects a from-square and a to-square.
        let parsed_move = self.parse_move(move_str)?;

        // Make the move on a copy of the current board
        let new_board = self.board.make_move_new(parsed_move);

        // Update self
        self.board = new_board;

        Ok(Game {
            board: self.board
        })
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

        let mv = ChessMove::new(from_square, to_square, None);
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
            Ok(board) => Ok(Game { board }),
            Err(e) => Err(format!("Invalid FEN: {}", e))
        }
    }
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
        let stalemate_fen = "8/5BK1/8/8/8/8/p7/k7 b - - 0 1";
        let game = Game::from_fen(stalemate_fen)
            .expect("Should be able to create a board from a legal stalemate FEN");

        // Print the initial game state for debugging
        println!("{:?}", game);

        assert!(game.is_terminal(), "Game should be terminal due to stalemate.");
        let legal_moves = game.legal_moves();
        assert_eq!(legal_moves.len(), 0, "No legal moves should be available in stalemate.");
    }

}
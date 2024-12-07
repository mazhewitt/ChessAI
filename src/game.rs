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
            // If we rely on our custom insufficient material check:
            if has_insufficient_material(&self.board) {
                return true;
            }
            // Also check if there are no legal moves and not in check (then it's stalemate)
            if MoveGen::new_legal(&self.board).count() == 0 {
                return true; // Stalemate
            }
            return false;
        }
        // If status is already Checkmate or Stalemate (if the crate sets it, e.g., threefold repetition or 50-move rule)
        status != chess::BoardStatus::Ongoing
    }

    pub fn legal_moves(&self) -> Vec<String> {
        if self.is_terminal() {
            return Vec::new();
        }
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

}
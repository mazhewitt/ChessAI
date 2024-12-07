use ChessAI::game::Game; // We'll create this soon.

#[test]
fn test_initial_board_state() {
    let game = Game::new();
    assert_eq!(game.is_terminal(), false, "Initial position should not be terminal.");
    let legal_moves = game.legal_moves();
    assert!(legal_moves.len() > 0, "Initial position should have legal moves.");
}
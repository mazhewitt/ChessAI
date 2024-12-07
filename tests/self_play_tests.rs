use ChessAI::game::Game;

#[test]
fn test_self_play_single_game() {
    let mut game = Game::new();

    // A mock model that returns equal probability for all legal moves and 0.0 value.
    let mock_model = MockModel::new();

    // Your MCTS struct, which uses the model:
    let mut mcts = Mcts::new(&mock_model);

    let mut states_and_policies = Vec::new();
    let mut final_value = 0.0;

    while !game.is_terminal() {
        // Run MCTS to get a policy
        let policy = mcts.search(&game);

        // Store state and policy
        let encoded_state = game.encode(); // Implement a method that returns a tensor/array
        states_and_policies.push((encoded_state, policy.clone()));

        // Choose a move based on the policy
        let action = choose_action(&policy); // some method to pick the move index
        let move_str = action_to_move_str(action); // convert index back to algebraic notation
        game = game.make_move(&move_str).expect("Move should be legal");
    }

    // Once terminal, get final value from perspective of the starting player
    final_value = game.final_value(); // +1 for win, 0 for draw, -1 for loss, etc.

    // Check that we have a sequence of states/policies and a final value
    assert!(!states_and_policies.is_empty(), "Should have recorded states and policies.");
    // Check final value is something sensible given the outcome.
    assert!(final_value == 1.0 || final_value == 0.0 || final_value == -1.0);
}
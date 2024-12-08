use ChessAI::game::Game;


#[test]
fn test_real_model_evaluation() {
    let game = Game::new();
    let model = RealChessModel::new();
    let input_tensor = tch::Tensor::from_slice(&game.encode());
    let output = model.evaluate(&game);

    assert!(output.value.abs() <= 1.0, "Model evaluation value should be within [-1, 1].");
    assert_eq!(output.policy.len(), game.legal_moves().len(), "Model policy output length should match the number of legal moves.");
}

#[test]
fn test_mcts_with_real_model() {
    let game = Game::new();
    let model = RealChessModel::new();
    let mut mcts = MCTSManager::new(game.clone(), model);

    assert!(mcts.playout_n_parallel(1000, 4).is_ok(), "MCTS playouts should run successfully.");
    let best_move = mcts.best_move();
    assert!(best_move.is_some(), "MCTS should return a best move.");
}
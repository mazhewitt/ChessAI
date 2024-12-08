use std::sync::Arc;
use mcts::{Evaluator, GameState, SearchHandle, MCTS};
use crate::game::Game;
use mcts::transposition_table::{ApproxTable, TranspositionHash};
use mcts::tree_policy::UCTPolicy;
use tch::Tensor;
use crate::chess_ai_model::ChessAIModel;

#[derive(Clone)]
pub struct ChessMCTSState {
    game: Game,
}

impl ChessMCTSState {
    pub fn new(game: Game) -> Self {
        ChessMCTSState { game }
    }
}

impl TranspositionHash for ChessMCTSState {
    fn hash(&self) -> u64 {
        self.game.get_hash()
    }
}

impl GameState for ChessMCTSState {
    type Move = String;
    type Player = String;
    type MoveList = Vec<Self::Move>;

    fn current_player(&self) -> Self::Player {
        self.game.current_player().parse().unwrap()
    }

    fn available_moves(&self) -> Self::MoveList {
        self.game.legal_moves()
    }

    fn make_move(&mut self, mov: &Self::Move) {
        self.game.make_move(mov).expect("Move should be legal");
    }
}

pub trait ChessModel: Send + Sync {
    fn evaluate(&self, game: &Game) -> ModelOutput;
}
#[derive(Clone)]
pub struct ModelOutput {
    pub value: f64,        // Position evaluation (-1 to 1)
    pub policy: Vec<f64>,  // Probabilities for each legal move
}



pub struct ChessEvaluator {
    model: Box<dyn ChessModel>,  // Your trained model
}

impl ChessEvaluator {
    fn evaluate_state(&self, state: &ChessMCTSState) -> f64 {
        // Use the existing result_value() function for terminal states
        let terminal_value = state.game.result_value();
        if terminal_value != 0.0 {
            // Convert f32 to f64, since the Evaluator trait uses f64
            return terminal_value as f64;
        }

        // If the state is not terminal, implement additional evaluation logic here
        // For now, we'll use 0.0 as a placeholder
        // Replace this with heuristic evaluations or a neural network call
        0.0
    }
}



#[derive(Default)]
pub struct NodeStats {
    visits: u32,
    total_value: f64,
    mean_value: f64,
}
#[derive(Default)]
pub struct ChessMCTS;

impl MCTS for ChessMCTS {
    type State = ChessMCTSState;
    type Eval = ChessEvaluator;
    type NodeData = NodeStats;
    type ExtraThreadData = ();
    type TreePolicy = UCTPolicy;
    type TranspositionTable = ApproxTable<Self>;

    fn cycle_behaviour(&self) -> mcts::CycleBehaviour<Self> {
        mcts::CycleBehaviour::UseCurrentEvalWhenCycleDetected
    }
}

impl Evaluator<ChessMCTS> for ChessEvaluator {
    type StateEvaluation = ModelOutput;

    fn evaluate_new_state(
        &self,
        state: &ChessMCTSState,
        moves: &Vec<String>,
        _: Option<SearchHandle<ChessMCTS>>,
    ) -> (Vec<()>, ModelOutput) {
        let model_output = self.model.evaluate(&state.game);
        (vec![(); moves.len()], model_output)
    }

    fn evaluate_existing_state(
        &self,
        _state: &ChessMCTSState,
        eval: &ModelOutput,
        handle: SearchHandle<ChessMCTS>,
    ) -> ModelOutput {
        // Use the existing evaluation
        eval.clone()
    }

    fn interpret_evaluation_for_player(&self, eval: &ModelOutput, player: &String) -> i64 {
        let value = if player == "White" {
            eval.value
        } else {
            -eval.value  // Negate value for Black
        };
        (value * 10000.0) as i64
    }
}

pub struct RealChessModel {
    ai_model: Arc<ChessAIModel>,
}

impl RealChessModel {
    pub fn new() -> Self {
        RealChessModel {
            ai_model: Arc::new(ChessAIModel::new()),
        }
    }
    pub fn from_file(filepath: &str) -> Self {
        RealChessModel {
            ai_model: Arc::new(ChessAIModel::from_file(filepath)),
        }
    }
}

impl ChessModel for RealChessModel {
    fn evaluate(&self, game: &Game) -> ModelOutput {
        let input_tensor = Tensor::from_slice(&game.encode());
        let value = self.ai_model.evaluate(&input_tensor);
        // Placeholder for policy vector
        let policy = vec![1.0 / game.legal_moves().len() as f64; game.legal_moves().len()];
        ModelOutput { value, policy }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use mcts::MCTSManager;

    #[test]
    fn test_available_moves() {
        let game = Game::new();
        let state = ChessMCTSState::new(game);
        let moves = state.available_moves();
        assert!(!moves.is_empty(), "There should be available moves.");
    }

    #[test]
    fn test_mcts_search() {
        let game = Game::new();
        let state = ChessMCTSState::new(game);

        let mut mcts = MCTSManager::new(
            state,
            ChessMCTS,
            ChessEvaluator { model: Box::new(MockModel) },
            UCTPolicy::new(0.5),
            ApproxTable::new(1024),
        );

        mcts.playout_n_parallel(1000, 4); // 1,000 playouts with 4 threads

        let best_move = mcts.best_move();
        assert!(best_move.is_some(), "MCTS should find a best move.");
    }

    struct MockModel;

    impl ChessModel for MockModel {
        fn evaluate(&self, game: &Game) -> ModelOutput {
            // Return a mock value and policy
            ModelOutput {
                value: 0.0,  // Mock value, e.g., neutral evaluation
                policy: vec![1.0 / game.legal_moves().len() as f64; game.legal_moves().len()],  // Equal probability for all moves
            }
        }
    }

    #[test]
    fn test_mcts_with_mock_model() {
        let game = Game::new();
        let state = ChessMCTSState::new(game);

        let mut mcts = MCTSManager::new(
            state,
            ChessMCTS,
            ChessEvaluator { model: Box::new(MockModel) },
            UCTPolicy::new(0.5),
            ApproxTable::new(1024),
        );

        mcts.playout_n_parallel(1000, 4); // 1,000 playouts with 4 threads

        let best_move = mcts.best_move();
        // print the best move
        println!("{:?}", best_move);
        assert!(best_move.is_some(), "MCTS should find a best move.");

        // Additional checks can be added here to verify the behavior
    }

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
        let state = ChessMCTSState::new(game);
        let evaluator = ChessEvaluator { model: Box::new(model) };
        let mut mcts = MCTSManager::new(
            state,
            ChessMCTS,
            evaluator,
            UCTPolicy::new(0.5),
            ApproxTable::new(1024),
        );

        mcts.playout_n_parallel(1000, 4);
        let best_move = mcts.best_move();
        assert!(best_move.is_some(), "MCTS should return a best move.");
    }

    #[test]
    fn test_model_save_and_load() {
        let model = RealChessModel::new();
        let filepath = "dummy_model_file";

        // Save the model to a file
        model.ai_model.save_to_file(filepath);

        // Load the model from the file
        let loaded_model = RealChessModel::from_file(filepath);
        let game = Game::new();
        let output = loaded_model.evaluate(&game);

        assert!(output.value.abs() <= 1.0, "Model evaluation value should be within [-1, 1].");
        assert_eq!(output.policy.len(), game.legal_moves().len(), "Model policy output length should match the number of legal moves.");
    }

}
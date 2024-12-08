use mcts::{Evaluator, GameState, SearchHandle, MCTS};
use crate::game::Game;
use mcts::transposition_table::{ApproxTable, TranspositionHash};
use mcts::tree_policy::UCTPolicy;

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

pub trait ChessModel {
    fn evaluate(&self, game: &Game) -> ModelOutput;
}

pub struct ModelOutput {
    pub value: f64,        // Position evaluation (-1 to 1)
    pub policy: Vec<f64>,  // Probabilities for each legal move
}


impl Evaluator<ChessMCTS> for ChessEvaluator {
    type StateEvaluation = ModelOutput;

    fn evaluate_new_state(
        &self,
        state: &ChessMCTSState,
        moves: &Vec<String>,
        handle: Option<SearchHandle<ChessMCTS>>,
    ) -> (Vec<NodeStats>, ModelOutput) {
        let model_output = self.model.evaluate(&state.game);
        let node_data = vec![NodeStats::default(); moves.len()];
        if let Some(h) = handle {
            if let Some(stats) = h.get_data() {
                stats.update(model_output.value);
            }
        }
        (node_data, model_output)
    }

    fn evaluate_existing_state(
        &self,
        _state: &ChessMCTSState,
        eval: &ModelOutput,
        handle: SearchHandle<ChessMCTS>,
    ) -> ModelOutput {
        if let Some(stats) = handle.get_data() {
            stats.update(eval.value);
        }
        eval.clone()
    }

    fn interpret_evaluation_for_player(&self, eval: &f64, player: &String) -> i64 {
        let value = if player == "White" {
            *eval
        } else {
            -(*eval)  // Negate value for Black
        };
        (value * 10000.0) as i64
    }
}

struct ChessEvaluator {
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
struct NodeStats {
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
            ChessEvaluator,
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
            MyMCTS,
            ChessEvaluator { model: Box::new(MockModel) },
            UCTPolicy::new(0.5),
            ApproxTable::new(1024),
        );

        mcts.playout_n_parallel(1000, 4); // 1,000 playouts with 4 threads

        let best_move = mcts.best_move();
        assert!(best_move.is_some(), "MCTS should find a best move.");

        // Additional checks can be added here to verify the behavior
    }
}
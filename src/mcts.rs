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

struct ChessEvaluator;

impl Evaluator<MyMCTS> for ChessEvaluator {
    type StateEvaluation = f64;

    fn evaluate_new_state(
        &self,
        state: &ChessMCTSState,
        moves: &Vec<String>,
        _: Option<SearchHandle<MyMCTS>>,
    ) -> (Vec<()>, f64) {
        let value = self.evaluate_state(state);
        (vec![(); moves.len()], value)
    }

    fn interpret_evaluation_for_player(&self, eval: &f64, player: &String) -> i64 {
        let value = if player == "White" {
            *eval
        } else {
            -(*eval)  // Negate value for Black
        };
        (value * 10000.0) as i64
    }

    fn evaluate_existing_state(
        &self,
        _: &ChessMCTSState,
        eval: &f64,
        _: SearchHandle<MyMCTS>,
    ) -> f64 {
        *eval
    }
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
struct MyMCTS;

impl MCTS for MyMCTS {
    type State = ChessMCTSState;
    type Eval = ChessEvaluator;
    type NodeData = ();
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
            MyMCTS,
            ChessEvaluator,
            UCTPolicy::new(0.5),
            ApproxTable::new(1024),
        );

        mcts.playout_n_parallel(1000, 4); // 1,000 playouts with 4 threads

        let best_move = mcts.best_move();
        assert!(best_move.is_some(), "MCTS should find a best move.");
    }
}

use crate::game::Game;
use crate::mcts::{ChessModel, ModelOutput};
use tch::{nn, nn::Module, Device, Tensor};
use std::sync::{Arc, Mutex};


pub struct ChessAIModel {
    vs: nn::VarStore,
    net: Arc<Mutex<Box<dyn Module + Send>>>,
}

impl ChessAIModel {
    pub fn new() -> Self {
        let vs = nn::VarStore::new(Device::Cpu);
        let net = nn::seq()
            .add(nn::linear(vs.root(), 384, 128, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), 128, 64, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), 64, 1, Default::default()));
        ChessAIModel {
            vs,
            net: Arc::new(Mutex::new(Box::new(net))),
        }
    }

    pub fn evaluate(&self, input: &Tensor) -> f64 {
        let net = self.net.lock().unwrap();
        let output = net.forward(input);
        output.double_value(&[0])
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


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



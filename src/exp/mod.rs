pub mod roadgame;
pub mod pole;
pub mod tictactoe;

use std::collections::HashMap;
use genes;
use nn;

pub trait Experiment {
    fn initial_genome(&self) -> genes::Genome;
    fn node_names(&self) -> HashMap<genes::NodeId, String>;
    fn evaluate(&self, network: &mut nn::Network) -> f64;
}

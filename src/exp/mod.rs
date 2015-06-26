pub mod roadgame;
pub mod pole;
pub mod tictactoe;

use std::collections::HashMap;
use genes;
use nn;
use pop;

pub trait Experiment {
    fn initial_genome(&self) -> genes::Genome;
    fn node_names(&self) -> HashMap<genes::NodeId, String>;
    fn evaluate(&self, network: &mut nn::Network, organisms: &[pop::Organism]) -> f64;
    fn evaluate_to_string(&self, network: &mut nn::Network) -> String;
    fn post_evaluation(&mut self, population: &pop::Population);
}

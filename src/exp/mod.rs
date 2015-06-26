pub mod roadgame;
pub mod pole;
pub mod tictactoe;

use std::collections::HashMap;
use genes;
use nn;
use pop;
use mutation;

pub trait Experiment : Clone {
    fn population_settings(&self) -> pop::Settings;
    fn mutation_settings(&self) -> mutation::Settings;
    fn compat_coefficients(&self) -> genes::CompatCoefficients;

    fn initial_genome(&self) -> genes::Genome;
    fn node_names(&self) -> HashMap<genes::NodeId, String>;

    fn evaluate(&self, network: &mut nn::Network, organisms: &[pop::Organism]) -> f64;
    fn post_evaluation(&mut self, population: &pop::Population);

    fn evaluate_to_string(&self, network: &mut nn::Network) -> String;
}

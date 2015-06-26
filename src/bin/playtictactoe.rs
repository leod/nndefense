extern crate neat;

use std::env;
use std::path::Path;

use neat::genes;
use neat::exp;
use neat::pop;
use neat::exp::Experiment;

fn main() {
    let genome = genes::Genome::load(Path::new(&env::args().nth(1).unwrap()));
    let mut organism = pop::Organism::new(&genome);

    exp::tictactoe::play(&mut exp::tictactoe::InputStrategy, &mut exp::tictactoe::NetworkStrategy { network: &mut organism.network }, true);
}

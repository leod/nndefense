extern crate neat;

use std::env;
use std::path::Path;

use neat::genes;
use neat::exp::tictactoe;
use neat::pop;
use neat::exp::Experiment;

fn main() {
    let genome = genes::Genome::load(Path::new(&env::args().nth(1).unwrap()));
    let mut organism = pop::Organism::new(&genome);

    tictactoe::game::play(&mut tictactoe::strats::InputStrategy, &mut tictactoe::exp::NetworkStrategy { network: &mut organism.network }, true);
}

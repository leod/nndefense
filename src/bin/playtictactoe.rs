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

    println!("BEST vs NETWORK");
    tictactoe::game::play(&mut tictactoe::strats::BestStrategy { forkable: false },
                          &mut tictactoe::exp::NetworkStrategy { network: &mut organism.network }, true);

    println!("");
    println!("NETWORK vs BEST");
    tictactoe::game::play(&mut tictactoe::exp::NetworkStrategy { network: &mut organism.network },
                          &mut tictactoe::strats::BestStrategy { forkable: false }, true);

    println!("");
    println!("FORKABLE vs NETWORK");
    tictactoe::game::play(&mut tictactoe::strats::BestStrategy { forkable: false },
                          &mut tictactoe::exp::NetworkStrategy { network: &mut organism.network }, true);

    println!("");
    println!("NETWORK vs FORKABLE");
    tictactoe::game::play(&mut tictactoe::exp::NetworkStrategy { network: &mut organism.network },
                          &mut tictactoe::strats::BestStrategy { forkable: false }, true);

    println!("");
    println!("");
    println!("");
    println!("PLAY!");
    tictactoe::game::play(&mut tictactoe::exp::NetworkStrategy { network: &mut organism.network },
                          &mut tictactoe::strats::InputStrategy, true);
}

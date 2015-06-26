extern crate getopts;
extern crate neat;

use std::env;
use std::path::Path;

use getopts::Options;

use neat::genes;
use neat::exp::tictactoe;
use neat::pop;
use neat::exp::Experiment;

fn play<S: tictactoe::game::Strategy>(network_strategy: &mut tictactoe::exp::NetworkStrategy,
                                      network_player: tictactoe::game::Player,
                                      strategy: &mut S) -> Option<tictactoe::game::Player> {
    match network_player {
        tictactoe::game::Player::X =>
            tictactoe::game::play(network_strategy, strategy, true),
        tictactoe::game::Player::O =>
            tictactoe::game::play(strategy, network_strategy, true)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let network_player = (match args[1].trim() {
        "x" => Some(tictactoe::game::Player::X),
        "o" => Some(tictactoe::game::Player::O),
        _ => None
    }).unwrap();
    
    let network_path = Path::new(&args[2]);
    let genome = genes::Genome::load(network_path);
    let mut organism = pop::Organism::new(&genome);
    let mut network_strategy = tictactoe::exp::NetworkStrategy { network: &mut organism.network };
    
    let n = args[4].parse::<usize>().unwrap();
    let mut wins = 0;
    let mut draws = 0;

    for _ in 0..n {
        let winner = match args[3].trim() {
            "best" => play(&mut network_strategy,
                           network_player,
                           &mut tictactoe::strats::BestStrategy { forkable: false }),
            "forkable" => play(&mut network_strategy,
                               network_player,
                               &mut tictactoe::strats::BestStrategy { forkable: true }),
            "center" => play(&mut network_strategy,
                             network_player,
                             &mut tictactoe::strats::CenterStrategy),
            "random" => play(&mut network_strategy,
                             network_player,
                             &mut tictactoe::strats::RandomStrategy),
            "input" => play(&mut network_strategy,
                            network_player,
                            &mut tictactoe::strats::InputStrategy),
            _ => {
                assert!(false);
                None
            }
        };

        match winner {
            Some(player) =>
                if player == network_player {
                    wins += 1;
                },
            None =>
                draws += 1
        };
    }

    println!("Wins: {}, draws: {}", wins, draws);
}

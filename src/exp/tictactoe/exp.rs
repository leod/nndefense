use std::collections::HashMap;

use genes;
use nn;
use exp;
use pop;

use exp::tictactoe::game::*;
use exp::tictactoe::strats::*;

struct HallOfFame {
    fitness_weight: f64,
    generations_per_champion: usize,

    champions: Vec<pop::Organism>,
    last_update: usize,
}

pub struct TicTacToeExperiment {
    hall_of_fame: HallOfFame
}

pub struct NetworkStrategy<'a> {
    pub network: &'a mut nn::Network
}

impl<'a> Strategy for NetworkStrategy<'a> {
    fn get_move(&mut self, me: Player, state: &GameState) -> (usize, usize) {
        let mut input = Vec::new();
        let mut i = 0;

        for x in 0..3 {
            for y in 0..3 {
                let value = match state.field[x][y] {
                    Some(x) =>
                        if x == me {
                            1.0
                        } else {
                            -1.0
                        },
                    None => 0.0
                };

                input.push((i, value));
                i += 1;
            }
        }

        self.network.flush();
        self.network.set_input(&input);

        for _ in 1..10 {
            self.network.activate();
        }

        // Find legal output with highest activation
        let output = self.network.get_output();  
        let mut highest_move = None;
        let mut highest_activation = 0.0;

        i = 0;
        for x in 0..3 {
            for y in 0..3 {
                if state.field[x][y].is_some() {
                    continue;
                }

                if highest_move.is_none() || output[i].1 > highest_activation {
                    highest_move = Some((x,y));
                    highest_activation = output[i].1;
                }

                i += 1;
            }
        }
        
        highest_move.unwrap()
    }
}

pub fn score_network<Other: Strategy>(network: &mut nn::Network, other: &mut Other, runs: usize) -> f64 {
    let mut total_score = 0.0; 
    let mut strategy = NetworkStrategy { network: network };

    // Alternate who starts first
    let mut player = Player::X;

    for _ in 0..2*runs {
        let outcome = match player {
            Player::X => play(&mut strategy, other, false),
            Player::O => play(other, &mut strategy, false)
        };

        let score = match outcome {
            Some(winner) =>
                if winner == player {
                    5.0
                } else {
                    0.0
                },
            None => 2.0
        };

        total_score += score;
            
        player = match player {
            Player::X => Player::O,
            Player::O => Player::X
        };
    }

    total_score
}

pub fn score_network_vs_network(network1: &mut nn::Network, network2: &mut nn::Network) -> f64 {
    let mut strategy2 = NetworkStrategy { network: network2 };

    score_network(network1, &mut strategy2, 1)
}

impl HallOfFame {
    pub fn update(&mut self, population: &pop::Population) {
        if self.generations_per_champion <= population.generation - self.last_update {
            println!("Add champion, fitness: {}", population.best_organism().unwrap().fitness);

            self.champions.push(population.best_organism().unwrap().clone()); 

            self.last_update = population.generation;
        }
    }
}

impl TicTacToeExperiment {
    pub fn new() -> TicTacToeExperiment {
        TicTacToeExperiment {
            hall_of_fame: HallOfFame {
                fitness_weight: 0.1,
                generations_per_champion: 25,
                champions: Vec::new(),
                last_update: 0
            }
        }
    }
}

impl exp::Experiment for TicTacToeExperiment {
    fn initial_genome(&self) -> genes::Genome {
        genes::Genome::initial_genome(9, 9, 9, true)
    }

    fn node_names(&self) -> HashMap<genes::NodeId, String> {
        let mut map = HashMap::new();
        let mut i = 0;

        for x in 0..3 {
            for y in 0..3 {
                map.insert(i, format!("{}{}", x, y));
                i += 1;
            }
        }

        map.insert(i, "B".to_string());
        i += 1;

        for x in 0..3 {
            for y in 0..3 {
                map.insert(i, format!("{}{}", x, y));
                i += 1;
            }
        }

        map
    }

    fn evaluate(&self, network: &mut nn::Network, organisms: &[pop::Organism]) -> f64 {
        /*let vs_fixed = score_network(network, &mut BestStrategy { forkable: false }, 100) + 
                       score_network(network, &mut BestStrategy { forkable: true }, 100) + 
                       score_network(network, &mut RandomStrategy, 100) +
                       score_network(network, &mut CenterStrategy, 100) +
                       score_network(network, &mut BadStrategy, 100);*/

        let vs_fixed = 0.0;

        // Play against all the other organisms
        let mut vs_pop = 0.0;

        for organism in organisms {
            // TODO: To avoid having to clone here, maybe separate a network's definition from its
            // activation state
            let mut network2 = organism.network.clone();

            vs_pop += score_network_vs_network(network, &mut network2);
        }

        // Play against hall of fame
        let mut vs_hof = 0.0;

        for organism in self.hall_of_fame.champions.iter() {
            let mut network2 = organism.network.clone();

            vs_hof += score_network_vs_network(network, &mut network2);
        }

        /*println!("pop: {}, hof: {}", vs_pop, vs_hof);
        println!("{}", vs_fixed + vs_pop / organisms.len() as f64 + vs_hof / self.hall_of_fame.champions.len() as f64);*/
        vs_fixed +
        vs_pop / organisms.len() as f64 +
        if !self.hall_of_fame.champions.is_empty() { vs_hof / self.hall_of_fame.champions.len() as f64 } else { 0.0 }
    }

    fn evaluate_to_string(&self, network: &mut nn::Network) -> String {
        "".to_string()
    }

    fn post_evaluation(&mut self, population: &pop::Population) {
        self.hall_of_fame.update(population);
    }
}

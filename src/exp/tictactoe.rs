extern crate rand;

use std::collections::HashMap;
use rand::Rng;
use rand::StdRng;
use rand::SeedableRng;
use genes;
use nn;
use exp;

pub struct TicTacToeExperiment;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Player {
    X,
    O
}

pub struct GameState {
    field: [[Option<Player>; 3]; 3]
}

pub type Strategy = FnMut(&GameState) -> (usize, usize);

pub fn random_strategy(state: &GameState) -> (usize, usize) {
    let mut legal_moves = vec![];

    for x in 0..3 {
        for y in 0..3 {
            if state.field[x][y].is_none() {
                legal_moves.push((x,y));
            }
        }
    }

    let mut rng = rand::thread_rng();
    *rng.choose(&legal_moves).unwrap()
}

pub fn network_strategy<'a>(me: Player, network: &'a mut nn::Network) -> &'a Strategy {
    &|&state| {
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

        network.set_input(&input);

        for _ in 1..10 {
            network.activate();
        }

        // Find legal output with highest activation
        let output = network.get_output();  
        let highest_move = None;
        let highest_activation = 0.0;

        i = 0;
        for x in 0..3 {
            for y in 0..3 {
                if highest_move.is_none() || output[i].1 > highest_activation {
                    highest_move = Some((x,y));
                    highest_activation = output[i].1;
                }
            }
        }
        
        highest_move.unwrap()
    }
}

fn initial_state() -> GameState {
    GameState {
        field: [
            [None, None, None],
            [None, None, None],
            [None, None, None]
        ]
    }
}

fn print_state(state: &GameState) {
    //println!("---");

    for y in 0..3 {
        for x in 0..3 {
            print!("{}", match state.field[x][y] {
                Some(Player::X) => "X",
                Some(Player::O) => "O",
                None => " ",
            });
        }

        println!("");
    }

    println!("---");
}

pub fn play(strat_x: &mut Strategy, strat_o: &mut Strategy, print: bool) -> Option<Player> {
    const HOW_TO_WIN: &'static [[(usize, usize); 3]] = &[
        [(0,0),(0,1),(0,2)],
        [(1,0),(1,1),(1,2)],
        [(2,0),(2,1),(2,2)],

        [(0,0),(1,0),(2,0)],
        [(0,1),(1,1),(2,1)],
        [(0,2),(1,2),(2,2)],

        [(0,0),(1,1),(2,2)],
        [(2,0),(1,1),(0,2)]
    ];

    let mut state = initial_state();
    let mut turn = Player::X;

    for _ in 0..9 {
        let (move_x, move_y) = match turn {
            Player::X => strat_x(&state),
            Player::O => strat_o(&state)
        };
        assert!(move_x < 3 && move_y < 3);
        assert!(state.field[move_x][move_y].is_none());

        state.field[move_x][move_y] = Some(turn);

        if print {
            print_state(&state);
        }

        if HOW_TO_WIN.iter().any(|positions| positions.iter().all(|&(x,y)| state.field[x][y] == Some(turn))) {
            if print {
                println!("Player {} wins", match turn {
                    Player::X => "X",
                    Player::O => "O",
                });
            }

            return Some(turn);
        }

        turn = match turn {
            Player::X => Player::O,
            Player::O => Player::X
        };
    }

    if print {
        println!("Draw");
    }

    None
}

impl exp::Experiment for TicTacToeExperiment {
    fn initial_genome(&self) -> genes::Genome {
        genes::Genome::initial_genome(9, 9, 0, true)
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

    fn evaluate(&self, network: &mut nn::Network) -> f64 {
        0.0
    }
}

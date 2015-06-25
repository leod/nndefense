extern crate rand;

use std::collections::HashMap;
use rand::Rng;
use rand::StdRng;
use rand::SeedableRng;
use genes;
use nn;
use exp;

const ROAD_WIDTH: usize = 3; 
const ROAD_HEIGHT: usize = 4;

pub struct RoadGameExperiment;

struct GameState {
    road: [[bool; ROAD_HEIGHT]; ROAD_WIDTH],
    player_x: usize,
    hits: usize,
    hit_now: bool,
    rng: rand::StdRng,
}

enum MoveInput {
    Left,
    Right,
}

fn road_game_step(state: &mut GameState, input: Option<MoveInput>) {
    match input {
        Some(MoveInput::Left) => if state.player_x > 0 { state.player_x -= 1; },
        Some(MoveInput::Right) => if state.player_x < ROAD_WIDTH-1 { state.player_x += 1; },
        None => ()
    }

    for y in 0..ROAD_HEIGHT-1 {
        for x in 0..ROAD_WIDTH {
            state.road[x][y] = state.road[x][y+1];
        }
    }

    if state.road[state.player_x][0] {
        state.hits += 1;
        state.hit_now = true;
    } else {
        state.hit_now = false;
    }
    
    // Spawn new objects at the top
    let mut num_new = 0;
    for x in 0..ROAD_WIDTH {
        if state.rng.next_f64() < 0.3 && num_new < 2 {
            state.road[x][ROAD_HEIGHT-1] = true;
            num_new += 1;

            if x == 1 && state.road[1][1] && state.road[1][2] {
                // No wall in the middle
                state.road[x][ROAD_HEIGHT-1] = false;
                num_new -= 1;
            } else if num_new == 2 {
                // Try to prevent some unwinnable situations
                let mut num_next_line = 0;
                let mut one_different = false;
                for x2 in 0..ROAD_WIDTH {
                    if state.road[x2][ROAD_HEIGHT-2] {
                        num_next_line += 1;
                    }
                    if state.road[x2][ROAD_HEIGHT-2] != state.road[x2][ROAD_HEIGHT-1] {
                        one_different = true;
                    }
                }

                if num_next_line > 1 && one_different {
                    state.road[x][ROAD_HEIGHT-1] = false;
                    num_new -= 1;
                }
            }

        } else {
            state.road[x][ROAD_HEIGHT-1] = false;
        }
    }
}

fn network_input(state: &GameState, network: &mut nn::Network) -> Option<MoveInput> {
    let mut input = Vec::new();
    let mut i = 0;

    for x in 0..ROAD_WIDTH {
        let value = if x == state.player_x {
            1.0
        } else {
            -1.0
        };

        input.push((i, value));
        i += 1;
    }

    for y in 1..ROAD_HEIGHT {
        for x in 0..ROAD_WIDTH {
            let value = if state.road[x][y] {
                1.0 
            } else { 
                -1.0
            };

            input.push((i, value));
            i += 1;
        }
    }

    //let x_value = state.player_x as f64 - 1.0;
    //let x_value = if state.player_x == 0 { -1.0 } else { 1.0 };
    //let x_value = state.player_x as f64 / (ROAD_WIDTH-1) as f64 * 2.0 - 1.0;
    //input.push((i, x_value));

    network.set_input(&input);

    for _ in 1..10 {
        network.activate();
    }

    let out_value = network.get_output()[0].1;

    if out_value > 0.5 {
        Some(MoveInput::Right)
    } else if out_value < -0.5 {
        Some(MoveInput::Left)
    } else {
        None
    }
}

fn state_to_string(state: &GameState) -> String {
    let mut str = String::new();

    for y in (0..ROAD_HEIGHT).rev() {
        for x in 0..ROAD_WIDTH {
            let c = if y == 0 && x == state.player_x { 
                if state.hit_now { 'H' } else { 'X' }
            } else if state.road[x][y] {
                'o'
            } else {
                ' '
            };

            str.push(c);
        }
        str.push('\n');
    }

    return str;
}

fn initial_state(seed: usize) -> GameState {
    let empty_road = [false,false,false,false];
    GameState {
        road: [empty_road, empty_road, empty_road],
        player_x: 1,
        hits: 0,
        hit_now: false,
        rng: rand::StdRng::from_seed(&[seed])
    }
}

impl exp::Experiment for RoadGameExperiment {
    fn initial_genome(&self) -> genes::Genome {
        genes::Genome::initial_genome(ROAD_WIDTH * (ROAD_HEIGHT+0), 1, 0, true)
    }

    fn node_names(&self) -> HashMap<genes::NodeId, String> {
        /*[(0, "P0".to_string()), (1, "P1".to_string()), (2, "P2".to_string()),
         (3, "00".to_string()), (4, "01".to_string()), (5, "02".to_string()),
         (6, "10".to_string()), (7, "11".to_string()), (8, "12".to_string()),
         (9, "20".to_string()), (10, "21".to_string()), (11, "22".to_string()),
         (12, "B".to_string()), (13, "O".to_string())].into_iter().collect()*/

        let mut map = HashMap::new();

        map.insert(0, "P0".to_string());
        map.insert(1, "P1".to_string());
        map.insert(2, "P2".to_string());
        map.insert(3, "01".to_string());
        map.insert(4, "11".to_string());
        map.insert(5, "21".to_string());
        map.insert(6, "02".to_string());
        map.insert(7, "12".to_string());
        map.insert(8, "22".to_string());
        map.insert(9, "03".to_string());
        map.insert(10, "13".to_string());
        map.insert(11, "23".to_string());
        map.insert(12, "B".to_string());
        map.insert(13, "O".to_string());

        map
    }

    fn evaluate(&self, network: &mut nn::Network) -> f64 {
        let max_steps = 10000;
        let num_runs = 500;
        let mut num_steps = 0;

        let seed: &[_] = &[1337];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);

        for _ in 0..num_runs {
            let mut state = initial_state(rng.gen::<usize>());
            network.flush();

            for _ in 0..max_steps {
                let input = network_input(&state, network);
                road_game_step(&mut state, input);
                num_steps += 1;
                if state.hit_now {
                    break;
                }
            }
        }

        (num_steps as f64 / num_runs as f64).powf(2.0)
    }
}

pub fn evaluate_to_string(network: &mut nn::Network) -> String {
    let max_steps = 10000;
    let num_runs = 500;
    let mut num_steps = 0;

    let seed: &[_] = &[1337];
    let mut rng: StdRng = rand::SeedableRng::from_seed(seed);

    let mut str = String::new();

    network.flush();

    for _ in 0..num_runs {
        let mut state = initial_state(rng.gen::<usize>());
        network.flush();

        for _ in 0..max_steps {
            str.push_str(&state_to_string(&state));
            str.push_str(&"---\n");

            let input = network_input(&state, network);
            road_game_step(&mut state, input);
            num_steps += 1;
            if state.hit_now {
                break;
            }
        }

        str.push_str(&state_to_string(&state));

        str.push_str(&"=================\n");
    }

    format!("Steps per run: {}\n", num_steps as f64 / num_runs as f64) + &str
}

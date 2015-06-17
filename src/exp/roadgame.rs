extern crate rand;

use rand::Rng;
use genes;
use pop;
use nn;

const road_width: usize = 3; 
const road_height: usize = 4;

struct GameState {
    road: [[bool; road_height]; road_width],
    player_x: usize,
    hits: usize,
    hit_now: bool,
}

enum MoveInput {
    Left,
    Right,
}

fn road_game_step(state: &mut GameState, input: Option<MoveInput>) {
    match input {
        Some(MoveInput::Left) => if state.player_x > 0 { state.player_x -= 1; },
        Some(MoveInput::Right) => if state.player_x < road_width-1 { state.player_x += 1; },
        None => ()
    }

    for y in 0..road_height-1 {
        for x in 0..road_width {
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
    let mut rng = rand::thread_rng();
    let mut num_new = 0;
    for x in 0..road_width {
        if rng.next_f64() < 0.3 && num_new < 2 {
            state.road[x][road_height-1] = true;

            // Try to prevent some unwinnable situations
            if (num_new == 1) {
                let mut num_next_line = 0;
                let mut one_different = false;
                for x2 in 0..road_width {
                    if state.road[x2][road_height-2] {
                        num_next_line += 1;
                    }
                    if state.road[x2][road_height-2] != state.road[x2][road_height-1] {
                        one_different = true;
                    }
                }

                if (num_next_line > 1 && one_different) {
                    state.road[x][road_height-1] = false;
                    num_new -= 1;
                }
            }

            num_new += 1;
        } else {
            state.road[x][road_height-1] = false;
        }
    }
}

fn network_input(state: &GameState, network: &mut nn::Network) -> Option<MoveInput> {
    let mut input = Vec::new();
    let mut i = 0;

    for y in 0..road_height {
        for x in 0..road_width {
            let value = if state.road[x][y] {
                1.0 
            } else { 
                if y == 0 && x == state.player_x {
                    0.0
                } else {
                    -1.0
                }
            };

            input.push((i, value));

            i += 1;
        }
    }

    //let x_value = state.player_x as f64 - 1.0;
    //let x_value = if state.player_x == 0 { -1.0 } else { 1.0 };
    //let x_value = state.player_x as f64 / (road_width-1) as f64 * 2.0 - 1.0;
    //input.push((i, x_value));

    network.set_input(&input);

    for i in 1..10 {
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

    for y in (0..road_height).rev() {
        for x in 0..road_width {
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

pub fn initial_genome() -> genes::Genome {
    genes::Genome::initial_genome(road_width * road_height, 1, 0, true)
}

fn initial_state() -> GameState {
    let empty_road = [false,false,false,false];
    GameState {
        road: [empty_road, empty_road, empty_road],
        player_x: 1,
        hits: 0,
        hit_now: false,
    }
}

pub fn evaluate(network: &mut nn::Network) -> f64 {
    let max_steps = 2000;
    let mut num_steps = 0;
    let mut state = initial_state();
    network.flush();

    while num_steps < max_steps {
        let input = network_input(&state, network);
        road_game_step(&mut state, input);
        num_steps += 1;
        //println!("{}", state_to_string(&state));
        //println!("----------------");
    }

    //organism.fitness = ((max_steps - state.hits) as f64 / max_steps as f64).sqrt();
    ((max_steps - state.hits) as f64).powf(2.0)
}

pub fn evaluate_to_death(network: &mut nn::Network) -> f64 {
    let max_steps = 100;
    let num_runs = 500;
    let mut num_steps = 0;

    for _ in 0..num_runs {
        let mut state = initial_state();
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

    //organism.fitness = ((max_steps - state.hits) as f64 / max_steps as f64).sqrt();
    (num_steps as f64 / num_runs as f64).powf(2.0)
}

pub fn evaluate_to_string(network: &mut nn::Network) -> String {
    let max_steps = 2000;
    let mut num_steps = 0;
    let empty_road = [false,false,false];
    let mut state = initial_state();
    let mut str = String::new();

    network.flush();

    while num_steps < max_steps {
        str.push_str(&state_to_string(&state));
        str.push_str(&"---\n");

        let input = network_input(&state, network);
        road_game_step(&mut state, input);
        num_steps += 1;
    }

    format!("Hits: {}\n", state.hits) + &str
}

pub fn evaluate_to_death_to_string(network: &mut nn::Network) -> String {
    let max_steps = 100;
    let num_runs = 500;
    let mut num_steps = 0;
    let empty_road = [false,false,false];
    let mut state = initial_state();
    let mut str = String::new();

    network.flush();

    while num_steps < max_steps {
        let input = network_input(&state, network);
        road_game_step(&mut state, input);
        num_steps += 1;
    }

    for _ in 0..num_runs {
        let mut state = initial_state();
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

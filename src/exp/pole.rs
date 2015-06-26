extern crate rand;

use std::collections::HashMap;
use rand::Rng;
use rand::StdRng;
use rand::SeedableRng;
use genes;
use nn;
use exp;
use pop;

pub struct PoleExperiment;

struct PoleState {
    x: f64,
    x_dot: f64,
    theta: f64,
    theta_dot: f64
}

enum Input {
    Left,
    Right
}

// As in the pole simulator written by Richard Sutton and Charles Anderson
fn pole_step(state: &mut PoleState, input: Input) {
    const GRAVITY: f64 = 9.8;
    const MASS_CART: f64 = 1.0;
    const MASS_POLE: f64 = 0.1;
    const MASS_TOTAL: f64 = MASS_CART + MASS_POLE;
    const HALF_LENGTH: f64 = 0.5;
    const POLE_MASS_LENGTH: f64 = MASS_POLE * HALF_LENGTH;
    const FORCE_MAG: f64 = 10.0;
    const TAU: f64 = 0.02; // time between steps
    const FOUR_THIRDS: f64 = 1.3333333333333;

    let force = match input { Input::Left => -FORCE_MAG, Input::Right => FORCE_MAG };
    let cos_theta = state.theta.cos();
    let sin_theta = state.theta.sin();

    let temp = (force + POLE_MASS_LENGTH * state.theta_dot.powi(2) * sin_theta) / MASS_TOTAL;
    let theta_acc = (GRAVITY * sin_theta - cos_theta * temp) /
                    (HALF_LENGTH * (FOUR_THIRDS - MASS_POLE * cos_theta.powi(2) / MASS_TOTAL));
    let x_acc = temp - POLE_MASS_LENGTH * theta_acc * cos_theta / MASS_TOTAL;

    // Euler interpolation
    state.x += TAU * state.x_dot;
    state.x_dot += TAU * x_acc;
    state.theta += TAU * state.theta_dot;
    state.theta_dot += TAU * theta_acc;
}

fn initial_state() -> PoleState {
    let mut rng = rand::thread_rng();

    PoleState {
        x: rng.gen_range(-2.4, 2.4),
        x_dot: rng.gen_range(-1.0, 1.0),
        theta: rng.gen_range(-0.2, 0.2),
        theta_dot: rng.gen_range(-1.5, 1.5),
    }
}

fn network_input(state: &PoleState, network: &mut nn::Network) {
    const TWELVE_DEGREES: f64 = 0.2094384;

    let input = vec![(0, state.x / 2.4),
                     (1, state.x_dot),
                     (2, state.theta),
                     (3, state.theta_dot)];
    network.set_input(&input);

    for _ in 1..10 {
        network.activate();
    }
}

impl exp::Experiment for PoleExperiment {
    fn initial_genome(&self) -> genes::Genome {
        genes::Genome::initial_genome(4, 2, 0, true)
    }

    fn node_names(&self) -> HashMap<genes::NodeId, String> {
        let mut map = HashMap::new();

        map.insert(0, "X".to_string());
        map.insert(1, "X_DOT".to_string());
        map.insert(2, "THETA".to_string());
        map.insert(3, "THETA_DOT".to_string());

        map
    }

    fn evaluate(&self, network: &mut nn::Network, organisms: &[pop::Organism]) -> f64 {
        const MAX_STEPS: usize = 100;
        const THRESH: usize = 100;

        let state = initial_state();

        for steps in 0..MAX_STEPS {
            
        }

        0.0
    }

    fn evaluate_to_string(&self, network: &mut nn::Network) -> String {
        "".to_string()
    }

    fn post_evaluation(&mut self, population: &pop::Population) {

    }
}

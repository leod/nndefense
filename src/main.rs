extern crate rand;
extern crate rustc_serialize;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::thread;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

mod genes; 
mod mutation;
mod nn;
mod pop;
mod mating;
mod exp;

use exp::Experiment;

/*fn to_f(x: bool) -> f64 {
    if x { 1.0 } else { -1.0 }
}

fn evaluate(organism: &mut pop::Organism, print: bool) {
    let fitness = {
        let mut error = |x: bool, y: bool| -> f64 {
            organism.network.flush();
            organism.network.set_input(&vec![(0, to_f(x)), (1, to_f(y))]);
            for _ in 1..10 { organism.network.activate(); }

            let output = organism.network.get_output()[0].1;
            let expected_output = to_f(x != y);

            if (print) {
                println!("{},{} -> {} vs {}", x, y, output, expected_output);
            }

            (output - expected_output).abs() / 2.0
        };

        let sum_error = error(false, false) +
                        error(false, true) +
                        error(true, false) +
                        error(true, true);
        (4.0 - sum_error).powf(2.0)
    };

    organism.fitness = fitness;
} */

fn evaluate(experiment: &exp::Experiment, population: &mut pop::Population) {
    evaluate_single_threaded(experiment, population);
    //evaluate_multi_threaded(experiment, population);
}

fn evaluate_single_threaded(experiment: &exp::Experiment, population: &mut pop::Population) {
    for species in population.species.iter_mut() {
        for organism in species.organisms.iter_mut() {
            let fitness = experiment.evaluate(&mut organism.network);
            organism.fitness = fitness;
        }
    }
}

fn evaluate_multi_threaded(experiment: &exp::Experiment, population: &mut pop::Population) {
    let num_threads = 4;
    let num_population = population.num_organisms();

    let num_tasks_per_thread = num_population / num_threads;

    let mut organisms = vec![];

    for (species_index, species) in population.species.iter().enumerate() {
        for (organism_index, organism) in species.organisms.iter().enumerate() {
            organisms.push((species_index, organism_index, organism.clone()));
        }
    }

    let (results_send, results_recv): (Sender<(usize, usize, f64)>, Receiver<(usize, usize, f64)>) = channel();
    let shared_organisms = Arc::new(organisms); 
    let shared_experiment = Arc::new(experiment);
    let mut threads = vec![];

    for k in 0..num_threads {
        let thread_organisms = shared_organisms.clone();
        let thread_results = results_send.clone();
        let thread_experiment = shared_experiment.clone();

        threads.push(thread::spawn(move || {
            let local_organisms = &thread_organisms[num_tasks_per_thread*k..num_tasks_per_thread*(k+1)];

            for &(species_index, organism_index, ref organism) in local_organisms {
                let mut network = organism.network.clone();
                //let fitness = thread_experiment.evaluate(&mut network);
                let fitness = -1.0; // TODO: send experiment

                thread_results.send((species_index, organism_index, fitness)).unwrap();
            }
        }));
    }

    for _ in 0..num_population {
        let (species_index, organism_index, fitness) = results_recv.recv().unwrap();
        population.species[species_index].organisms[organism_index].fitness = fitness;
    }

    for thread in threads.into_iter() {
        thread.join().unwrap();
    }
}

fn main() {
    /*exp::tictactoe::play(&mut exp::tictactoe::RandomStrategy,
                         &mut exp::tictactoe::BestStrategy { forkable: false },
                         true);*/

    /*exp::tictactoe::play(&mut exp::tictactoe::BestStrategy { forkable: false },
                         &mut exp::tictactoe::BestStrategy { forkable: false },
                         true);*/

    let mut i = 0;
    let mut rng = rand::thread_rng();

    let num_population = 200;
    //let mut experiment = exp::roadgame::RoadGameExperiment;
    let mut experiment = exp::tictactoe::TicTacToeExperiment;
    let initial_genome = experiment.initial_genome();

    let mut population = pop::Population::from_initial_genome(&mut rng,
                                                              &pop::STANDARD_SETTINGS,
                                                              &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS},
                                                              //&mutation::STANDARD_SETTINGS,
                                                              &genes::STANDARD_COMPAT_COEFFICIENTS,
                                                              &initial_genome,
                                                              num_population);
    evaluate(&experiment as &exp::Experiment, &mut population);

    loop {
        i += 1;

        if i > 5000 { 
            break;
        }


        println!("Generation {}", i);
        population.epoch(&mut rng);
        println!("");

        {
            evaluate(&experiment as &exp::Experiment, &mut population);
            /*for species in population.species.iter_mut() {
                for organism in species.organisms.iter_mut() {
                    //evaluate(organism, false);
                    exp::roadgame::evaluate_to_death(organism);
                }
            }*/
        }

        {
            let mut best = population.best_organism().unwrap().clone();

            println!("best: {}, best forkable: {}, random: {}, center: {}, bad: {}",
                     exp::tictactoe::score_network(&mut best.network, &mut exp::tictactoe::BestStrategy { forkable: false }, 100),
                     exp::tictactoe::score_network(&mut best.network, &mut exp::tictactoe::BestStrategy { forkable: true }, 100),
                     exp::tictactoe::score_network(&mut best.network, &mut exp::tictactoe::RandomStrategy, 100),
                     exp::tictactoe::score_network(&mut best.network, &mut exp::tictactoe::CenterStrategy, 100),
                     exp::tictactoe::score_network(&mut best.network, &mut exp::tictactoe::BadStrategy, 100));

            /*let mut f = File::create(&Path::new(&format!("networks/runs/{}.txt", i))).unwrap();
            f.write_all(exp::roadgame::evaluate_to_death_to_string(&mut best.network).as_bytes()).unwrap();*/

            //evaluate(&mut best, true);
            //println!("genome: {:?}", &best.genome);
            //println!("network: {:?}", &pop::Organism::new(&best.genome).network);
            best.genome.compile_to_png(experiment.node_names(),
                                       Path::new(&format!("networks/dot/{}.dot", i)),
                                       Path::new(&format!("networks/{}-{}.png", i, best.fitness))).unwrap();

            best.genome.save(Path::new(&format!("networks/best/{}.json", i)));

            /*for (j, species) in population.species.iter().enumerate() {
                species.best_genome.compile_to_png(Path::new(&format!("networks/dot/{}_{}.dot", i, j)),
                                                   Path::new(&format!("networks/{}_{}.png", i, j)));
            }*/
        }
    }

    /*let mut best = population.best_organism().unwrap().clone();
    //evaluate(&mut best, true);
    exp::roadgame::evaluate_to_death(&mut best);
    println!("best fitness: {}", best.fitness);

    best.genome.compile_to_png(Path::new("best.dot"), Path::new("best.png"));*/
}

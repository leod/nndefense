extern crate neat;
extern crate rand;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::thread;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use neat::genes;
use neat::exp;
use neat::pop;
use neat::mutation;
use neat::exp::Experiment;

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

fn evaluate<E: exp::Experiment + Send + Sync + Clone + 'static>(experiment: &mut E, population: &mut pop::Population) {
    //evaluate_single_threaded(experiment, population);
    let shared_experiment = Arc::new(experiment.clone());
    
    evaluate_multi_threaded(shared_experiment, population);

    experiment.post_evaluation(&population);
}

fn evaluate_single_threaded<E: exp::Experiment + Send + Sync + 'static>(experiment: Arc<E>, population: &mut pop::Population) {
    for species in population.species.iter_mut() {
        for organism in species.organisms.iter_mut() {
            let fitness = 0.0; //experiment.evaluate(&mut organism.network);
            organism.fitness = fitness;
        }
    }
}

fn evaluate_multi_threaded<E: exp::Experiment + Send + Sync + 'static>(experiment: Arc<E>, population: &mut pop::Population) {
    let num_threads = 8;
    let num_population = population.num_organisms();

    let num_tasks_per_thread = num_population / num_threads;

    let mut organism_indices = vec![];
    let mut organisms = vec![];

    for (species_index, species) in population.species.iter().enumerate() {
        for (organism_index, organism) in species.organisms.iter().enumerate() {
            organism_indices.push((species_index, organism_index));
            organisms.push(organism.clone());
        }
    }

    let (results_send, results_recv): (Sender<(usize, usize, f64)>, Receiver<(usize, usize, f64)>) = channel();
    let shared_organism_indices = Arc::new(organism_indices);
    let shared_organisms = Arc::new(organisms); 
    let mut threads = vec![];

    for k in 0..num_threads {
        let thread_organism_indices = shared_organism_indices.clone();
        let thread_organisms = shared_organisms.clone();
        let thread_results = results_send.clone();
        let thread_experiment = experiment.clone();

        threads.push(thread::spawn(move || {
            let a = num_tasks_per_thread*k;
            let b = num_tasks_per_thread*(k+1);
            let local_organism_indices = &thread_organism_indices[a..b];
            let local_organisms = &thread_organisms[a..b];

            for (&(species_index, organism_index), ref organism) in local_organism_indices.iter().zip(local_organisms) {
                let mut network = organism.network.clone();
                let fitness = thread_experiment.evaluate(&mut network, &thread_organisms[..]);

                thread_results.send((species_index, organism_index, fitness)).unwrap();
            }
        }));
    }

    // Receive changes and make the actual mutations in the population.
    // Blocks until all organisms have been evaluated by the threads.
    for _ in 0..num_population {
        let (species_index, organism_index, fitness) = results_recv.recv().unwrap();
        population.species[species_index].organisms[organism_index].fitness = fitness;
    }

    for thread in threads.into_iter() {
        thread.join().unwrap();
    }
}

fn main() {
    let mut i = 0;
    let mut rng = rand::thread_rng();

    let num_population = 512;
    //let mut experiment = exp::roadgame::RoadGameExperiment;
    let mut experiment = exp::tictactoe::exp::TicTacToeExperiment::new();

    let mut population = pop::Population::from_initial_genome(&mut rng,
                                                              &experiment.population_settings(),
                                                              &experiment.mutation_settings(),
                                                              &experiment.compat_coefficients(),
                                                              &experiment.initial_genome(),
                                                              num_population);
    evaluate(&mut experiment, &mut population);

    loop {
        i += 1;

        if i > 5000 { 
            break;
        }

        println!("Generation {}", i);
        population.epoch(&mut rng);
        println!("");

        evaluate(&mut experiment, &mut population);

        {
            let mut best = population.best_organism().unwrap().clone();

            println!("best: {}, best forkable: {}, random: {}, center: {}, bad: {}",
                     exp::tictactoe::exp::score_network(&mut best.network, &mut exp::tictactoe::strats::BestStrategy { forkable: false }, 100),
                     exp::tictactoe::exp::score_network(&mut best.network, &mut exp::tictactoe::strats::BestStrategy { forkable: true }, 100),
                     exp::tictactoe::exp::score_network(&mut best.network, &mut exp::tictactoe::strats::RandomStrategy, 100),
                     exp::tictactoe::exp::score_network(&mut best.network, &mut exp::tictactoe::strats::CenterStrategy, 100),
                     exp::tictactoe::exp::score_network(&mut best.network, &mut exp::tictactoe::strats::BadStrategy, 100));

            best.genome.compile_to_png(experiment.node_names(),
                                       Path::new(&format!("networks/dot/{}.dot", i)),
                                       Path::new(&format!("networks/{}-{}.png", i, best.fitness))).unwrap();
            best.genome.save(Path::new(&format!("networks/best/{}.json", i)));
        }
    }
}

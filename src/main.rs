#![allow(unstable)]

extern crate rand;

use std::collections::HashMap;
use std::io::prelude::*;
use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::thread;

mod genes; 
mod mutation;
mod nn;
mod pop;
mod mating;
mod exp;

fn to_f(x: bool) -> f64 {
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
}

fn main() {
    let mut genome: genes::Genome = genes::Genome {
        /*nodes: vec![genes::Node { id: 0, node_type: genes::NodeType::Input },
                    genes::Node { id: 1, node_type: genes::NodeType::Input },
                    genes::Node { id: 5, node_type: genes::NodeType::Hidden },
                    genes::Node { id: 2, node_type: genes::NodeType::Bias },
                    genes::Node { id: 3, node_type: genes::NodeType::Output }],
        links: vec![genes::Link { from_id: 0, to_id: 3, enabled: true, innovation: 0, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 1, to_id: 3, enabled: true, innovation: 1, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 2, to_id: 3, enabled: true, innovation: 2, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 0, to_id: 5, enabled: true, innovation: 3, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 1, to_id: 5, enabled: true, innovation: 4, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 5, to_id: 3, enabled: true, innovation: 5, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 2, to_id: 5, enabled: true, innovation: 6, weight: 0.0, is_recurrent: false }]*/
        nodes: vec![genes::Node { id: 0, node_type: genes::NodeType::Input },
                    genes::Node { id: 1, node_type: genes::NodeType::Input },
                    genes::Node { id: 2, node_type: genes::NodeType::Bias },
                    genes::Node { id: 3, node_type: genes::NodeType::Output }],
        links: vec![genes::Link { from_id: 0, to_id: 3, enabled: true, innovation: 0, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 1, to_id: 3, enabled: true, innovation: 1, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 2, to_id: 3, enabled: true, innovation: 2, weight: 0.0, is_recurrent: false }]
    };

    /*let genome_xor: genes::Genome = genes::Genome {
        nodes: vec![genes::Node { id: 0, node_type: genes::NodeType::Input },
                    genes::Node { id: 1, node_type: genes::NodeType::Input },
                    genes::Node { id: 5, node_type: genes::NodeType::Hidden },
                    genes::Node { id: 2, node_type: genes::NodeType::Bias },
                    genes::Node { id: 3, node_type: genes::NodeType::Output }],
        links: vec![genes::Link { from_id: 0, to_id: 3, enabled: true, innovation: 0, weight: 1.0, is_recurrent: false },
                    genes::Link { from_id: 1, to_id: 3, enabled: true, innovation: 1, weight: 1.0, is_recurrent: false },
                    genes::Link { from_id: 2, to_id: 3, enabled: true, innovation: 2, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 0, to_id: 5, enabled: true, innovation: 3, weight: 0.5, is_recurrent: false },
                    genes::Link { from_id: 1, to_id: 5, enabled: true, innovation: 4, weight: 0.5, is_recurrent: false },
                    genes::Link { from_id: 5, to_id: 3, enabled: true, innovation: 5, weight: -2.0, is_recurrent: false },
                    genes::Link { from_id: 2, to_id: 5, enabled: true, innovation: 6, weight: 0.0, is_recurrent: false }]
    };*/

    /*let mut org = pop::Organism::new(&genome_xor);
    evaluate(&mut org, true);
    return;*/

    let mut i = 0;
    let mut rng = rand::thread_rng();
    let mut node_counter = 4;

    /*let mut genome2 = genome.clone();
    mutation::change_link_weights_reset_all(&mut genome2, &mut rng, 1.0);
    println!("{:?}", &genome2);

    println!("{}", genes::compatibility(&genes::STANDARD_COMPAT_COEFFICIENTS, &genome, &genome2));
    return;*/

    /*for i in 1..1000 {
        let mut g = genome.clone(); 
        let mut state = mutation::State {
            node_counter: 5,
            innovation_counter: 5,
            link_innovations: HashMap::new(),
            node_innovations: HashMap::new()
        };

        mutation::mutate(&mut g, &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS }, &mut rng, &mut state);
        mutation::mutate(&mut g, &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS }, &mut rng, &mut state);
        mutation::mutate(&mut g, &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS }, &mut rng, &mut state);
        mutation::mutate(&mut g, &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS }, &mut rng, &mut state);
        mutation::mutate(&mut g, &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS }, &mut rng, &mut state);
        mutation::mutate(&mut g, &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS }, &mut rng, &mut state);
        mutation::mutate(&mut g, &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS }, &mut rng, &mut state);
        //mutation::change_link_weights_standard(&mut g, &mut rng, 1.0, 1.0);
        g.compile_to_png(Path::new(&format!("networks/dot/{}.dot", i)), Path::new(&format!("networks/{}.png", i)));
    }
    return;*/

    let num_threads = 16;
    let num_population = 1024;

    let initial_genome = exp::roadgame::initial_genome();

    let mut population = pop::Population::from_initial_genome(&mut rng,
                                                              &pop::STANDARD_SETTINGS,
                                                              &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS},
                                                              //&mutation::STANDARD_SETTINGS,
                                                              &genes::STANDARD_COMPAT_COEFFICIENTS,
                                                              &initial_genome,
                                                              num_population);
    for species in population.species.iter_mut() {
        for organism in species.organisms.iter_mut() {
            //evaluate(organism, false);
            exp::roadgame::evaluate_to_death(organism);
        }
    }

    loop {
        i += 1;

        if i > 5000 { 
            break;
        }


        println!("Generation {}", i);
        population.epoch(&mut rng);
        println!("");

        {
            let num_tasks_per_thread = num_population / num_threads;

            // because fuck you
            /*let mut organisms = vec![];

            for species in population.species.iter() {
                for organism in species.organisms.iter() {
                    organisms.push(organism.clone());
                }
            }

            let threads: Vec<_> = organisms.chunks_mut(num_tasks_per_thread).map(|chunk| {
                thread::spawn(move || {
                    for organism in chunk.iter_mut() {
                        exp::roadgame::evaluate_to_death(organism); 
                    }
                })
            }).collect();

            for thread in threads.into_iter() {
                thread.join();
            }*/

            for species in population.species.iter_mut() {
                for organism in species.organisms.iter_mut() {
                    //evaluate(organism, false);
                    exp::roadgame::evaluate_to_death(organism);
                }
            }
        }

        {
            let mut best = population.best_organism().unwrap().clone();

            let mut f = File::create(&Path::new(&format!("networks/runs/{}.txt", i))).unwrap();
            f.write_all(exp::roadgame::evaluate_to_death_to_string(&mut best).as_bytes());

            //evaluate(&mut best, true);
            //println!("genome: {:?}", &best.genome);
            //println!("network: {:?}", &pop::Organism::new(&best.genome).network);
            best.genome.compile_to_png(Path::new(&format!("networks/dot/{}.dot", i)),
                                       Path::new(&format!("networks/{}-{}.png", i, best.fitness)));



            /*for (j, species) in population.species.iter().enumerate() {
                species.best_genome.compile_to_png(Path::new(&format!("networks/dot/{}_{}.dot", i, j)),
                                                   Path::new(&format!("networks/{}_{}.png", i, j)));
            }*/
        }
    }

    let mut best = population.best_organism().unwrap().clone();
    //evaluate(&mut best, true);
    exp::roadgame::evaluate_to_death(&mut best);
    println!("best fitness: {}", best.fitness);

    best.genome.compile_to_png(Path::new("best.dot"), Path::new("best.png"));
}

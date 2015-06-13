extern crate rand;

use std::path::Path;

mod genes; 
mod mutation;
mod nn;
mod pop;
mod mating;

fn evaluate(organism: &mut pop::Organism, print: bool) {
    let fitness = {
        let mut error = |x: bool, y: bool| -> f64 {
            organism.network.flush();
            organism.network.set_input(&vec![(0, x as f64 * 2.0 - 1.0), (1, y as f64 * 2.0 - 1.0)]);
            organism.network.activate();

            let output = organism.network.get_output()[0].1;
            let expected_output = (x != y) as f64 * 2.0 - 1.0;

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
        nodes: vec![genes::Node { id: 0, node_type: genes::NodeType::Input },
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
                    genes::Link { from_id: 2, to_id: 5, enabled: true, innovation: 6, weight: 0.0, is_recurrent: false }]
        /*nodes: vec![genes::Node { id: 0, node_type: genes::NodeType::Input },
                    genes::Node { id: 1, node_type: genes::NodeType::Input },
                    genes::Node { id: 2, node_type: genes::NodeType::Bias },
                    genes::Node { id: 3, node_type: genes::NodeType::Output }],
        links: vec![genes::Link { from_id: 0, to_id: 3, enabled: true, innovation: 0, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 1, to_id: 3, enabled: true, innovation: 1, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 2, to_id: 3, enabled: true, innovation: 2, weight: 0.0, is_recurrent: false }]*/
    };

    let mut i = 0;
    let mut rng = rand::thread_rng();
    let mut node_counter = 4;

    /*let mut genome2 = genome.clone();
    mutation::change_link_weights_reset_all(&mut genome2, &mut rng, 1.0);
    println!("{:?}", &genome2);

    println!("{}", genes::compatibility(&genes::STANDARD_COMPAT_COEFFICIENTS, &genome, &genome2));
    return;*/

    let mut population = pop::Population::from_initial_genome(&mut rng,
                                                              &pop::STANDARD_SETTINGS,
                                                              &mutation::Settings { recurrent_link_prob: 0.0, .. mutation::STANDARD_SETTINGS},
                                                              &genes::STANDARD_COMPAT_COEFFICIENTS,
                                                              &genome,
                                                              1000);

    loop {
        i += 1;

        if i > 500 { 
            break;
        }

        for species in population.species.iter_mut() {
            for organism in species.organisms.iter_mut() {
                evaluate(organism, false);
            }
        }

        {
            let mut best = population.best_organism().unwrap().clone();

            evaluate(&mut best, true);
            //println!("genome: {:?}", &best.genome);
            //println!("network: {:?}", &pop::Organism::new(&best.genome).network);



            for (j, species) in population.species.iter().enumerate() {
                let mut best = species.best_organism().clone();  
                /*for (k, organism) in species.organisms.iter().enumerate() {
                    organism.genome.compile_to_png(Path::new(&format!("networks/dot/{}_{}_{}.dot", i, j, k)),
                                                   Path::new(&format!("networks/{}_{}_{}_{}.png", i, j, k,
                                                                      organism.fitness)));
                }*/

                best.genome.compile_to_png(Path::new(&format!("networks/dot/{}_{}.dot", i, j)),
                                           Path::new(&format!("networks/{}_{}-{}.png", i, j,
                                                              best.fitness)));
            }
            //return;
        }

        println!("Generation {}", i);
        population.epoch(&mut rng);
        println!("");
    }

    for species in population.species.iter_mut() {
        for organism in species.organisms.iter_mut() {
            evaluate(organism, false);
        }
    }

    let mut best = population.best_organism().unwrap().clone();
    evaluate(&mut best, true);
    println!("best fitness: {}", best.fitness);

    best.genome.compile_to_png(Path::new("best.dot"), Path::new("best.png"));
}

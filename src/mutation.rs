extern crate rand;

use std::f32;
use std::collections::BTreeSet;
use rand::Rng;
use genes;
use std::collections::HashMap;

pub type Prob = f64;

pub struct MutationSettings {
    pub compat_c1: f64,
    pub compat_c2: f64,
    pub compat_c3: f64,
    pub compat_threshold: f64,

    pub add_node_prob: f32,
    pub add_link_prob: f32,

    pub link_weight_prob: f32,
    pub uniform_perturbation_prob: f32,

    pub disable_gene_prob: f32,

    pub no_crossover_prob: f32,
    pub interspecies_mating_rate: f32,

    pub keep_champion_min_species_size: i32,
    pub no_stagnant_reproduction_generations: i32,
}

/// We keep track of new link / new node mutations that happen in a generation as 'innovations'.
/// Roughly, we wish to give genes that are created due to the same mutation the same innovation number.
/// The innovation numbers are then used during crossover to determine matching genes.

/// Records the nodes that were connected in a new link mutation
#[derive(PartialEq, Eq, Hash)]
pub struct NewLinkInnovation {
    from_id: genes::NodeId,
    to_id: genes::NodeId,
}

/// Stores for each innovation parameter the innovation number that is assigned to the genes
pub type NewLinkInnovations = HashMap<NewLinkInnovation, usize>;

/// Records the link that was split to create a new node inbetween
#[derive(PartialEq, Eq, Hash)]
pub struct NewNodeInnovation {
    from_id: genes::NodeId,
    to_id: genes::NodeId,
    old_innovation: usize, 
}

/// Stores for each innovation parameter the innovation numbers of the two new links
pub type NewNodeInnovations = HashMap<NewNodeInnovation, (usize, usize)>;

pub static STANDARD_MUTATION_SETTINGS : MutationSettings =
    MutationSettings {
        compat_c1: 1.0,
        compat_c2: 1.0,
        compat_c3: 0.4,
        compat_threshold: 3.0,
        
        add_node_prob: 0.03,
        add_link_prob: 0.05,

        link_weight_prob: 0.8,
        uniform_perturbation_prob: 0.9,

        disable_gene_prob: 0.75,

        no_crossover_prob: 0.25,
        interspecies_mating_rate: 0.001,

        keep_champion_min_species_size: 5,
        no_stagnant_reproduction_generations: 15
    };                   

pub fn mutate(settings: &MutationSettings, genome: &mut genes::Genome) {
    let num_mutations = (genome.num_nodes() as f32).sqrt().floor();
}

pub fn rand_pos_neg<R: rand::Rng>(rng: &mut R) -> f64 {
    match rng.gen::<bool>() {
        true => 1.0,
        false => -1.0
    }
}

/// Add a new node to the genome by inserting it in the middle of an existing link between two nodes.
/// This function takes a set of node innovations that happened in this generation so far as a parameter.
pub fn mutation_add_node<R: rand::Rng>(rng: &mut R,
                                       innovations: &mut NewNodeInnovations,
                                       innovation_counter: &mut usize,
                                       node_counter: &mut usize,
                                       genome: &mut genes::Genome) {
    // Select a link gene to split up. The link must not be in a disabled state. 
    let enabled_gene_indices = 
        genome.links.iter().enumerate()
              .filter(|&(i, link)| link.enabled)
              .map(|(i, link)| i)
              .collect::<Vec<usize>>();
        //genome.links.iter_mut().filter(|link| link.enabled).collect::<Vec<&mut genes::Link>>();
    
    match rng.choose(&enabled_gene_indices) {
        Some(index) => {
            let (link1, link2, node) = {
                let link = &mut genome.links[*index];

                link.enabled = false;

                // Has this innovation already happened this generation?
                // If so, we will use the same innovation numbers for our new link genes.
                let new_node_innov = NewNodeInnovation { from_id: link.from_id,
                                                         to_id: link.to_id,
                                                         old_innovation: link.innovation };

                let (innovation1, innovation2) = match innovations.get(&new_node_innov) {
                    Some(numbers) => *numbers, 
                    None => {
                        // We have a new innovation
                        *innovation_counter += 2;
                        (*innovation_counter-2, *innovation_counter-1)
                    }
                };

                let new_node_id = *node_counter;
                *node_counter += 1;

                // Now we can create the new genes
                let link1 = genes::Link { from_id: link.from_id, 
                                          to_id: new_node_id,
                                          enabled: true,
                                          innovation: innovation1,
                                          weight: 1.0 };
                let link2 = genes::Link { from_id: new_node_id,
                                          to_id: link.to_id,
                                          enabled: true,
                                          innovation: innovation2,
                                          weight: link.weight };
                let node = genes::Node { id: new_node_id,
                                         bias: 0.0 };

                (link1, link2, node)
            };

            genome.links.push(link1);
            genome.links.push(link2);
            genome.nodes.push(node);
        }

        None => ()
    }
}

pub enum LinkMutation {
    Perturbate,
    Reset,
    None
}

pub fn rand_link_mutation<R: rand::Rng>(rng: &mut R, perturbate_point: f64, reset_point: f64) -> LinkMutation {
    let rand_choice: f64 = rng.next_f64();

    if rand_choice > perturbate_point {
        LinkMutation::Perturbate
    } else if rand_choice > reset_point {
        LinkMutation::Reset
    } else {
        LinkMutation::None
    }
}

/// Apply a link weight mutation to each gene in the genome.
/// F chooses for each link gene a mutation to be applied (depending on the position in the genome):
/// * Perturb adds a random value in `(-power,power)` to the link weight.
/// * Reset sets the link weight to a random value in `(-power,power)`.
/// * None leaves the link weight unmodified.
pub fn mutate_link_weights<R: rand::Rng, F: FnMut(&mut R, usize) -> LinkMutation>(rng: &mut R, mut f: F, power: f64, genome: &mut genes::Genome) {
    for (position, ref mut link) in genome.links.iter_mut().enumerate() {
        match f(rng, position) {
            LinkMutation::Perturbate =>
                link.weight += rand_pos_neg(rng) * rng.next_f64() * power,
            LinkMutation::Reset =>
                link.weight = rand_pos_neg(rng) * rng.next_f64() * power,
            LinkMutation::None => (),
        }
    }
}

pub fn mutate_link_weights_standard<R: rand::Rng>(rng: &mut R, rate: f64, power: f64, genome: &mut genes::Genome) {
    let severe = rng.gen::<bool>();
    let num_links = genome.links.len();

    let f = |rng: &mut R, position: usize| -> LinkMutation {
        if severe { 
            // If `severe` is true, use high probabilities for perturbation and reset
            rand_link_mutation(rng, 0.3, 0.1)
        }
        else if num_links > 10 && position as f64 >= (num_links as f64) * 0.8 {
            // If we have a reasonably large genome (more than 10 link genes),
            // and we are in the newer part of the genes, use high probability for reset.
            // Since these are the new genes, it is assumed that they need more adjustment still.
            rand_link_mutation(rng, 0.5, 0.3)
        }
        else if rng.gen::<bool>() {
            // Otherwise, sometimes disallow reset mutations...
            // This is achieved by setting perturbatePoint and resetPoint to the same value.
            rand_link_mutation(rng, 1.0-rate, 1.0-rate)
        } else {
            rand_link_mutation(rng, 1.0-rate, 1.0-rate-0.1)
        }
    };

    mutate_link_weights(rng, f, power, genome);
}

pub fn mutate_link_weights_reset_all<R: rand::Rng, F: Fn(usize) -> LinkMutation>(rng: &mut R, power: f64, genome: &mut genes::Genome) {
    mutate_link_weights(rng, |_, _| LinkMutation::Reset, power, genome);
}

extern crate rand;

use std::f32;
use std::collections::BTreeSet;
use std::collections::HashMap;
use rand::Rng;
use genes;

pub type Prob = f64;

#[derive(Clone)]
pub struct Settings {
    // Probabilities for specific genome mutations
    pub new_node_prob: Prob,
    pub new_link_prob: Prob,

    pub change_link_weights_prob: Prob,
    pub change_link_weights_power: f64,
    pub uniform_perturbation_prob: Prob,

    pub recurrent_link_prob: Prob,
    pub self_link_prob: Prob,

    pub toggle_enable_prob: Prob,

    // Probabilities for different kinds of reproduction
    pub mutate_only_prob: Prob,
    pub mutate_after_mating_prob: Prob,
    pub no_crossover_prob: Prob,
    pub interspecies_mating_prob: Prob,
}

pub static STANDARD_SETTINGS: Settings =
    Settings {
        new_node_prob: 0.01,
        new_link_prob: 0.3,

        change_link_weights_prob: 0.8,
        change_link_weights_power: 0.5,
        uniform_perturbation_prob: 0.9,

        recurrent_link_prob: 0.3,
        self_link_prob: 0.5,

        toggle_enable_prob: 0.05,

        mutate_only_prob: 0.25,
        mutate_after_mating_prob: 0.8,
        no_crossover_prob: 0.25,
        interspecies_mating_prob: 0.001,
    };                   

/// We keep track of new link / new node mutations that happen in a generation as 'innovations'.
/// Roughly, we wish to give genes that are created due to the same mutation the same innovation number.
/// The innovation numbers are then used during crossover to determine matching genes.

/// Records the nodes that were connected in a new link mutation
#[derive(PartialEq, Eq, Hash)]
pub struct NewLinkInnovation {
    from_id: genes::NodeId,
    to_id: genes::NodeId,
    is_recurrent: bool,
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

/// Stores for each innovation parameter the node id of the created node
/// and the innovation numbers of the two new links
pub type NewNodeInnovations = HashMap<NewNodeInnovation, (genes::NodeId, usize, usize)>;

/// Population-level state that is needed while mutating
pub struct State {
    // Counter for creating new node IDs
    pub node_counter: usize,

    // Keep track of innovations
    pub innovation_counter: usize,
    pub link_innovations: NewLinkInnovations,
    pub node_innovations: NewNodeInnovations,
}

pub fn mutate<R: rand::Rng>(genome: &mut genes::Genome,
                            settings: &Settings,
                            rng: &mut R,
                            state: &mut State) {
    if rng.next_f64() < settings.new_node_prob {
        //println!("NEW NODE");
        new_node(genome, rng, &mut state.node_innovations,
                              &mut state.innovation_counter,
                              &mut state.node_counter);
    } else if rng.next_f64() < settings.new_link_prob {
        //println!("NEW LINK");
        new_link(genome, rng,
                 &mut state.link_innovations,
                 &mut state.innovation_counter,
                 settings.recurrent_link_prob,
                 settings.self_link_prob, 30);
    } else {
        if rng.next_f64() < settings.toggle_enable_prob {
            toggle_enable(genome, rng);
        }

        if rng.next_f64() < settings.change_link_weights_prob {
            let p = 1.0 / (genome.links.len() as f64).sqrt();
            change_link_weights_standard(genome, rng, 1.0, settings.change_link_weights_power);
            //change_link_weights_perturbate_some(genome, rng, 0.3, settings.change_link_weights_power);
        }
    }
}

fn rand_pos_neg<R: rand::Rng>(rng: &mut R) -> f64 {
    match rng.gen::<bool>() {
        true => 1.0,
        false => -1.0
    }
}

/// Toggles the enabled status of a random link gene
pub fn toggle_enable<R: rand::Rng>(genome: &mut genes::Genome, rng: &mut R) {
    let gene_indices = (0..genome.links.len()).collect::<Vec<usize>>();

    match rng.choose(&gene_indices) {
        Some(index) => {
            if genome.links[*index].enabled {
                // Check that the in node of the link has another outgoing link
                if genome.successor_links(genome.links[*index].from_id)
                               .iter()
                               .filter(|link| link.enabled)
                               .count() == 1 {
                    // If so, we can disable this one
                    genome.links[*index].enabled = false;
                }
            } else {
                genome.links[*index].enabled = true;
            }
        },

        None => ()
    }
}

/// Reenable the first gene we can find


/// Add a new node to the genome by inserting it in the middle of an existing link between two nodes.
/// This function takes a set of node innovations that happened in this generation so far as a parameter.
pub fn new_node<R: rand::Rng>(genome: &mut genes::Genome,
                              rng: &mut R,
                              innovations: &mut NewNodeInnovations,
                              innovation_counter: &mut usize,
                              node_counter: &mut usize) {
    // Select a link gene to split up. The link must not be in a disabled state. 
    let enabled_gene_indices = 
        genome.links.iter().enumerate()
              .filter(|&(i, link)| link.enabled)
              .map(|(i, link)| i)
              .collect::<Vec<usize>>();
    
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

                //println!("split from {} to {} old {}", link.from_id, link.to_id, link.innovation);

                let (is_new, (new_node_id, innovation1, innovation2)) = match innovations.get(&new_node_innov) {
                    Some(numbers) => (false, *numbers), 
                    None => {
                        //println!("new innovation");
                        // We have a new innovation
                        *node_counter += 1;
                        *innovation_counter += 2;

                        let numbers = (*node_counter-1, *innovation_counter-2, *innovation_counter-1);
                        (true, numbers)
                    }
                };

                if is_new {
                    innovations.insert(new_node_innov, (new_node_id, innovation1, innovation2));
                }

                // Now we can create the new genes
                let link1 = genes::Link { from_id: link.from_id, 
                                          to_id: new_node_id,
                                          enabled: true,
                                          innovation: innovation1,
                                          weight: 1.0,
                                          is_recurrent: link.is_recurrent };
                let link2 = genes::Link { from_id: new_node_id,
                                          to_id: link.to_id,
                                          enabled: true,
                                          innovation: innovation2,
                                          weight: link.weight,
                                          is_recurrent: false }; // ???
                let node = genes::Node { id: new_node_id,
                                         node_type: genes::NodeType::Hidden };

                (link1, link2, node)
            };

            genome.nodes.push(node);
            genome.add_link(link1);
            genome.add_link(link2);
        }

        None => ()
    }
}

/// Add a new link between two nodes. The two nodes are selected at random.
/// We make `num_tries` tries to find two compatible nodes.
pub fn new_link<R: rand::Rng>(genome: &mut genes::Genome,
                              rng: &mut R,
                              innovations: &mut NewLinkInnovations,
                              innovation_counter: &mut usize,
                              recurrent_link_prob: Prob,
                              self_link_prob: Prob,
                              num_tries: usize) {
    let node_ids =
        genome.nodes.iter()
              .map(|node| node.id)
              .collect::<Vec<usize>>();

    let hidden_node_ids = 
        genome.nodes.iter()
              .filter(|node| node.node_type == genes::NodeType::Hidden)
              .map(|node| node.id)
              .collect::<Vec<usize>>();

    if node_ids.is_empty() || hidden_node_ids.is_empty() {
        return;
    }

    // Decide whether to create a recurrent or a feed-forward link
    let recurrent = rng.next_f64() < recurrent_link_prob;

    //if recurrent { println!("{}", "creating recurrent link"); }

    // Randomly select from and to node until they fit our criterion
    let mut from_id = 0;
    let mut to_id = 0;
    let mut found = false;

    for try in 0..num_tries {
        if recurrent && rng.next_f64() < self_link_prob {
            // Sometimes make a self loop
            from_id = *rng.choose(&hidden_node_ids).unwrap();
            to_id = to_id 
        } else {
            from_id = *rng.choose(&node_ids).unwrap();
            to_id = *rng.choose(&hidden_node_ids).unwrap();
        }

        if !recurrent && from_id == to_id {
            continue;
        }

        if !genome.is_link(from_id, to_id) &&
           genome.is_new_link_recurrent(from_id, to_id) == recurrent {
            found = true;
            break;
        }
    }

    if !found {
        return
    }

    let link = {
        let from_node = genome.get_node(from_id).unwrap();
        let to_node = genome.get_node(to_id).unwrap();

        // See if this innovation has already happened in this generation.
        // If yes, we will use the same innovation number for the new link gene.
        let new_link_innov = NewLinkInnovation { from_id: from_node.id,
                                                 to_id: to_node.id,
                                                 is_recurrent: recurrent };

        let (is_new, innovation) = match innovations.get(&new_link_innov) {
            Some(innovation) => (false, *innovation), 
            None => {
                // We have a new innovation
                *innovation_counter += 1;
                (true, *innovation_counter-1)
            }
        };

        if is_new {
            innovations.insert(new_link_innov, innovation);
        }

        // Create the new link gene    
        let weight = rand_pos_neg(rng) * rng.next_f64();
        let link = genes::Link { from_id: from_node.id, 
                                 to_id: to_node.id,
                                 enabled: true,
                                 innovation: innovation,
                                 weight: weight,
                                 is_recurrent: recurrent };
        link
    };

    genome.add_link(link);
}

pub enum LinkMutation {
    Perturbate,
    Reset,
    None
}

/// Apply a link weight mutation to each gene in the genome.
/// F chooses for each link gene a mutation to be applied (depending on the position in the genome):
/// * Perturb adds a random value in `(-power,power)` to the link weight.
/// * Reset sets the link weight to a random value in `(-power,power)`.
/// * None leaves the link weight unmodified.
pub fn change_link_weights<R: rand::Rng, F: FnMut(&mut R, usize) -> LinkMutation>(genome: &mut genes::Genome, rng: &mut R, mut f: F, power: f64) {
    for (position, link) in genome.links.iter_mut().enumerate() {
        match f(rng, position) {
            LinkMutation::Perturbate =>
                link.weight += rand_pos_neg(rng) * rng.next_f64() * power,
            LinkMutation::Reset =>
                link.weight = rand_pos_neg(rng) * rng.next_f64() * power,
            LinkMutation::None => (),
        }

        if link.weight > 8.0 { link.weight = 8.0; }
        if link.weight < -8.0 { link.weight = -8.0; }
    }
}

fn rand_link_mutation<R: rand::Rng>(rng: &mut R, perturbate_point: f64, reset_point: f64) -> LinkMutation {
    let rand_choice: f64 = rng.next_f64();

    if rand_choice > perturbate_point {
        LinkMutation::Perturbate
    } else if rand_choice > reset_point {
        LinkMutation::Reset
    } else {
        LinkMutation::None
    }
}

pub fn change_link_weights_standard<R: rand::Rng>(genome: &mut genes::Genome, rng: &mut R, rate: f64, power: f64) {
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

    change_link_weights(genome, rng, f, power);
}

pub fn change_link_weights_perturbate_some<R: rand::Rng>(genome: &mut genes::Genome, rng: &mut R, prob: f64, power: f64) {
    change_link_weights(genome, rng, |rng, _| if rng.next_f64() < prob { LinkMutation::Perturbate } else { LinkMutation::None }, power);
}

pub fn change_link_weights_reset_all<R: rand::Rng>(genome: &mut genes::Genome, rng: &mut R, power: f64) {
    change_link_weights(genome, rng, |_, _| LinkMutation::Reset, power);
}

use std::collections::HashMap;
use std::cell::RefCell;
use genes;

pub fn sigmoid(input_sum: f64) -> f64 {
    let slope = 4.924273;

    (1.0 / (1.0 + (-slope*input_sum).exp()))
}

pub struct Node {
    gene: genes::Node,

    predecessor_indices: Vec<usize>,
    successor_indices: Vec<usize>,
    weights: Vec<f64>,
    bias: f64,
    node_type: genes::NodeType,

    active: bool,
    input_sum: f64,
    activation: f64,
}

pub struct Network {
    id_to_index: Box<HashMap<genes::NodeId, usize>>,
    nodes: Vec<Node>, // not ideal to use RefCell here I guess
}

impl Network {
    pub fn from_genome(genome: &genes::Genome) -> Network {
        let (id_to_index, nodes) = {
            let id_to_index = {
                let mut map = HashMap::new();

                for (index, node) in genome.nodes.iter().enumerate() {
                    assert!(!map.contains_key(&node.id), "Non-unique node ID in genome");
                    map.insert(node.id, index);
                }

                Box::new(map)
            };

            let nodes = genome.nodes.iter().map(
                |node| Node {
                    gene: *node,
                    predecessor_indices: genome.predecessor_links(node.id).iter()
                                               .map(|link| *id_to_index.get(&link.from_id).unwrap())
                                               .collect(),
                    successor_indices: genome.successor_links(node.id).iter()
                                             .map(|link| *id_to_index.get(&link.to_id).unwrap())
                                             .collect(),
                    weights: genome.predecessor_links(node.id).iter()
                                   .map(|link| link.weight)
                                   .collect(),
                    bias: node.bias,
                    node_type: node.node_type,
                    active: false,
                    input_sum: 0.0,
                    activation: 0.0,
                });

            (id_to_index.clone(), nodes.collect())
        };

        Network {
            id_to_index: id_to_index,
            nodes: nodes,
        }
    }

    pub fn num_inputs(&self) -> usize {
        self.nodes.iter().filter(|node| node.node_type == genes::NodeType::Input).count()
    }

    pub fn get_output(&self) -> Vec<(genes::NodeId, f64)> {
        self.nodes.iter().filter(|node| node.node_type == genes::NodeType::Output)
                         .map(|node| (node.gene.id, node.activation))
                         .collect()
    }

    pub fn are_outputs_activated(&self) -> bool {
        self.nodes.iter().filter(|node| node.node_type == genes::NodeType::Output)
                         .all(|node| node.active)
    }

    pub fn set_input(&mut self, input: &Vec<(genes::NodeId, f64)>) {
        for &(id, activation) in input.iter() {
            self.nodes[*self.id_to_index.get(&id).unwrap()].activation = activation;
        }
    }

    pub fn activate(&mut self) {
        for try in 0..50 {
            // Calculate input activation for each non-input node
            for node_index in 0..self.nodes.len() {
                let (active, input_sum) = {
                    let node = &self.nodes[node_index];

                    // Take the weighted sum of those inputs of the node that are activated.
                    // Activate when at least one of our inputs is activated.
                    if node.node_type == genes::NodeType::Input { 
                        continue;
                    }

                    println!("{}", node.gene.id);

                    node.predecessor_indices.iter()
                        .zip(node.weights.iter())
                        .fold((false, 0.0),
                              |(active, input_sum), (in_index, weight)| {
                                  let in_node = &self.nodes[*in_index];
                                  let in_active = in_node.active || in_node.node_type == genes::NodeType::Input;

                                  (active || in_active,
                                   if in_active { input_sum + weight * in_node.activation }
                                   else { input_sum })
                              })
                };

                // Update state in array
                self.nodes[node_index].active = active;
                self.nodes[node_index].input_sum = input_sum;
            }

            // Calculate activation of each node based on the input we just calculated
            for node in self.nodes.iter_mut() {
                if node.node_type == genes::NodeType::Input {
                    continue;
                }

                if node.active {
                    node.activation = sigmoid(node.input_sum);
                    println!("activate {} with {}", node.gene.id, node.activation);
                }
            }

            if self.are_outputs_activated() {
                break;
            }
        }

        if !self.are_outputs_activated() {
            println!("couldn't activate all outputs in time");
        }
    }
}

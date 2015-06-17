use std::collections::HashMap;
use std::cell::RefCell;
use std::cmp;
use genes;

pub fn sigmoid(input_sum: f64) -> f64 {
    let slope = 4.924273;

    //(1.0 / (1.0 + (-slope*input_sum).exp()))
    input_sum.tanh()
}

#[derive(Debug, Clone)]
pub struct Node {
    gene: genes::Node,

    predecessor_indices: Vec<usize>,
    successor_indices: Vec<usize>,
    weights: Vec<f64>,
    node_type: genes::NodeType,

    depth: Option<usize>,

    active: bool,
    input_sum: f64,
    activation: f64,
}

#[derive(Debug, Clone)]
pub struct Network {
    id_to_index: Box<HashMap<genes::NodeId, usize>>,
    nodes: Vec<Node>, 
    max_depth: usize, // Length of the longest path contained in the network
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
                    node_type: node.node_type,
                    depth: None,
                    active: false,
                    input_sum: 0.0,
                    activation: if node.node_type == genes::NodeType::Bias { 1.0 } else { 0.0 },
                });

            (id_to_index.clone(), nodes.collect::<Vec<Node>>())
        };

        for (index, node) in nodes.iter().enumerate() {
            assert_eq!(index, *id_to_index.get(&node.gene.id).unwrap());

            for succ_index in node.successor_indices.iter() {
                let succ_node = &nodes[*succ_index];

                assert!(genome.is_link(node.gene.id, succ_node.gene.id));
            }
            
            for i in 1..node.predecessor_indices.len() {
                let pred_node = &nodes[node.predecessor_indices[i]];
                assert!(genome.is_link(pred_node.gene.id, node.gene.id));

                assert_eq!(genome.get_link(pred_node.gene.id, node.gene.id).unwrap().weight, 
                           node.weights[i]);
            }
        }

        let mut network = Network {
            id_to_index: id_to_index,
            nodes: nodes,
            max_depth: 0,
        };

        network.calc_depths();
        network
    }

    /// Maximl number of links from an input to an output node
    fn calc_depths(&mut self) {
        /*// Depth search

        let mut visited = BTreeSet::new();
        let mut queue = self.nodes.iter().enumerate()
                                         .filter(|(i, node)| node.node_type == genes::NodeType::Input)
                                         .map(|(i, node)| (i, 0))
                                         .collect::<Vec<Node>>();

        while let Some(node_index, depth) = queue.pop() {
            visited.insert(node_index);

            let node = &mut self.nodes[node_index];
            node.depth = match node.depth {
                Some(old_depth) => Some(cmp::min(old_depth, depth));
                None => Some(depth);
            };

            for link in self.successor_links(node_id) {
                if visited.contains(&link.to_id) || link.is_recurrent {
                    continue
                }

                queue.push(link.to_id);
            }
        }

        return false;*/
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

    /// Resets the state of the network, setting all nodes to inactive
    pub fn flush(&mut self) {
        for node in self.nodes.iter_mut() {
            node.active = false;
            node.activation = if node.node_type == genes::NodeType::Bias { 1.0 } else { 0.0 };
        }
    }

    pub fn activate(&mut self) {
        for node in self.nodes.iter_mut() {
            /*if node.node_type == genes::NodeType::Input {
                println!("INPUT: {}, {}", node.gene.id, node.activation);
            }
            if node.node_type == genes::NodeType::Bias {
                println!("BIAS: {}, {}", node.gene.id, node.activation);
            }*/
            if node.node_type == genes::NodeType::Bias {
                assert_eq!(node.activation, 1.0);
            }
        }

        for try in 0..50 {
            // Calculate input activation for each non-input node
            for node_index in 0..self.nodes.len() {
                let (active, input_sum) = {
                    let node = &self.nodes[node_index];

                    if node.node_type == genes::NodeType::Input ||
                       node.node_type == genes::NodeType::Bias { 
                        continue;
                    }

                    // Take the weighted sum of those inputs of the node that are activated.
                    // Activate when at least one of our inputs is activated.
                    node.predecessor_indices.iter()
                        .zip(node.weights.iter())
                        .fold((false, 0.0),
                              |(active, input_sum), (in_index, weight)| {
                                  let in_node = &self.nodes[*in_index];
                                  let in_active = in_node.active || in_node.node_type == genes::NodeType::Input
                                                                 || in_node.node_type == genes::NodeType::Bias;

                                  if in_active {
                                      //println!("{} gets {} * {} from {}", node.gene.id, weight, in_node.activation, in_node.gene.id);
                                  }

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
                if node.node_type == genes::NodeType::Input ||
                   node.node_type == genes::NodeType::Bias {
                    continue;
                }

                if node.active {
                    node.activation = sigmoid(node.input_sum);
                    //println!("activate {} with {} -> {}", node.gene.id, node.input_sum, node.activation);
                }
            }

            if self.are_outputs_activated() {
                break;
            }
        }

        if !self.are_outputs_activated() {
            //println!("couldn't activate all outputs in time");
        }
    }
}

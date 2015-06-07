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
    nodes: Vec<RefCell<Node>>, // not ideal to use RefCell here I guess
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
                |node| RefCell::new(Node {
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
                }));

            (id_to_index.clone(), nodes.collect())
        };

        Network {
            id_to_index: id_to_index,
            nodes: nodes,
        }
    }

    pub fn num_inputs(&self) -> usize {
        self.nodes.iter().filter(|node| node.borrow().node_type == genes::NodeType::Input).count()
    }

    pub fn get_output(&self) -> Vec<(genes::NodeId, f64)> {
        self.nodes.iter().filter(|node| node.borrow().node_type == genes::NodeType::Output)
                         .map(|node| (node.borrow().gene.id, node.borrow().activation))
                         .collect()
    }

    pub fn are_outputs_activated(&self) -> bool {
        self.nodes.iter().all(|node| node.borrow().node_type == genes::NodeType::Output)
    }

    pub fn activate(&mut self, input: &Vec<(genes::NodeId, f64)>) {
        // Set input activation to given values
        for &(id, activation) in input.iter() {
            self.nodes[*self.id_to_index.get(&id).unwrap()].borrow_mut().activation = activation;
        }

        loop {
            // Calculate input activation for each non-input node
            for node in self.nodes.iter() {
                if node.borrow().node_type == genes::NodeType::Input { 
                    continue;
                }

                // Take the weighted sum of those inputs of the node that are activated.
                // Activate when at least one of our inputs is activated.
                node.borrow_mut().input_sum = 0.0;

                for (predecessor_index, weight) in node.borrow_mut().predecessor_indices.iter().zip(node.borrow_mut().weights.iter()) {
                    let predecessor_node = self.nodes[*predecessor_index].borrow();

                    if predecessor_node.active || predecessor_node.node_type == genes::NodeType::Input {
                        node.borrow_mut().active = true; 
                        node.borrow_mut().input_sum += weight * predecessor_node.activation;
                    }
                }
            }

            // Calculate activation of each node based on the input we just calculated
            for node in self.nodes.iter() {
                if node.borrow().node_type == genes::NodeType::Input { 
                    continue;
                }

                if node.borrow().active {
                    let activation = sigmoid(node.borrow().input_sum);
                    node.borrow_mut().activation = activation;
                }
            }

            if self.are_outputs_activated() {
                break;
            }
        }
    }
}

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::process::Command;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;

use rustc_serialize::json::{self, ToJson, Json};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, RustcEncodable, RustcDecodable)]
pub enum NodeType {
    Input,
    Output,
    Hidden,
    Bias,
}

pub type NodeId = usize;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, RustcEncodable, RustcDecodable)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, RustcEncodable, RustcDecodable)]
pub struct Link {
    pub from_id: NodeId,
    pub to_id: NodeId,
    pub enabled: bool,
    pub innovation: usize,
    pub weight: f64,
    pub is_recurrent: bool,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Genome {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>, // Sorted by innovation number in increasing order
}

#[derive(Clone, Debug)]
pub struct CompatCoefficients {
    pub disjoint: f64,
    pub excess: f64,
    pub weight_diff: f64,
}

pub static STANDARD_COMPAT_COEFFICIENTS: CompatCoefficients = CompatCoefficients {
    disjoint: 1.0,
    excess: 1.0,
    weight_diff: 0.4,
};

pub fn compatibility(c: &CompatCoefficients,
                     genome_a: &Genome, genome_b: &Genome) -> f64 {
    let mut i = 0;
    let mut j = 0;

    let mut num_disjoint = 0;
    let mut num_excess = 0;

    let mut num_matching = 0;
    let mut weight_diff = 0.0;

    // Iterate through the links of both genomes, counting matches in innovation
    while i < genome_a.links.len() || j < genome_b.links.len() {
        if i == genome_a.links.len() {
            j += 1;
            num_excess += 1;
            continue;
        }
        if j == genome_b.links.len() {
            i += 1;
            num_excess += 1;
            continue;
        }

        let gene_a = &genome_a.links[i];
        let gene_b = &genome_b.links[j];

        if gene_a.innovation == gene_b.innovation {
            weight_diff += (gene_a.weight - gene_b.weight).abs();
            num_matching += 1;
            i += 1;
            j += 1;
        } else if gene_a.innovation > gene_b.innovation {
            num_disjoint += 1;
            j += 1;
        } else {
            num_disjoint += 1;
            i += 1;
        }
    }

    assert!(num_matching > 0);

    return c.disjoint * num_disjoint as f64 +
           c.excess * num_excess as f64 +
           c.weight_diff * (weight_diff / num_matching as f64);
}

impl Genome {
    pub fn initial_genome(num_inputs: usize, num_outputs: usize, num_connected: usize, bias_connected: bool) -> Genome {
        assert!(num_connected <= num_inputs);

        let mut genome = Genome { nodes: vec![], links: vec![] };
        let mut node_counter = 0;
        let mut innovation_counter = 0;

        for _ in 0..num_inputs {
            genome.nodes.push(Node { id: node_counter, node_type: NodeType::Input });
            node_counter += 1;
        }

        genome.nodes.push(Node { id: node_counter, node_type: NodeType::Bias });
        node_counter += 1;

        for _ in 0..num_outputs {
            genome.nodes.push(Node { id: node_counter, node_type: NodeType::Output });

            if bias_connected {
                genome.add_link(Link {
                    from_id: num_inputs,
                    to_id: node_counter,
                    enabled: true,
                    innovation: innovation_counter,
                    weight: 0.0,
                    is_recurrent: false,
                });
                innovation_counter += 1;
            }

            for x in 0..num_connected {
                genome.add_link(Link {
                    from_id: x,
                    to_id: node_counter,
                    enabled: true,
                    innovation: innovation_counter,
                    weight: 0.0,
                    is_recurrent: false,
                });

                innovation_counter += 1;
            }

            node_counter += 1;
        }

        genome
    }

    pub fn assert_integrity(&self) {
        for link in self.links.iter() {
            assert!(self.is_link(link.from_id, link.to_id));
            assert!(self.is_node(link.from_id));
            assert!(self.is_node(link.to_id));
        }

        for node in self.nodes.iter() { 
            assert!(self.is_node(node.id));

            if node.node_type == NodeType::Input {
                assert_eq!(self.predecessor_links(node.id).len(), 0);
            }
        }

        {
            //assert!(self.links.len() > 0);

            if self.links.len() > 0 {
                for i in 0..self.links.len()-1 {
                    assert!(self.links[i].innovation < self.links[i+1].innovation);
                }
            }
        }
    }

    /// Adds a new link to the genome, keeping the list sorted by innovation number
    pub fn add_link(&mut self, new_link: Link) {
        assert!(self.is_node(new_link.from_id));
        assert!(self.is_node(new_link.to_id));
        assert!(!self.is_link(new_link.from_id, new_link.to_id));

        //println!("{:?}", self.links.iter().map(|l| l.innovation).collect::<Vec<usize>>());

        // Try to find the position of the first link having a bigger innovation number than the new link
        match self.links.iter().position(|link| link.innovation > new_link.innovation) {
            Some(position) =>
                self.links.insert(position, new_link),
            None =>
                self.links.push(new_link)
        }
        //println!("=> {:?}", self.links.iter().map(|l| l.innovation).collect::<Vec<usize>>());
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len() //self.nodes.iter().filter(|node| node.enabled).count();
    }

    pub fn num_links(&self) -> usize {
        self.links.iter().filter(|link| link.enabled).count()
    }

    pub fn is_link(&self, from_id: NodeId, to_id: NodeId) -> bool {
        let count = self.links.iter()
                              .filter(|link| link.from_id == from_id && link.to_id == to_id)
                              .count();
        assert!(count == 0 || count == 1, "More than one link between the same nodes");

        return count == 1;
    }

    pub fn get_link(&self, from_id: NodeId, to_id: NodeId) -> Option<&Link> {
        self.links.iter().filter(|link| link.from_id == from_id && link.to_id == to_id).next()
    }

    pub fn is_node(&self, id: NodeId) -> bool {
        let count = self.nodes.iter().filter(|node| node.id == id).count();
        assert!(count == 0 || count == 1, "More than one node with the same id");

        return count == 1;
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().filter(|node| node.id == id).next()
    }

    pub fn successor_links(&self, from_id: NodeId) -> Vec<&Link> {
        // This kind of stuff is quite inefficient with the representation,
        // will need to see if it has to get optimized
        assert!(self.is_node(from_id));

        return self.links.iter()
                         .filter(|link| link.enabled && link.from_id == from_id)
                         .collect();
    }

    pub fn predecessor_links(&self, to_id: NodeId) -> Vec<&Link> {
        // This kind of stuff is quite inefficient with the representation,
        // will need to see if it has to get optimized
        assert!(self.is_node(to_id));

        return self.links.iter()
                         .filter(|link| link.enabled && link.to_id == to_id)
                         .collect();
    }

    /// Checks if a new link would be considered recurrent in the network.
    /// This allows us to control the frequency with which recurrent loops are added by mutations.
    pub fn is_new_link_recurrent(&self, from_id: NodeId, to_id: NodeId) -> bool {
        // Starting from `to_id`, try to find `from_id` using depth search.
        // Here, we only use links that are considered feed forward in the network.

        let mut visited = BTreeSet::new();
        let mut queue = vec![to_id];

        while let Some(node_id) = queue.pop() {
            visited.insert(node_id);

            if node_id == from_id {
                return true;
            }

            for link in self.successor_links(node_id) {
                if visited.contains(&link.to_id) || link.is_recurrent {
                    continue
                }

                queue.push(link.to_id);
            }
        }

        return false;
    }

    pub fn to_dot_string(&self, names: HashMap<NodeId, String>) -> String {
        let mut str = String::new();

        str.push_str("digraph g {\n");
        //str.push_str("layout=fdp;");

        let get_name = |node: &Node| {
            let mut name = match names.get(&node.id) {
                Some(name) => name.clone(),
                None => "#".to_string() + &node.id.to_string()
            };

            if node.node_type == NodeType::Input {
                name = name + &">";
            } else if node.node_type == NodeType::Output {
                name = name + &"<";
            }

            "\"".to_string() + &name + &"\""
        };

        for node in self.nodes.iter() {
            str.push_str(&get_name(node));
            str.push_str(" [color=none, shape=plaintext");
            //str.push_str(", fontsize=12");
            str.push_str("];");
            str.push_str("\n");
        }

        for link in self.links.iter() {
            if !link.enabled {
                continue;
            }

            let from_name = match names.get(&link.from_id) {
                Some(name) => name.clone(),
                None => link.from_id.to_string()
            };
            let to_name = match names.get(&link.to_id) {
                Some(name) => name.clone(),
                None => link.to_id.to_string()
            };

            let color = if link.weight < 0.0 { "blue".to_string() } else { "orange".to_string() };
            let width = link.weight.abs().to_string();
            let label = &format!("{:.2}", link.weight);

            str.push_str(&get_name(self.get_node(link.from_id).unwrap()));
            str.push_str(" -> ");
            str.push_str(&get_name(self.get_node(link.to_id).unwrap()));
            str.push_str(" [label=");
            str.push_str(&label);
            str.push_str(", penwidth=");
            str.push_str(&width);
            str.push_str(", color=");
            str.push_str(&color);
            str.push_str(", fontsize=10");
            str.push_str("];");
            str.push_str("\n");
        }

        str.push_str("}\n");

        return str;
    }

    pub fn compile_to_png(&self, names: HashMap<NodeId, String>,
                          dot_path: &Path, png_path: &Path) -> io::Result<()> {
        let mut f = try!(File::create(dot_path));
        try!(f.write_all(self.to_dot_string(names).as_bytes()));

        match Command::new("dot")
            .arg("-Tpng")
            .arg("-o".to_string() + png_path.to_str().unwrap())
            .arg(dot_path)
            .output() {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    pub fn save(&self, path: &Path) {
        let mut f = File::create(path).unwrap();
        f.write_all(json::encode(&self).unwrap().as_bytes()).unwrap();
    }
    
    pub fn load(path: &Path) -> Genome {
        let mut f = File::open(path).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        json::decode(&s).unwrap()
    }
}

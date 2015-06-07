use std::collections::BTreeSet;
use std::process::Command;
use std::io::prelude::*;
use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeType {
    Input,
    Output,
    Hidden,
}

pub type NodeId = usize;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub bias: f64,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Link {
    pub from_id: NodeId,
    pub to_id: NodeId,
    pub enabled: bool,
    pub innovation: usize,
    pub weight: f64,
    pub is_recurrent: bool,
}

pub struct Genome {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

pub struct CompatCoefficients {
    pub disjoint: f64,
    pub excess: f64,
    pub weight_diff: f64,
}

impl Genome {
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

    pub fn is_node(&self, id: NodeId) -> bool {
        let count = self.nodes.iter().filter(|node| node.id == id).count();
        assert!(count == 0 || count == 1, "More than one node with the same id");

        return count == 1;
    }

    pub fn get_node(&self, id: NodeId) -> &Node {
        assert!(self.is_node(id));
        let mut matches = self.nodes.iter().filter(|node| node.id == id);
        
        return matches.next().unwrap();
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
        // Starting from `from_id`, try to find `to_id` using depth search.
        // Here, we only use links that are considered feed forward in the network.

        let mut visited = BTreeSet::new();
        let mut queue = vec![from_id];

        while let Some(node_id) = queue.pop() {
            visited.insert(node_id);

            if node_id == to_id {
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

    pub fn to_dot_string(&self) -> String {
        let mut str = String::new();

        str.push_str("digraph g {\n");

        for node in self.nodes.iter() {
            str.push_str(&node.id.to_string());
            str.push_str("\n");
        }

        for link in self.links.iter() {
            str.push_str(&link.from_id.to_string());
            str.push_str(" -> ");
            str.push_str(&link.to_id.to_string());
            str.push_str("\n");
        }

        str.push_str("}\n");

        return str;
    }

    pub fn compile_to_png(&self, dot_path: &Path, png_path: &Path) -> io::Result<()> {
        let mut f = try!(File::create(dot_path));
        try!(f.write_all(self.to_dot_string().as_bytes()));

        match Command::new("dot")
            .arg("-Tpng")
            .arg("-o".to_string() + png_path.to_str().unwrap())
            .arg(dot_path)
            .output() {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }
}

pub enum NodeType {
    Input,
    Output,
    Hidden,
}

pub type NodeId = usize;

pub struct Node {
    pub id: NodeId,
    pub bias: f64,
}

pub struct Link {
    pub from_id: NodeId,
    pub to_id: NodeId,
    pub enabled: bool,
    pub innovation: usize,
    pub weight: f64,
}

pub struct Genome {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

impl Genome {
    pub fn num_nodes(&self) -> usize {
        return self.nodes.len(); //self.nodes.iter().filter(|node| node.enabled).count();
    }

    pub fn num_links(&self) -> usize {
        return self.links.iter().filter(|link| link.enabled).count();
    }
}

extern crate rand;

use std::path::Path;

mod genes; 
mod mutation;
mod nn;

fn main() {
    let mut genome: genes::Genome = genes::Genome {
        nodes: vec![genes::Node { id: 0, node_type: genes::NodeType::Input, bias: 0.0 },
                    genes::Node { id: 1, node_type: genes::NodeType::Input, bias: 0.0 },
                    genes::Node { id: 2, node_type: genes::NodeType::Output, bias: 0.0 },
                    genes::Node { id: 3, node_type: genes::NodeType::Output, bias: 0.0 }],
        links: vec![genes::Link { from_id: 0, to_id: 2, enabled: true, innovation: 0, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 0, to_id: 3, enabled: true, innovation: 0, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 1, to_id: 2, enabled: true, innovation: 0, weight: 0.0, is_recurrent: false },
                    genes::Link { from_id: 1, to_id: 3, enabled: true, innovation: 0, weight: 0.0, is_recurrent: false }]
    };
    
    let mut i = 0;
    let mut rng = rand::thread_rng();
    let mut node_counter = 4;

    loop {
        println!("{}", genome.to_dot_string());
        genome.compile_to_png(Path::new(&format!("test{}.dot", i)),
                              Path::new(&format!("test{}.png", i)));

        i += 1;

        if i > 30 { 
            break;
        }
        
        let mut node_innovations = mutation::NewNodeInnovations::new();
        let mut link_innovations = mutation::NewLinkInnovations::new();
        let mut innovation_counter = 0;


        if i > 15 {
            mutation::new_link(&mut rng, &mut link_innovations, &mut innovation_counter,
                               0.3, 0.5, 30, &mut genome);
        } else {
            mutation::new_node(&mut rng, &mut node_innovations, &mut innovation_counter,
                               &mut node_counter, &mut genome);
        }
    }

    let mut network = nn::Network::from_genome(&genome);
    network.activate(&vec![(0, 1.0), (1, 1.0)]);

    println!("{}","done");

    for (id, value) in network.get_output() {
        println!("{}: {}", id, value);
    }
}

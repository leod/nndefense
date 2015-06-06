extern crate rand;

mod genes; 
mod mutation;

fn main() {
    println!("Hello, world!");
    let genome: genes::Genome = genes::Genome { nodes: vec![], links: vec![] };
}

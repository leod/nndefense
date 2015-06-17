extern crate rand;

use rand::Rng;

use genes;

/// Mate two genomes by aligning their links by innovation number.
/// `parent_a` is assumed to be the better genome.
pub fn multipoint<R: rand::Rng>(rng: &mut R, genome_a: &genes::Genome, genome_b: &genes::Genome) -> genes::Genome {
    // Indices into the link genes of parent_a and parent_b
    let mut i = 0;
    let mut j = 0;

    let mut offspring = genes::Genome { nodes: vec![], links: vec![] };

    // Add all nodes from the better genome so we don't lose any inputs
    offspring.nodes = genome_a.nodes.clone();

    while i < genome_a.links.len() {
        let link_a = &genome_a.links[i];

        // Chose a gene to insert
        let choice =
            if j == genome_b.links.len() {
                i += 1;

                // End of worse genome reached - take excess from better genome
                Some((link_a.clone(), genome_a))
            } else {
                let link_b = &genome_b.links[j];

                // Check for a match in innovation numbers
                if link_a.innovation == link_b.innovation {
                    assert_eq!(link_a.from_id, link_b.from_id);
                    assert_eq!(link_a.to_id, link_b.to_id);

                    i += 1;
                    j += 1;

                    // We have a match, select randomly
                    let (gene, genome) =
                        if rng.gen::<bool>() {
                            (link_a, genome_a)
                        } else {
                            (link_b, genome_b)
                        };

                    // If the link is disabled in a parent, probably disable as well
                    if (!link_a.enabled || !link_b.enabled) &&
                       rng.next_f64() < 0.75 {
                        Some((genes::Link { enabled: false, .. *gene }, genome))
                    } else {
                        Some((*gene, genome))
                    }
                } else if link_a.innovation < link_b.innovation {
                    i += 1;

                    // Take disjoint genes from better genome
                    Some((*link_a, genome_a))
                } else { // link_a.innovation > link_b.innovation
                    j += 1;

                    // Skip disjoint genes from worse genome
                    None
                }
            }; 

        match choice {
            Some((gene, genome)) => {
                if offspring.is_link(gene.from_id, gene.to_id) {
                    continue;
                }

                // Create the link's nodes if they don't exist yet
                if !offspring.is_node(gene.from_id) {
                    offspring.nodes.push(genome.get_node(gene.from_id).unwrap().clone());
                }
                if !offspring.is_node(gene.to_id) {
                    offspring.nodes.push(genome.get_node(gene.to_id).unwrap().clone());
                }

                offspring.add_link(gene);
            },

            None => continue
        }
    }

    offspring
}

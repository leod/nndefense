extern crate rand;

use std::cmp::Ordering;
use rand::Rng;
use genes;
use mutation;
use nn;

#[derive(Clone)]
pub struct Settings {
    pub survival_threshold: f64,
    pub compat_threshold: f64,
}

pub static STANDARD_SETTINGS: Settings = Settings {
    survival_threshold: 0.4,
    compat_threshold: 12.0
};

#[derive(Clone)]
pub struct Organism {
    pub genome: genes::Genome,
    pub network: nn::Network,
    pub fitness: f64,

    adj_fitness: f64,
    expected_offspring: f64,
}

impl Organism {
    pub fn new(genome: &genes::Genome) -> Organism {
        Organism {
            genome: genome.clone(),
            network: nn::Network::from_genome(genome),
            fitness: 0.0,
            adj_fitness: 0.0,
            expected_offspring: 0.0
        }
    }
}

pub struct Species {
    pub organisms: Vec<Organism>,

    expected_offspring: usize,
}

pub struct Population {
    settings: Settings,
    mutation_settings: mutation::Settings,
    compat_coefficients: genes::CompatCoefficients,

    node_counter: usize,
    innovation_counter: usize,
    pub species: Vec<Species>,
}

impl Species {
    pub fn average_adj_fitness(&self) -> f64 {
        self.organisms.iter().map(|organism| organism.adj_fitness).fold(0.0, |x,y| x+y) /
            self.organisms.len() as f64
    }

    pub fn best_organism(&self) -> &Organism {
        return &self.organisms[0];
    }

    /// Calculate organisms' adjusted fitness by dividing by species size (fitness sharing).
    /// Then, the organisms of the species are sorted by their adjusted fitness.
    pub fn prepare_for_epoch(&mut self) {
        let num_organisms = self.organisms.len();

        for organism in self.organisms.iter_mut() {
            assert!(organism.fitness >= 0.0);

            if organism.fitness <= 0.0 {
                organism.fitness = 0.001;
            }

            organism.adj_fitness = organism.fitness / num_organisms as f64;
        }

        self.organisms.sort_by(
            |a, b| b.adj_fitness.partial_cmp(&a.adj_fitness).unwrap_or(Ordering::Equal));
    }

    /// Before reproducing, delete the lowest performing members of the species -
    /// only the fittest can reproduce
    pub fn prune_to_elite(&mut self, survival_threshold: f64) {
        let num_organisms = self.organisms.len() as f64;
        let num_parents = (survival_threshold * num_organisms + 1.0).floor() as usize; // at least keep the champion

        self.organisms.truncate(num_parents);
    }

    /// Only keep the best organism
    pub fn prune_to_champ(&mut self) {
        self.organisms.truncate(1);
    }

    /// Each species is assigned a number of expected offspring based on its share of the total fitness pie.
    /// The parameter `skim` is the fractional part left over from previous species' allotting
    pub fn allot_offspring(&mut self,
                           total_average_adj_fitness: f64,
                           total_population: usize,
                           skim: &mut f64) {
        let expected_offspring = self.average_adj_fitness() / total_average_adj_fitness *
                                 total_population as f64;

        let int_part = expected_offspring.floor() as usize;
        let fract_part = expected_offspring.fract();

        self.expected_offspring = int_part;
        *skim += fract_part;

        if *skim >= 1.0 {
            // Combine the previous fractional part with our fractional part to get more offspring
            self.expected_offspring += skim.floor() as usize;
            *skim -= skim.floor();
        }
    }

    pub fn reproduce<R: rand::Rng>(&self,
                                   mutation_settings: &mutation::Settings,
                                   rng: &mut R,
                                   mutation_state: &mut mutation::State) -> Vec<Organism> {
        assert!(self.expected_offspring > 0);
        assert!(self.organisms.len() > 0, "Empty species cannot reproduce");

        // Create as many organisms as we are allotted
        let mut offspring = Vec::<Organism>::new();

        while offspring.len() < self.expected_offspring - 1 { // Leave room for the champ
            if rng.next_f64() < mutation_settings.mutate_only_prob {
                // Pick one organism and just mutate it and that's the new offspring
                let organism_index = rng.gen_range(0, self.organisms.len());
                let organism = &self.organisms[organism_index];

                let mut new_genome = organism.genome.clone();
                mutation::mutate(&mut new_genome, mutation_settings, rng, mutation_state);

                offspring.push(Organism::new(&new_genome));
            } else {
                // Mate two organisms to create a new one

                continue; // TODO
            }
        }

        return offspring; 
    }
}

impl Population {
    pub fn from_initial_genome<R: rand::Rng>(rng: &mut R,
                                             settings: &Settings,
                                             mutation_settings: &mutation::Settings,
                                             compat_coefficients: &genes::CompatCoefficients,
                                             genome: &genes::Genome,
                                             total_population: usize) -> Population {
        assert!(genome.nodes.len() > 0, "Cannot start with empty genome");

        let mut organisms = Vec::<Organism>::new();

        for i in 0..total_population {
            // Generate completely random weights for each organism
            let mut new_genome = genome.clone();
            mutation::change_link_weights_reset_all(&mut new_genome, rng, 1.0);

            organisms.push(Organism::new(&new_genome));
        }

        // Start with one species containing all organisms
        let species = Species {
            organisms: organisms,
            expected_offspring: 0,
        };

        Population {
            settings: settings.clone(),
            mutation_settings: mutation_settings.clone(),
            compat_coefficients: compat_coefficients.clone(),
            node_counter: genome.nodes.iter().map(|node| node.id).max().unwrap() + 1,
            innovation_counter: 0,
            species: vec![species]
        }
    }

    pub fn num_organisms(&self) -> usize {
        self.species.iter().map(|species| species.organisms.len()).fold(0, |x,y| x+y)
    }

    pub fn average_adj_fitness(&self) -> f64 {
        self.species.iter()
                    .map(|species| species.average_adj_fitness())
                    .fold(0.0, |x,y| x+y)
        / self.species.len() as f64
    }

    pub fn best_organism(&mut self) -> Option<&mut Organism> {
        let mut best = None;
        let mut best_fitness = 0.0;

        for species in self.species.iter_mut() {
            for organism in species.organisms.iter_mut() {
                if organism.fitness > best_fitness  {
                    best_fitness = organism.fitness;
                    best = Some(organism);
                }
            }
        }

        return best;
    }

    /// Insert organisms into the first species they match
    pub fn insert_organism(&mut self, organism: Organism) { 
        assert!(self.species.len() > 0);

        for i in 0..self.species.len() {
            if genes::compatibility(&self.compat_coefficients,
                                    &self.species[i].best_organism().genome,
                                    &organism.genome) < self.settings.compat_threshold {
                self.species[i].organisms.push(organism);
                return;
            }
        }

        // No matching species found - create a new one
        println!("Creating new species");
        self.species.push(Species {
            organisms: vec![organism],
            expected_offspring: 0
        });
    }

    /// Create a new generation of organisms
    pub fn epoch<R: rand::Rng>(&mut self, rng: &mut R) {
        let total_population = self.num_organisms();

        assert!(self.species.len() > 0);
        assert!(total_population > 0);

        for species in self.species.iter_mut() {
            species.prepare_for_epoch();
        }

        let average_adj_fitness = self.average_adj_fitness();
        println!("Average fitness: {}", average_adj_fitness);

        // Allot number of offspring for each species.
        // Here the isue is that if we just round down each species' expected offspring to get whole numbers,
        // we won't necessarily reach `total_population` again. For this reason, we carry around
        // a `skim` that tells us how much fractional part we have left over.
        let mut skim: f64 = 0.0;
        let num_species = self.species.len();
        let mut expected_offspring = 0;
        let total_average_adj_fitness = self.species.iter().map(|species| species.average_adj_fitness()).fold(0.0, |x,y| x+y);
        println!("Total average fitness: {}", total_average_adj_fitness);
        for species in self.species.iter_mut() {
            species.allot_offspring(total_average_adj_fitness, total_population, &mut skim);
            expected_offspring += species.expected_offspring;
            println!("Species with {} organisms, {} average fitness, {} expected offspring",
                     species.organisms.len(), species.average_adj_fitness(), species.expected_offspring);
        }

        // We might still not have reached `total_population`, give the rest to the best species
        assert!(expected_offspring <= total_population);

        {
            // gahhhh
            let mut best_species = None;
            let mut best_fitness = 0.0;

            for i in 0..self.species.len() {
                if self.species[i].best_organism().fitness > best_fitness {
                    best_species = Some(i);
                    best_fitness = self.species[i].best_organism().fitness;
                }
            }

            self.species[best_species.unwrap()].expected_offspring += total_population - expected_offspring;
        }

        // Only allow the elite of each species to reproduce
        self.species.retain(|species| species.expected_offspring > 0);

        for species in self.species.iter_mut() {
            species.prune_to_elite(self.settings.survival_threshold);
        }

        // While reproducing, keep track of the genetic innovations in this generation
        let mut offspring = Vec::<Organism>::new();
        let mut mutation_state = mutation::State {
            node_counter: self.node_counter,
            innovation_counter: self.innovation_counter,
            link_innovations: mutation::NewLinkInnovations::new(),
            node_innovations: mutation::NewNodeInnovations::new()
        };

        for species in self.species.iter() {
            if species.expected_offspring > 0 {
                offspring.extend(species.reproduce(&self.mutation_settings, rng, &mut mutation_state));
            }
        }

        for species in self.species.iter_mut() {
            species.prune_to_champ();
        }

        //assert!(offspring.len() == total_population);

        self.node_counter = mutation_state.node_counter;
        self.innovation_counter = mutation_state.innovation_counter;

        for organism in offspring.iter() {
            self.insert_organism(organism.clone());
        }

        // Delete any species that is now empty
        for species in self.species.iter() {
            if species.organisms.len() == 0 {
                println!("{}", "EMPTY SPECIES!");
            }
        }

        //self.species.retain(|species| species.organisms.len() > 0 && species.expected_offspring > 0);

        assert_eq!(self.species.iter().map(|species| species.organisms.len()).fold(0, |x,y| x+y), total_population);
    }
}

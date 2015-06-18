extern crate rand;

use std::cmp::Ordering;
use rand::Rng;
use genes;
use mutation;
use nn;
use mating;

#[derive(Clone)]
pub struct Settings {
    pub survival_threshold: f64,
    pub compat_threshold: f64,
    pub dropoff_age: usize,
    pub target_num_species: usize,
}

pub static STANDARD_SETTINGS: Settings = Settings {
    survival_threshold: 0.3,
    compat_threshold: 6.0,
    dropoff_age: 15,
    target_num_species: 10
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
        genome.assert_integrity();

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

    id: usize, // For identification by the user

    expected_offspring: usize,

    age: usize,
    highest_fitness: f64, // Over all time
    time_since_last_improvement: usize,

    // Used to compute the compatibility of organisms to this species
    pub best_genome: genes::Genome,

    // Can be changed to give more offspring to the best genome.
    // For all copies after the first one, the weights are mutated.
    best_offspring: usize,
}

pub struct Population {
    settings: Settings,
    mutation_settings: mutation::Settings,
    compat_coefficients: genes::CompatCoefficients,

    species_counter: usize,
    node_counter: usize,
    innovation_counter: usize,

    pub species: Vec<Species>,

    generation: usize,
    highest_fitness: f64, // Over all time
    time_since_last_improvement: usize, // Used to detect stagnation
}

impl Species {
    pub fn new(id: usize, organisms: Vec<Organism>) -> Species {
        let best_genome = organisms[0].genome.clone();
        Species {
            id: id,
            organisms: organisms,
            expected_offspring: 0,
            age: 0,
            time_since_last_improvement: 0,
            highest_fitness: 0.0,
            best_genome: best_genome,
            best_offspring: 0,
        }
    }

    pub fn average_adj_fitness(&self) -> f64 {
        self.organisms.iter().map(|organism| organism.adj_fitness).fold(0.0, |x,y| x+y) /
            self.organisms.len() as f64
    }

    fn best_organism(&self) -> &Organism {
        return &self.organisms[0];
    }

    /// Calculate organisms' adjusted fitness by dividing by species size (fitness sharing).
    /// Then, the organisms of the species are sorted by their adjusted fitness.
    pub fn prepare_for_epoch(&mut self, dropoff_age: usize) {
        let num_organisms = self.organisms.len();
        assert!(num_organisms > 0);

        // Penalty for species that don't improve for a longer time
        let penalize = self.time_since_last_improvement > dropoff_age;

        if penalize {
            println!("Penalizing species {}", self.id);
        }

        for organism in self.organisms.iter_mut() {
            assert!(organism.fitness >= 0.0);

            if organism.fitness <= 0.0 {
                organism.fitness = 0.001;
            }

            // Fitness sharing
            organism.adj_fitness = organism.fitness / num_organisms as f64;

            if penalize {
                organism.adj_fitness *= 0.01;
            }
        }

        self.organisms.sort_by(
            |a, b| b.adj_fitness.partial_cmp(&a.adj_fitness).unwrap_or(Ordering::Equal));

        self.age += 1;

        if self.best_organism().fitness > self.highest_fitness {
            self.time_since_last_improvement = 0;
            self.highest_fitness = self.best_organism().fitness;
        } else {
            self.time_since_last_improvement += 1;
        }

        self.best_genome = self.best_organism().genome.clone();
        self.best_offspring = 1; // By default, give at least 1 offspring to the best genome
    }

    /// Before reproducing, delete the lowest performing members of the species -
    /// only the fittest can reproduce
    pub fn prune_to_elite(&mut self, survival_threshold: f64) {
        let num_organisms = self.organisms.len() as f64;
        let num_parents = (survival_threshold * num_organisms + 1.0).floor() as usize; // at least keep the champion

        self.organisms.truncate(num_parents);
    }

    /// Each species is assigned a number of expected offspring based on its share of the total fitness pie.
    /// The parameter `skim` is the fractional part left over from previous species' allotting
    pub fn allot_offspring(&mut self,
                           average_adj_fitness: f64,
                           skim: &mut f64) {
        let sum_adj_fitness = self.organisms.iter().fold(0.0, |x, o| x + o.adj_fitness);
        let expected_offspring = sum_adj_fitness / average_adj_fitness;

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

        // Give offspring to the best genome
        assert!(self.best_offspring > 0);

        for i in 0..self.best_offspring {
            // Only one unmodified copy
            if i > 0 {
                let mut genome = self.best_genome.clone();
                mutation::change_link_weights_standard(&mut genome, rng, 1.0,
                                                       mutation_settings.change_link_weights_power);
                offspring.push(Organism::new(&genome));
            } else {
                offspring.push(Organism::new(&self.best_genome));
            }
        }

        while offspring.len() < self.expected_offspring { // HACK: Leave room for the champ
            if rng.next_f64() < mutation_settings.mutate_only_prob {
                // Pick one organism and just mutate it and that's the new offspring
                let organism_index = rng.gen_range(0, self.organisms.len());
                let organism = &self.organisms[organism_index];

                let mut new_genome = organism.genome.clone();
                mutation::mutate(&mut new_genome, mutation_settings, rng, mutation_state);

                offspring.push(Organism::new(&new_genome));
            } else {
                // Random parents
                let (parent_a, parent_b) = if rng.next_f64() < mutation_settings.interspecies_mating_prob {
                    // TODO: interspecies mating
                    (&self.organisms[rng.gen_range(0, self.organisms.len())],
                     &self.organisms[rng.gen_range(0, self.organisms.len())])
                } else {
                    (&self.organisms[rng.gen_range(0, self.organisms.len())],
                     &self.organisms[rng.gen_range(0, self.organisms.len())])
                };

                let mut new_genome =
                    if parent_a.fitness >= parent_b.fitness {
                        mating::multipoint(rng, &parent_a.genome, &parent_b.genome)
                    } else {
                        mating::multipoint(rng, &parent_b.genome, &parent_a.genome)
                    };

                // Mutate the offspring's genome according to some probability,
                // or if parent_a is the same genome as parent_b
                if rng.next_f64() < mutation_settings.mutate_after_mating_prob ||
                   genes::compatibility(&genes::STANDARD_COMPAT_COEFFICIENTS,
                                        &parent_a.genome,
                                        &parent_b.genome) == 0.0 {
                    mutation::mutate(&mut new_genome, mutation_settings, rng, mutation_state);     
                }

                offspring.push(Organism::new(&new_genome));
            }
        }

        assert_eq!(offspring.len(), self.expected_offspring);

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

        for _ in 0..total_population {
            // Generate completely random weights for each organism
            let mut new_genome = genome.clone();
            mutation::change_link_weights_reset_all(&mut new_genome, rng, 1.0);

            organisms.push(Organism::new(&new_genome));
        }

        // Start with one species containing all organisms
        let species = Species::new(0, organisms);

        let mut max_innovation = 0;
        for link in genome.links.iter() {
            if link.innovation > max_innovation {
                max_innovation = link.innovation;
            }
        }

        Population {
            settings: settings.clone(),
            mutation_settings: mutation_settings.clone(),
            compat_coefficients: compat_coefficients.clone(),

            node_counter: genome.nodes.iter().map(|node| node.id).max().unwrap() + 1,
            innovation_counter: max_innovation + 1,
            species_counter: 1,

            species: vec![species],

            generation: 0,
            highest_fitness: 0.0,
            time_since_last_improvement: 0,
        }
    }

    pub fn num_organisms(&self) -> usize {
        self.species.iter().map(|species| species.organisms.len()).fold(0, |x,y| x+y)
    }

    pub fn average_adj_fitness(&self) -> f64 {
        /*self.species.iter()
                    .map(|species| species.average_adj_fitness())
                    .fold(0.0, |x,y| x+y)
        / self.species.len() as f64*/

        self.species.iter()
                    .map(|s| s.organisms.iter().map(|o| o.adj_fitness)
                                               .fold(0.0, |x,y| x+y))
                    .fold(0.0, |x,y| x+y)
        / self.num_organisms() as f64
    }

    pub fn best_organism(&self) -> Option<&Organism> {
        let mut best = None;
        let mut best_fitness = 0.0;

        for species in self.species.iter() {
            for organism in species.organisms.iter() {
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
                                    &self.species[i].best_genome,
                                    &organism.genome) < self.settings.compat_threshold {
                self.species[i].organisms.push(organism);
                return;
            }
        }

        // No matching species found - create a new one
        println!("Creating species {}", self.species_counter);

        self.species.push(Species::new(self.species_counter, vec![organism]));
        self.species_counter += 1;
    }

    /// Allot number of offspring for each species.
    fn allot_offspring(&mut self) {
        let total_population = self.num_organisms();

        // If we have population-wide stagnation, give all the offspring to the best two species
        if self.time_since_last_improvement >= self.settings.dropoff_age {
            println!("No improvement for {} epochs, keeping only the first two species",
                     self.time_since_last_improvement);

            self.time_since_last_improvement = 0;

            // Create a sorted list of species indicies, sort by unmodified fitness of best organism
            let mut species_sorted = (0..self.species.len()).collect::<Vec<_>>();
            species_sorted.sort_by(
                |a, b| self.species[*b].best_organism().fitness
                           .partial_cmp(&self.species[*a].best_organism().fitness)
                           .unwrap_or(Ordering::Equal));

            let half_population1 = total_population / 2;
            let half_population2 = total_population - half_population1;

            let best_index = species_sorted[0];

            for i in 0..self.species.len() {
                self.species[i].expected_offspring = 0;
            }

            if self.species.len() > 1 {
                let next_index = species_sorted[1];

                self.species[best_index].expected_offspring = half_population1;
                self.species[next_index].expected_offspring = half_population2;

                self.species[best_index].best_offspring = half_population1;
                self.species[next_index].best_offspring = half_population2;

                self.species[best_index].time_since_last_improvement = 0;
                self.species[next_index].time_since_last_improvement = 0;
            } else {
                self.species[best_index].expected_offspring = half_population1 + half_population2;
                self.species[best_index].best_offspring = half_population1 + half_population2;
                self.species[best_index].time_since_last_improvement = 0;
            }

            return;
        }

        // Otherwise, distribute offspring among species according to the adjusted fitness
        let average_adj_fitness = self.average_adj_fitness();
        println!("Average fitness: {}", average_adj_fitness);

        let total_average_adj_fitness = self.species.iter().map(|species| species.average_adj_fitness())
                                            .fold(0.0, |x,y| x+y);
        println!("Total average fitness: {}", total_average_adj_fitness);

        // Here the isue is that if we just rounded down each species' expected offspring to get whole numbers,
        // we wouldn't necessarily reach `total_population` again. For this reason, we carry around
        // a `skim` that tells us how much fractional part we have left over.
        let mut skim: f64 = 0.0;
        let mut expected_offspring = 0;

        for species in self.species.iter_mut() {
            species.allot_offspring(average_adj_fitness, &mut skim);
            expected_offspring += species.expected_offspring;
        }

        /*for species in self.species.iter() {
            println!("S({}:{}) size: {}, fit: {}, new: {}, nodes: {}, best: {}",
                     species.age, species.time_since_last_improvement,
                     species.organisms.len(), species.average_adj_fitness(), species.expected_offspring,
                     species.organisms.iter().map(|o| o.genome.nodes.len() as f64).fold(0.0, |x,y| x+y) / species.organisms.len() as f64, species.best_organism().fitness);
        }*/

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
    }


    /// Create a new generation of organisms
    pub fn epoch<R: rand::Rng>(&mut self, rng: &mut R) {
        let total_population = self.num_organisms();

        assert!(self.species.len() > 0);
        assert!(total_population > 0);

        // Adjust the threshold by which we consider two organisms to be in the same species.
        // We try to keep the number of species constant (though this does not always seem to work).
        if self.generation > 0 {
            if self.species.len() < self.settings.target_num_species {
                self.settings.compat_threshold -= 0.3;
            }
            if self.species.len() > self.settings.target_num_species {
                self.settings.compat_threshold += 0.3;
            }

            if self.settings.compat_threshold < 0.3 {
                self.settings.compat_threshold = 0.3;
            }
        }

        // Check for stagnation
        {
            let best_fitness = self.best_organism().unwrap().fitness;

            //assert!(self.highest_fitness <= best_fitness);

            if self.highest_fitness >= best_fitness {
                self.time_since_last_improvement += 1;
            } else {
                self.time_since_last_improvement = 0;
                self.highest_fitness = best_fitness;
            }
        }

        for species in self.species.iter_mut() {
            species.prepare_for_epoch(self.settings.dropoff_age);
        }

        self.allot_offspring();

        // Debug species
        {
            let mut species_sorted = (0..self.species.len()).collect::<Vec<_>>();
            species_sorted.sort_by(
                |a, b| self.species[*b].best_organism().fitness
                           .partial_cmp(&self.species[*a].best_organism().fitness)
                           .unwrap_or(Ordering::Equal));

            for i in species_sorted.iter() {
                let s = &self.species[*i];

                println!("S{}: best {}, size {}->{}, adj fit {}, nodes {}",
                         s.id, s.best_organism().fitness, s.organisms.len(), s.expected_offspring,
                         s.average_adj_fitness(), 
                         s.organisms.iter().map(|o| o.genome.nodes.len() as f64).fold(0.0, |x,y| x+y) / s.organisms.len() as f64);
            }

            /*println!("Compatibilities: ");
            for i in species_sorted.iter() {
                for j in species_sorted.iter() {
                    print!("{:.1} ", genes::compatibility(&self.compat_coefficients, &self.species[*i].best_organism().genome, &self.species[*j].best_organism().genome));
                }
                println!("");
            }*/
        }

        println!("Highest fitness: {}, time since last improvement: {}",
                 self.highest_fitness, self.time_since_last_improvement);
        println!("Num species: {}, threshold: {}", self.species.len(), self.settings.compat_threshold);
        
        // Only allow the elite of each species to reproduce
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

        // Reproduce
        for species in self.species.iter() {
            if species.expected_offspring > 0 {
                offspring.extend(species.reproduce(&self.mutation_settings, rng, &mut mutation_state));
            }
        }

        self.node_counter = mutation_state.node_counter;
        self.innovation_counter = mutation_state.innovation_counter;

        for species in self.species.iter_mut() {
            species.organisms.clear(); 
        }

        for organism in offspring.iter() {
            self.insert_organism(organism.clone());
        }

        // Delete any species that is now empty
        for species in self.species.iter() {
            if species.organisms.len() == 0 {
                println!("Species {} empty", species.id);
            }
        }
        self.species.retain(|species| species.organisms.len() > 0);

        // Check that we have the same number of organisms as before this epoch
        assert_eq!(self.species.iter().map(|species| species.organisms.len()).fold(0, |x,y| x+y), total_population);

        self.generation += 1;
    }
}

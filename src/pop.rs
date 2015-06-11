use std::cmp::Ordering;
use genes;
use mutation;
use nn;

pub struct Settings {
    survival_threshold: f64,
}

pub static DEFAULT_SETTINGS: Settings = Settings {
    survival_threshold: 0.2
};

pub struct Organism {
    network: nn::Network,
    fitness: f64,
    adj_fitness: f64,
    expected_offspring: f64,
}

pub struct Species {
    organisms: Vec<Organism>,
    num_parents: usize,
    expected_offspring: usize,
}

pub struct Population {
    settings: Settings,

    node_counter: usize,
    species: Vec<Species>,
}

impl Species {
    pub fn average_adj_fitness(&self) -> f64 {
        self.organisms.iter().map(|organism| organism.adj_fitness).fold(0.0, |x,y| x+y)
    }

    /// Calculate organisms' adjusted fitness by dividing by species size (fitness sharing).
    /// Then, the organisms of the species are sorted by their adjusted fitness.
    pub fn prepare_for_epoch(&mut self, survival_threshold: f64) {
        let num_organisms = self.organisms.len();

        for organism in self.organisms.iter_mut() {
            assert!(organism.fitness >= 0.0);

            organism.adj_fitness = organism.fitness / num_organisms as f64;
        }

        self.organisms.sort_by(
            |a, b| b.adj_fitness.partial_cmp(&a.adj_fitness).unwrap_or(Ordering::Equal));

        // Only the fittest can reproduce - we can delete the rest already
        self.num_parents = (survival_threshold * (num_organisms as f64) + 1.0).floor() as usize; // at least one offspring
        self.organisms.truncate(self.num_parents);
    }

    pub fn allot_offspring(&mut self, total_average_adj_fitness: f64) {
        // Each organism is assigned a number of expected offspring based on its share of the
        // fitness pie
        for organism in self.organisms.iter_mut() {
            organism.expected_offspring = organism.adj_fitness / total_average_adj_fitness;
        }
    }

    pub fn count_offspring(&mut self, skim: &mut f64) {
        self.expected_offspring = 0;

        for organism in self.organisms.iter_mut() {
            //self.expected_offspring +=     
        }
    }
}

impl Population {
    pub fn num_organisms(&self) -> usize {
        self.species.iter().map(|species| species.organisms.len()).fold(0, |x, y| x+y)
    }

    pub fn average_adj_fitness(&self) -> f64 {
        self.species.iter()
                    .map(|species| species.organisms.iter()
                                                    .map(|organism| organism.adj_fitness)
                                                    .fold(0.0, |x,y| x+y))
                    .fold(0.0, |x,y| x+y)
        / self.num_organisms() as f64
    }

    /// Create a new generation of organisms
    pub fn epoch(&mut self) {
        let total_organisms = self.num_organisms();

        for species in self.species.iter_mut() {
            species.prepare_for_epoch(self.settings.survival_threshold);
        }

        let average_adj_fitness = self.average_adj_fitness();
        println!("Average fitness: {}", average_adj_fitness);

        for species in self.species.iter_mut() {
            species.allot_offspring(average_adj_fitness);
        }

        // Here the isue is that if we just round down each species'
        // expected_offspring to get whole numbers, we won't necessarily
        // reach `total_organisms` again. For this reason, we carry around
        // a `skim` that tells us how much fractional part we have left over.

    }
}

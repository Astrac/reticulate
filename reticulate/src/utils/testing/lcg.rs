use rand::{Rng, SeedableRng, rngs::StdRng};

pub struct SeededRandom(StdRng);

impl SeededRandom {
    pub fn new(seed: u64) -> SeededRandom {
        SeededRandom(StdRng::seed_from_u64(seed))
    }

    pub fn next(&mut self, bound: usize) -> usize {
        self.0.next_u64() as usize % bound
    }
}

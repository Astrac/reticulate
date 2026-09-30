/// A small deterministic linear congruential generator, so randomized tests
/// are reproducible without pulling in a dependency.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Lcg {
        Lcg(seed)
    }

    /// A pseudo-random value in `0..bound`.
    pub fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % bound
    }
}

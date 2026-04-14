use crate::math::random::Random;

pub struct Cellular {
    pub(super) random_gen: Random,
}

impl Cellular {
    pub fn new(seed: u64) -> Self {
        Self { random_gen: Random::new(seed) }
    }
}
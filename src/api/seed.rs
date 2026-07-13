use crate::math::random::Random;

/// Generates a psuedo-random seed for a single octave.
pub fn gen_octave_seed<const D: usize>(frequencies: [f32; D], seed: u64) -> u32 {
    match D {
        0..2 => seed as u32,
        2 => Random::mix_u64_pair(
            seed.wrapping_mul(frequencies[0].to_bits() as u64),
            seed.wrapping_mul(frequencies[1].to_bits() as u64),
        ) as u32,
        3 => Random::mix_u64_triple(
            seed.wrapping_mul(frequencies[0].to_bits() as u64),
            seed.wrapping_mul(frequencies[1].to_bits() as u64),
            seed.wrapping_mul(frequencies[2].to_bits() as u64),
        ) as u32,
        4.. => {
            let mut cur_freq = frequencies[0].to_bits() as u64;
            for new_freq in frequencies.iter().skip(1) {
                cur_freq = Random::mix_u64_pair(
                    seed.wrapping_mul(cur_freq),
                    seed.wrapping_mul(new_freq.to_bits() as u64),
                );
            }
            cur_freq as u32
        }
    }
}

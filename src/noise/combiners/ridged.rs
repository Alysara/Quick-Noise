use crate::simd::arch_simd::ArchSimd;
use crate::{Combiner, CombinerArray};

#[derive(Copy, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RidgedConfig {
    pub gain: f32,
}

impl Default for RidgedConfig {
    fn default() -> Self {
        Self { gain: 2.0 }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Ridged {}
impl Combiner for Ridged {
    const WEIGHT_DECAY: bool = false;
    type State = CombinerArray<1>;
    type Config = RidgedConfig;

    #[inline(always)]
    fn apply_sample(
        config: &RidgedConfig,
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        let one = ArchSimd::splat(1.0);
        let gain = ArchSimd::splat(config.gain);
        let zero = ArchSimd::splat(0.0);

        let weight = state[0];

        let signal = one - new_sample.abs();
        let signal = signal * signal * weight;

        let next_weight = (signal * gain).clamp(zero, one);
        let mut next_state = state;
        next_state[0] = next_weight;

        (next_state, cur_result + signal)
    }

    #[inline(always)]
    fn initialize_sample(
        _config: &RidgedConfig,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        let one = ArchSimd::splat(1.0);

        let signal = one - new_sample.abs();
        let signal = signal * signal;

        let mut state = Self::State::default();
        state[0] = signal;

        (state, signal)
    }

    #[inline(always)]
    fn finalize_sample(_config: &RidgedConfig, _state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

use crate::combiners::{Combiner, CombinerArray};
use crate::simd::arch_simd::ArchSimd;

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PingPongConfig {
    pub strength: f32,
}

impl Default for PingPongConfig {
    fn default() -> Self {
        Self { strength: 2.0 }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct PingPong {}
impl Combiner for PingPong {
    const WEIGHT_DECAY: bool = true;
    type State = CombinerArray<0>;
    type Config = PingPongConfig;

    #[inline(always)]
    fn apply_sample(
        config: &PingPongConfig,
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        let one = ArchSimd::splat(1.0);
        let two = ArchSimd::splat(2.0);
        let t = (cur_result + new_sample) * ArchSimd::splat(config.strength);
        let sawtooth = t - (t * ArchSimd::splat(0.5)).floor() * two;
        let folded = one - (sawtooth - one).abs();
        (state, folded)
    }
}

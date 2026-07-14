use crate::combiners::{Combiner, CombinerArray};
use crate::simd::arch_simd::ArchSimd;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TerraceConfig {
    pub steps: f32,
    pub step_size: f32,
}

impl Default for TerraceConfig {
    fn default() -> Self {
        Self {
            steps: 8.0,
            step_size: 1.0 / 8.0,
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Terrace {}
impl Combiner for Terrace {
    const WEIGHT_DECAY: bool = true;
    type State = CombinerArray<0>;
    type Config = TerraceConfig;

    #[inline(always)]
    fn apply_sample(
        _config: &TerraceConfig,
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        (state, cur_result + new_sample)
    }

    #[inline(always)]
    fn initialize_sample(
        _config: &TerraceConfig,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        (Self::State::default(), new_sample)
    }

    #[inline(always)]
    fn finalize_sample(
        config: &TerraceConfig,
        _state: Self::State,
        last: ArchSimd<f32>,
    ) -> ArchSimd<f32> {
        (last * ArchSimd::splat(config.steps)).round() * ArchSimd::splat(config.step_size)
    }
}

use crate::{Combiner, CombinerArray, simd::arch_simd::ArchSimd};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct HybridMultiConfig {
    pub gain: f32,
    pub offset: f32,
}

impl Default for HybridMultiConfig {
    fn default() -> Self {
        Self {
            gain: 2.0,
            offset: 1.0,
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct HybridMulti {}
impl Combiner for HybridMulti {
    const WEIGHT_DECAY: bool = false;
    type State = CombinerArray<1>;
    type Config = HybridMultiConfig;

    #[inline(always)]
    fn apply_sample(
        config: &HybridMultiConfig,
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        let zero = ArchSimd::splat(0.0);
        let one = ArchSimd::splat(1.0);
        let signal = new_sample + ArchSimd::splat(config.offset);
        let weighted_signal = state[0] * signal;

        let result = cur_result + weighted_signal;
        let weight = (weighted_signal * ArchSimd::splat(config.gain)).clamp(zero, one);

        ([weight], result)
    }

    #[inline(always)]
    fn initialize_sample(
        config: &HybridMultiConfig,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        let zero = ArchSimd::splat(0.0);
        let one = ArchSimd::splat(1.0);

        let signal = new_sample + ArchSimd::splat(config.offset);
        let weight = (signal * ArchSimd::splat(config.gain)).clamp(zero, one);

        ([weight], signal)
    }
}

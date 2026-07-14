use crate::combiners::{Combiner, CombinerArray};
use crate::simd::arch_simd::ArchSimd;

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Fbm {}
impl Combiner for Fbm {
    const WEIGHT_DECAY: bool = true;
    type State = CombinerArray<0>;
    type Config = ();

    #[inline(always)]
    fn apply_sample(
        _config: &(),
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        (state, cur_result + new_sample)
    }

    #[inline(always)]
    fn initialize_sample(_config: &(), new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {
        (Self::State::default(), new_sample)
    }

    #[inline(always)]
    fn finalize_sample(_config: &(), _state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

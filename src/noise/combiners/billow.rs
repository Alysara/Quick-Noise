use crate::simd::arch_simd::ArchSimd;
use crate::{Combiner, CombinerArray};

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Billow {}

impl Combiner for Billow {
    const WEIGHT_DECAY: bool = true;
    type State = CombinerArray<0>;
    type Config = ();

    #[inline(always)]
    fn sample(
        _config: &(),
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        (state, cur_result + new_sample.abs())
    }

    #[inline(always)]
    fn initialize(_config: &(), new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {
        (Self::State::default(), new_sample.abs())
    }

    #[inline(always)]
    fn finalize(_config: &(), _state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

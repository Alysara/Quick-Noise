use std::ops::{Index, IndexMut};

use crate::simd::arch_simd::ArchSimd;

pub trait FractalState:
    Copy + Index<usize, Output = ArchSimd<f32>> + IndexMut<usize> + Default
{
    const STATE_SIZE: usize;
}

impl<const N: usize> FractalState for [ArchSimd<f32>; N]
where
    [ArchSimd<f32>; N]: Default,
{
    const STATE_SIZE: usize = N;
}

pub type FractalArray<const N: usize> = [ArchSimd<f32>; N];

pub trait Fractal: Default + Copy + Clone + PartialEq {
    /// Determines whether or not octave weight parameters are ignored.
    /// If this is set to false, every octave has a weight of `1.0`.
    /// If this is set to true, every subsequent octave's weight is multiplied by persistence.
    const WEIGHT_DECAY: bool;

    /// The type used for expressing State. The type `[ArchSimd<f32>; N]` can be used,
    /// where N is the number of variables tracked across samples. N does not include
    /// the running result total. The type alias `FractalArray<N>` can also be used.
    ///
    /// Each additional variable tracked across samples has a signifcant performance
    /// penalty when computing grid noise. The impact is minimal for batch noise.
    type State: FractalState;

    /// The config struct that is passed through noise calls to the Fractal's usage.
    /// This is used for storing new parameters specific to a custom Fractal type.
    type Config: Copy + Default;

    /// Determines how new noise samples are combined with previous samples.
    ///
    /// # Parameters
    /// - `current`: Existing noise value from previous samples
    /// - `output`: New sample output from the current noise pass
    fn sample(
        config: &Self::Config,
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>);

    /// Determines how the first sample is initialized.
    ///
    /// For maximum performance and compiler optimization, it is
    /// recommended to avoid unnecessary instructions such as
    /// adding to 0.0 and multiplying by 1.0.
    ///
    /// # Parameters
    /// - `current`: Existing noise value from previous samples
    /// - `output`: New sample output from the current noise pass
    #[inline(always)]
    fn initialize(
        config: &Self::Config,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        Self::sample(config, Default::default(), Default::default(), new_sample)
    }

    /// Determines how the final noise sample is processed after
    /// being fully combined. This is after `sample` or `sample_first`
    /// has been called.
    ///
    /// # Parameters
    /// - `last`: The final noise sample after prior fractal processing
    #[inline(always)]
    fn finalize(_config: &Self::Config, _state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Fbm {}
impl Fractal for Fbm {
    const WEIGHT_DECAY: bool = true;
    type State = FractalArray<0>;
    type Config = ();

    #[inline(always)]
    fn sample(
        _config: &(),
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        (state, cur_result + new_sample)
    }

    #[inline(always)]
    fn initialize(_config: &(), new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {
        (Self::State::default(), new_sample)
    }

    #[inline(always)]
    fn finalize(_config: &(), _state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Billow {}
impl Fractal for Billow {
    const WEIGHT_DECAY: bool = true;
    type State = FractalArray<0>;
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

#[derive(Default, Copy, Debug, Clone)]
pub struct RidgedConfig {
    pub gain: f32,
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Ridged {}
impl Fractal for Ridged {
    const WEIGHT_DECAY: bool = false; // gain/weight cascade replaces simple persistence decay
    type State = FractalArray<1>; // state[0] = weight carried to next octave
    type Config = RidgedConfig;

    #[inline(always)]
    fn sample(
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
    fn initialize(_config: &RidgedConfig, new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {
        let one = ArchSimd::splat(1.0);

        let signal = one - new_sample.abs();
        let signal = signal * signal;

        let mut state = Self::State::default();
        state[0] = signal; // first weight = first signal, per Musgrave's algorithm

        (state, signal)
    }

    #[inline(always)]
    fn finalize(_config: &RidgedConfig, _state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

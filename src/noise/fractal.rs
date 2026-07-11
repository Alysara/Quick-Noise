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

    /// Determines how new noise samples are combined with previous samples.
    ///
    /// # Parameters
    /// - `current`: Existing noise value from previous samples
    /// - `output`: New sample output from the current noise pass
    fn sample(
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
    fn initialize(new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {
        let state = Self::State::default();
        let prev = ArchSimd::splat(0.0);
        Self::sample(state, prev, new_sample)
    }

    /// Determines how the final noise sample is processed after
    /// being fully combined. This is after `sample` or `sample_first`
    /// has been called.
    ///
    /// # Parameters
    /// - `last`: The final noise sample after prior fractal processing
    fn finalize(_state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Fbm {}
impl Fractal for Fbm {
    const WEIGHT_DECAY: bool = true;
    type State = FractalArray<0>;

    fn sample(
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        (state, cur_result + new_sample)
    }

    fn initialize(new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {
        (Self::State::default(), new_sample)
    }

    fn finalize(_state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Billow {}
impl Fractal for Billow {
    const WEIGHT_DECAY: bool = true;
    type State = FractalArray<0>;

    fn sample(
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        (state, cur_result + new_sample.abs())
    }

    fn initialize(new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {
        (Self::State::default(), new_sample.abs())
    }

    fn finalize(_state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Ridged {}
impl Fractal for Ridged {
    const WEIGHT_DECAY: bool = false; // gain/weight cascade replaces simple persistence decay
    type State = FractalArray<1>; // state[0] = weight carried to next octave

    fn sample(
        state: Self::State,
        cur_result: ArchSimd<f32>,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        let one = ArchSimd::splat(1.0);
        let gain = ArchSimd::splat(2.0);
        let zero = ArchSimd::splat(0.0);

        let weight = state[0];

        // println!("state: {:?}", state);

        let signal = one - new_sample.abs();
        let signal = signal * signal * weight;

        let next_weight = (signal * gain).clamp(zero, one);
        let mut next_state = state;
        next_state[0] = next_weight;

        (next_state, cur_result + signal)
    }

    fn initialize(new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {
        let one = ArchSimd::splat(1.0);

        let signal = one - new_sample.abs();
        let signal = signal * signal;

        let mut state = Self::State::default();
        state[0] = signal; // first weight = first signal, per Musgrave's algorithm

        (state, signal)
    }

    fn finalize(_state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

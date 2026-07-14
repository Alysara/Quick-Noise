use std::ops::{Index, IndexMut};

use crate::simd::arch_simd::ArchSimd;

pub mod billow;
pub mod fbm;
pub mod ridged;

pub use billow::Billow;
pub use fbm::Fbm;
pub use ridged::Ridged;

pub trait CombinerState:
    Copy + Index<usize, Output = ArchSimd<f32>> + IndexMut<usize> + Default
{
    const STATE_SIZE: usize;
}

impl<const N: usize> CombinerState for [ArchSimd<f32>; N]
where
    [ArchSimd<f32>; N]: Default,
{
    const STATE_SIZE: usize = N;
}

pub type CombinerArray<const N: usize> = [ArchSimd<f32>; N];

pub trait Combiner: Default + Copy + Clone + PartialEq {
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
    type State: CombinerState;

    /// The config struct that is passed through noise calls to the Fractal's usage.
    /// This is used for storing new parameters specific to a custom Fractal type.
    type Config: Copy + Default;

    /// Determines how new noise samples are combined with previous samples.
    ///
    /// # Parameters
    /// - `current`: Existing noise value from previous samples
    /// - `output`: New sample output from the current noise pass
    fn apply_sample(
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
    fn initialize_sample(
        config: &Self::Config,
        new_sample: ArchSimd<f32>,
    ) -> (Self::State, ArchSimd<f32>) {
        Self::apply_sample(config, Default::default(), Default::default(), new_sample)
    }

    /// Determines how the final noise sample is processed after
    /// being fully combined. This is after `sample` or `sample_first`
    /// has been called.
    ///
    /// # Parameters
    /// - `last`: The final noise sample after prior fractal processing
    #[inline(always)]
    fn finalize_sample(_config: &Self::Config, _state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {
        last
    }
}

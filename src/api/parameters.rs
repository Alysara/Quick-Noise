// ————————————————————————————————————————————————————————————————
// ————— Builder Macros ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

// Macro functions to keep documentation in one place.

/// Common interface for execucting all noise builders.
///
/// All builders support these three execution methods:
///  - `build()`: Create a new Vec
///  - `into_iter()`: Get a lazy iterator
///  - `fill()`: Reuse existing memory
///  - `fill_onto()`: Add onto existing values
macro_rules! declare_build {
    ($self:ident, $body:tt) => {
        /// Creates the noise and returns the result in an output array.
        ///
        /// Needs to know the length of the output SimdArray because
        /// const generic expr is not yet available in stable Rust when this was
        /// created.
        pub fn build($self) -> Vec<f32> $body
    };
}
pub(crate) use declare_build;

macro_rules! declare_into_iter {
    ($self:ident, $body:tt) => {
        /// Returns an iterator containing chunks of the noise output.
        /// Ideal for managing streams of noise without unnecessary read/writes.
        pub fn into_iter($self) -> impl Iterator<Item = ArchSimd<f32>> $body
    };
}
pub(crate) use declare_into_iter;

macro_rules! declare_fill {
    ($self:ident, $result:ident, $body:tt) => {
        /// Creates the noise and puts the result in a given array.
        pub fn fill($self, $result: &mut [f32]) $body
    };
}
pub(crate) use declare_fill;

macro_rules! declare_fill_onto {
    ($self:ident, $result:ident, $body:tt) => {
        /// Creates the noise and *adds* to the existing values in a given array.
        pub fn fill_onto($self, $result: &mut [f32]) $body
    };
}
pub(crate) use declare_fill_onto;

macro_rules! params_grid_seed_builder {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        impl< $($full_generics)* > $name< $($short_generics)* > {
            /// Determines the psuedo-random values used in noise generation.
            /// Can reproduce the same noise output as grid noise given the
            /// same grid seed + noise seed pair.
            pub fn seed_with_grid(mut self, grid_seed: i64, noise_seed: i64) -> Self {
                let grid_seed = Random::mix_u64(grid_seed as u64);
                let noise_seed = Random::mix_u64(noise_seed as u64 ^ 0xD5E7B3C94F8A1E6B);
                self.noise_config.seed = Random::mix_u64_pair(grid_seed, noise_seed);
                self
            }
        }
    };
}
pub(crate) use params_grid_seed_builder;

macro_rules! params_noise_builder {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        impl< $($full_generics)* > $name< $($short_generics)* > {
            /// Determines the psuedo-random values used in noise generation.
            /// Different seeds produce different noise.
            pub fn seed(mut self, seed: i64) -> Self {
                self.noise_config.seed = Random::mix_u64(seed as u64 ^ 0xD5E7B3C94F8A1E6B);
                self
            }

            /// Controls the range of the noise output. All output is normalized
            /// to be in the range of `[-amplitude, amplitude]`, except for cellular,
            /// which is in the range [0, amplitude].
            ///
            /// # Default
            /// `1.0`
            ///
            /// # Note
            /// As the number of octaves increases, the average noise value trends
            /// closer to zero due to more noise layers averaging eachother out.
            pub fn amplitude(mut self, amplitude: f32) -> Self {
                self.noise_config.amplitude = amplitude;
                self
            }

            /// Controls the magnification of the noise output. For most use cases,
            /// this value can be ignored. Useful for LODs or multi-quality noise
            /// generation.
            ///
            /// # Default
            /// `1.0`
            pub fn magnification(mut self, magnification: f32) -> Self {
                self.noise_config.magnification = magnification;
                self
            }

            /// Controls whether or not normalization is performed. This ensures the noise
            /// output is clamped according to the amplitude. When set to false, output
            /// can be above the specified amplitude. For batched noise, normalization
            /// can be expensive.
            ///
            /// # Default
            /// `true`
            pub fn normalization(mut self, normalization: bool) -> Self {
                self.noise_config.normalization = normalization;
                self
            }

            /// Determines whether or not to overwrite the values in the given slice.
            /// When set to true, the current values are treated as previous octave samples.
            ///
            /// # Default
            /// `true`
            pub fn initialization(mut self, initialization: bool) -> Self {
                self.noise_config.initialization = initialization;
                self
            }

            /// Determines whether or not to finalize the values after the final octave.
            /// This finalization uses what is defined by the [Fractal] type.
            pub fn finalization(mut self, finalization: bool) -> Self {
                self.noise_config.finalization = finalization;
                self
            }

        }
    };
}
pub(crate) use params_noise_builder;

macro_rules! params_lacunarity_builder {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        impl< $($full_generics)* > $name< $($short_generics)* > {
            /// Determines the number of perlin noise passes layered ontop of one another.
            /// More octaves generally leads to more natural-appearing noise.
            ///
            /// # Default
            /// `1`
            pub fn octaves(mut self, octaves: usize) -> Self {
                self.noise_config.octaves = octaves;
                self
            }

            /// Controls how 'compressed' the noise is. Lower frequencies are smoother
            /// and change slower from pixel to pixel, while higher frequencies are sharper and
            /// change more quickly from pixel to pixel.
            ///
            /// # Default
            /// `0.03125` (1.0 / 32.0)
            ///
            /// # Note
            /// Frequencies higher than 0.5 are not properly supported by the uniform grid
            /// algorithm. For accurate noise at super-high frequencies, use perlin_batch().
            pub fn frequency(mut self, frequency: f32) -> Self {
                self.noise_config.frequency = frequency;
                self
            }

            /// Controls how the frequency changes after each subsequenct octave
            /// (noise layer). The next octave's frequency is the previous octave's
            /// frequency multiplied by the lacunarity.
            ///
            /// # Default
            /// `2.0`
            pub fn lacunarity(mut self, lacunarity: f32) -> Self {
                self.noise_config.lacunarity = lacunarity;
                self
            }

            /// Controls how much each subsequenct octave (noise layer) impacts
            /// the final noise result. The next octave's weight is the previous octave's
            /// frequency multiplied by the persistence.
            ///
            /// # Default
            /// `0.5`
            pub fn persistence(mut self, persistence: f32) -> Self {
                self.noise_config.persistence = persistence;
                self
            }
        }
    };
}
pub(crate) use params_lacunarity_builder;

macro_rules! params_noise_scaling_2d {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        /// Controls how much each axis of the grid is 'stretched' in the noise
        /// sample space. Creates visible stretching in the noise output.
        /// The default values have no stretching.
        ///
        /// # Default
        ///  - `1.0`: x_scaling
        ///  - `1.0`: y_scaling
        impl< $($full_generics)* > $name< $($short_generics)* > {
            pub fn scaling(mut self, x_scaling: f32, y_scaling: f32) -> Self {
                self.noise_config.scaling = [x_scaling, y_scaling];
                self
            }
        }
    };
}
pub(crate) use params_noise_scaling_2d;

macro_rules! params_noise_scaling_3d {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        impl< $($full_generics)* > $name< $($short_generics)* > {
            /// Controls how much each axis of the grid is 'stretched' in the noise
            /// sample space. Creates visible stretching in the noise output.
            /// The default values have no stretching.
            ///
            /// # Default
            ///  - `1.0`: x_scaling
            ///  - `1.0`: y_scaling
            ///  - `1.0`: z_scaling
            pub fn scaling(mut self, x_scaling: f32, y_scaling: f32, z_scaling: f32) -> Self {
                self.noise_config.scaling = [x_scaling, y_scaling, z_scaling];
                self
            }
        }
    };
}
pub(crate) use params_noise_scaling_3d;

macro_rules! params_ridged_builder {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        impl< $($full_generics)* > $name< $($short_generics)* > {
            /// Controls how much the previous octave's ridge height is allowed to
            /// contribute.
            ///
            /// # Default
            /// `2.0`
            pub fn gain(mut self, gain: f32) -> Self {
                self.fractal_config.gain = gain;
                self
            }
        }
    };
}
pub(crate) use params_ridged_builder;

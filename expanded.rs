   Compiling quick-noise v0.1.1 (/home/alysara/Documents/Programming/Personal/quick-noise)
error[E0432]: unresolved import `crate::simd::architectures::interface::Static`
 --> src/simd/static_simd.rs:1:5
  |
1 | use crate::simd::architectures::interface::Static;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^------
  |                                            |
  |                                            no `Static` in `simd::architectures::interface`

error[E0432]: unresolved import `crate::simd::static_simd::SIMD_WIDTH`
 --> src/noise/util/grid_helpers.rs:9:44
  |
9 | use crate::simd::static_simd::{StaticSimd, SIMD_WIDTH};
  |                                            ^^^^^^^^^^ no `SIMD_WIDTH` in `simd::static_simd`

error[E0432]: unresolved import `crate::simd::static_simd::ArchFamily`
 --> src/simd/register/iters.rs:5:32
  |
5 | use crate::simd::static_simd::{ArchFamily, StaticSimd};
  |                                ^^^^^^^^^^ no `ArchFamily` in `simd::static_simd`

error[E0405]: cannot find trait `Arch` in this scope
  --> src/simd/static_simd.rs:39:38
   |
39 | pub type ScalarArch = <StaticArch as Arch>::ScalarFamily;
   |                                      ^^^^ not found in this scope
   |
help: consider importing this trait through its public re-export
   |
 1 + use crate::simd::Arch;
   |

warning: unused import: `StaticArch`
  --> src/api/batch/builder.rs:10:31
   |
10 | use crate::simd::{Arch, Simd, StaticArch, StaticSimd};
   |                               ^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `StaticSimd`
 --> src/api/grid/builder.rs:7:31
  |
7 | use crate::simd::{StaticArch, StaticSimd};
  |                               ^^^^^^^^^^

#![feature(prelude_import)]
#![doc =
"Blazingly fast SIMD procedural noise library for batch and uniform grid sampling. Works on stable Rust.\n\n# Performance\n\n### 2D Noise\nTime taken to produce 3 octaves of FBM noise for 1024x1024 (1,048,576) samples.\n| Library              | Perlin  | Value   | Simplex | Cellular |\n|----------------------|---------|---------|---------|----------|\n| quick-noise (grid)   | 0.66 ms | 0.50 ms |    X    |    X     |\n| quick-noise (batch)  | 4.19 ms | 3.79 ms | 5.84 ms | 7.03 ms  |\n| fastnoise2           | 6.22 ms | 5.01 ms | 7.33 ms | 21.4 ms  |\n| simd-noise           | 9.70 ms |    X    |    X    | 14.0 ms  |\n| noise-rs             | 30.1 ms | 29.2 ms | 49.4 ms | 96.3 ms  |\n| noiz                 | 31.4 ms | 26.3 ms | 44.9 ms | 92.6 ms  |\n| libnoise             | 87.8 ms | 27.9 ms | 117 ms  | 176 ms   |\n| noise-functions      | 12.0 ms | 5.77 ms | 44.6 ms | 52.7 ms  |\n\n### 3D Noise\nTime taken to produce 3 octaves of FBM noise for 128x128x128 (2,097,152) samples.\n| Library              | Perlin  | Value   | Simplex | Cellular |\n|----------------------|---------|---------|---------|----------|\n| quick-noise (grid)   | 0.87 ms | 0.62 ms |    X    |    X     |\n| quick-noise (batch)  | 27.2 ms | 12.0 ms | 24.1 ms | 43.4 ms  |\n| fastnoise2           | 29.7 ms | 16.0 ms | 37.9 ms | 137 ms   |\n| simd-noise           | 35.7 ms |    X    |    X    | 96.3 ms  |\n| noise-rs             | 92.0 ms | 212 ms  | 251 ms  | 460 ms   |\n| noiz                 | 127 ms  | 107 ms  | 163 ms  | 489 ms   |\n| libnoise             | 232 ms  | 90.0 ms | 250 ms  | 919 ms   |\n| noise-functions      | 113 ms  | 82.0 ms | 322 ms  | 334 ms   |\n\n* X signifies the noise type is not supported or readily exposed\n* Grid path performance degrades for very high frequencies, and cannot support\nfrequencies >= 1.0. Grid noise. However, it can generate 10+ billion samples per second\nat smaller grid sizes (64x64, 32x32x32) where memory transfer is a smaller barrier.\nMore detailed benchmarks below.\n\n\n# Usage\n\nquick-noise offers two public facing interfaces. The first is grid noise.\nThe performance of grid noise is often magnitudes higher than the second interface,\nbatch noise, and the recommended path for high-performance procedural generation.\nGrid noise samples a squared (2D) or cubed (3D) region uniformly while batch noise \nsamples points at arbitrary inputs.\n\n## Builders\n\nBuilders are used to offer extensive options while remaining approachable.\nEvery builder can be executed with one of three methods: `build()`, `fill()`, and `into_iter()`.\n- `build()`: returns a new Vec of the noise result directly\n- `fill()`: fills a slice that you provide, potentially saving costly memory copies\n- `into_iter()`: returns an iterator containing simd registers of the output\n\nIterators allow multiple steps of the noise pipeline to fuse together, providing speedups by keeping data in registers directly.\nNote that grid noise is an exception to this rule, but makes up for it many times over in speed.\n\n\n## Combiners and Generators\n\nGenerators are structs that define how to generate noise. This includes `Perlin`, `Value`, `Simplex`, and `Cellular`.\nCombiners specify *how* that noise is applied across multiple octaves (noise passes). This includes\n`Fbm`, `Billow`, `Ridged`, `Multi`, `HybridMulti`, `Terrace`, and `PingPong`. Combiners apply to both batch and grid noise.\n\n## Grid Noise\n\nGrid noise is called through a grid region. Each noise call takes into account both the grid seed and the seed of the noise call,\nmaking it easier to have multiple noise maps with the same primary seed.\n\n```rust\nuse quick_noise::{Grid, Fbm, Perlin};\nuse quick_noise::emit::NoiseImageExt;\n\n// Creates an anchor into a region of sample space.\nlet grid = Grid::<2>::new(200, 200) // Specify a 2D 200x200 grid.\n\t.grid_position(0, 0)\n\t.seed(102);\n\t\ngrid.builder::<Fbm, Perlin>()\n\t.octaves(6)\n\t.frequency(0.01)\n\t.into_iter()\n\t.to_grayscale_image(200, 200, \"noise_images/perlin_batch_2d.png\");\n\t\n// FBM Grid noise with all parameters.\nlet noise = grid.builder::<Fbm, Perlin>()\n\t.seed(0)\n\t.octaves(1)\n\t.frequency(0.03125)\n\t.lacunarity(2.0)\n\t.persistence(0.5)\n\t.amplitude(1.0)\n\t.normalization(true)\n\t.scaling(1.0, 1.0)\n    .initialize(true) // Setting to false adds noise to current values.\n    .finalize(true) // Some combiners have a finalization stage.\n\t.build();\n```\n\nCurrently, only Perlin and Value is supported for grid noise. For octave sequences more complicated than FBM noise,\n`builder_with_octaves` can be used for granular control over frequencies and weights.\n\n```rust\nuse quick_noise::{Octave, Grid, Billow, Value};\n\nlet grid = Grid::<2>::new(200, 200);\n\n// Custom list of octaves that can\'t be easily described by FBM noise.\nlet octave_list = [\n\t// Creates octaves from frequency and weight.\n\tOctave::<2>::splat(0.05, 7.0),\n\tOctave::<2>::splat(0.02, 4.0),\n\tOctave::<2>::splat(0.03, 15.0),\n\tOctave::<2>::splat(0.04, 9.0),\n\tOctave::<2>::splat(0.05, 15.0),\n\n\t// Allows axis-specific granularity for frequency.\n\t// This creates \'stretched\' noise.\n\tOctave::<2>::new([0.01, 0.015], 50.0),\n];\n\nlet mut result = vec![0.0; 40000];\nlet noise = grid.builder_with_octaves::<Billow, Value>(octave_list.as_slice())\n\t.seed(1000)\n\t.amplitude(2.0)\n\t.fill(result.as_mut_slice());\n```\n\nquick-noise makes FBM warped noise convenient through a dedicated grid method.\nIt internally adds the values of the grid to the offset iterators you provide it.\nThis can be chained together for complex warp configurations. Since it uses batch noise,\nPerlin, Value, Simplex, and Cellular can all be used here.\n\n```rust\nuse quick_noise::{Grid, Fbm, Perlin};\nuse quick_noise::emit::NoiseImageExt;\n\nlet grid = Grid::<2>::new(1024, 512);\n\n// Create noise offsets to warp by with fast grid noise.\nlet noise1 = grid.builder::<Fbm, Perlin>().octaves(6).seed(0).into_iter();\nlet noise2 = grid.builder::<Fbm, Perlin>().octaves(6).seed(1).into_iter();\n\ngrid.warp_builder::<Fbm, Perlin>(100.0, noise1, noise2)\n    .octaves(2) // Cheap two octaves for expensive batch noise call.\n    .frequency(1. / 32.0)\n    .into_iter()\n    .to_grayscale_image(1024, 512, \"noise_images/perlin_warp_2d.png\");\n\n```\n\n![Warped Perlin Noise](images/perlin_warp_2d.png)\n\nYou can also set custom tiling parameters to the grid to generate noise that\nwraps around and repeats. Unlike other methods, this method does not\nrequire a higher dimension and operates natively in that algorithm.\nHowever, frequencies must align with the given tiling. For example,\na frequency of `1 / 1000` would not work with a tiling of `(2048, 2048)`.\nFrequencies of `1 / 1024` and `1 / 512` would. Tiling is only supported\nfor grid_noise currently.\n\nYou can choose to only enable tiling for specific axes and can specify the\nsize of the tiles for each axis specifically.\n\n```rust\nuse quick_noise::{Grid, Fbm, Perlin};\nuse quick_noise::emit::NoiseImageExt;\n\nlet grid = Grid::<2>::new(1024, 1024)\n\t.grid_position(0, 0)\n\t.seed(100)\n\t.tiling(Some(128), Some(128)); // Put None to disable tiling for that axis.\n\ngrid.builder::<Fbm, Perlin>()\n\t.octaves(6)\n\t.frequency(1.0 / 128.0)\n\t.into_iter()\n\t.to_grayscale_image(1024, 1024, \"noise_images/perlin_tiles.png\");\n```\n\n![Tiled Perlin Noise](images/perlin_tiles_2d.png)\n\n## Batch Noise\n\nBatch noise operates directly on static methods and takes iterators as inputs. Perlin, Value, Simplex, and Cellular all support Batch noise.\n\n```rust\nuse quick_noise::{Grid, BatchNoise, Fbm, Simplex};\nuse quick_noise::emit::NoiseImageExt;\n\n// Use grid for generating iters.\nlet grid = Grid::<2>::new(100, 100).grid_position(0, 0);\n\nlet noise = BatchNoise::<2, Fbm, Simplex>::builder(grid.x_iter(), grid.y_iter())\n\t.octaves(6)\n\t.frequency(0.2)\n\t.lacunarity(0.4)\n\t.persistence(0.6)\n\t.scaling(1.0, 0.5)\n\t.build();\n```\n\nBatch noise allows for arbitrary input coordinates, enabling techniques such as domain warping.\nIn this example, a uniform grid is being generated manually for demonstration purposes. Using grid noise is much faster for this use case.\nSince simplex and cellular do not currently support grid noise, this method can be used to generate\nthem on a grid. Batch noise also supports custom octaves.\n\n## Feature Flags\n\nquick-noise offers a couple of utility features. These are disabled by default to keep compilation lean.\n\n### image\nThe image feature flag uses the `image` crate and enables the usage of `to_grayscale_image` for generating\ngrayscale images of your noise.\n\n### serde\nThe serde feature flag dervies `Serialize` and `Deserialize` for config structs.\n\n## Simd\n\nquick-noise uses a custom simd module purpose-built for noise. Unlike std::simd, it works on stable.\nHowever, only SSE, AVX2, AVX512, and NEON are supported currently. For other systems, a scalar fallback exists,\nbut the performance is much worse. Luckily the vast majority of computers used today support one of these instruction sets.\n\nThis simd module can support most basic operations, and can be used directly to benefit from it:\n\n```rust\nuse quick_noise::{Grid, BatchNoise, Ridged, Cellular};\nuse quick_noise::simd::ArchSimd;\nuse std::iter::zip;\n\nlet grid = Grid::<2>::new(128, 128).grid_position(0, 0);\n\nlet iter_1 = BatchNoise::<2, Ridged, Cellular>::builder(grid.x_iter(), grid.y_iter())\n    .seed(0)\n    .into_iter();\n\nlet iter_2 = BatchNoise::<2, Ridged, Cellular>::builder(grid.x_iter(), grid.y_iter())\n    .seed(1)\n    .into_iter();\n\nlet iter_3 = zip(iter_1, iter_2).map(|(x, y)| x * y);\n```\n\nUsing these iterators can fuse operations and avoid multiple vertical passes, particularly for batch noise.\n`ArchSimd` represents a raw simd register for a given architecture. Unlike std::simd which abstracts these architecture details,\nthis simd module offers you the ability to explicitly control loops that work best for your CPU.\n\n`simd_iter` and `simd_iter_mut` are exposed by the `SimdSliceIterExt` to create these iters from slices.\n\n# Extensibility\n\nquick-noise allows you to implement your own custom combiners and generators.\nThey are defined once in one place and work for both grid and batch noise.\nFor example, the Fbm combiner is defined as:\n\n```rust\n#[derive(Default, Copy, Clone, PartialEq, Debug)]\npub struct Fbm {}\nuse quick_noise::{Combiner, CombinerArray};\nuse quick_noise::simd::ArchSimd;\nimpl Combiner for Fbm {\n    const WEIGHT_DECAY: bool = true;\n\n    // Array of values carried across octaves; unnecessary for Fbm.\n    type State = CombinerArray<0>;\n    type Config = ();\n\n    #[inline(always)]\n    fn apply_sample(\n        _config: &(),\n        state: Self::State,\n        cur_result: ArchSimd<f32>,\n        new_sample: ArchSimd<f32>,\n    ) -> (Self::State, ArchSimd<f32>) {\n        (state, cur_result + new_sample)\n    }\n\n    #[inline(always)]\n    fn initialize_sample(_config: &(), new_sample: ArchSimd<f32>) -> (Self::State, ArchSimd<f32>) {\n        (Self::State::default(), new_sample)\n    }\n\n    #[inline(always)]\n    fn finalize_sample(_config: &(), _state: Self::State, last: ArchSimd<f32>) -> ArchSimd<f32> {\n        last\n    }\n}\n```\n(See Combiner trait documentation for more details)\n\nCustom noise generators can be created by implementing the `GridGenerator` and `BatchGenerator`\ntraits.\n\nTo sample directly, `GridNoise` and `BatchNoise` both have `sample` and `sample_with_octaves`.\nStructs that implement `GridGenerator` and `BatchGenerator` support `sample_grid` and `sample_batch`.\n\n# Detailed Performance\n\n## Grid Noise\n\nGrid noise shares computations across samples to achieve greater performance.\nAs a result, lower frequencies have greater performance than higher frequencies.\nAdditionally, the dimensions of a noise call impact performance as well. Array sizes\nthat are multiples of 16 offer the best SIMD usage. When sizes get very large, memory \ntransfer and cache intermediaries becomes more expensive. For maximum performance, 32x32 and 64x64\nis recommended. However, it is better to use larger calls directly than to transfer\nmemory from smaller calls onto a larger noise map.\n\nResults are measured in billions of points per second single-threaded for one noise pass\nover a 64x64 grid (2D) and 32x32x32 grid (3D).\n- AVX2: I7-13700H | XPS 15 9530 Laptop | Linux\n- AVX512: Ryzen 7 9800X3D | Linux\n\n### Perlin\n| Frequency | 2D AVX2  | 3D AVX2  | 2D AVX512 | 3D AVX512 |\n|-----------|----------|----------|-----------|-----------|\n| 1 / 64    | 13.2 B/s | 11.4 B/s | 35.0 B/s  | 15.9 B/s  |\n| 1 / 48    | 11.6 B/s | 11.4 B/s | 29.4 B/s  | 16.0 B/s  |\n| 1 / 32    | 11.3 B/s | 11.4 B/s | 29.5 B/s  | 16.0 B/s  |\n| 1 / 24    | 10.3 B/s | 9.69 B/s | 24.2 B/s  | 13.4 B/s  |\n| 1 / 16    | 9.52 B/s | 9.58 B/s | 22.1 B/s  | 13.7 B/s  |\n| 1 / 8     | 6.52 B/s | 6.96 B/s | 12.9 B/s  | 9.47 B/s  |\n| 1 / 4     | 3.38 B/s | 2.86 B/s | 5.35 B/s  | 4.37 B/s  |\n\n### Value\n| Frequency | 2D AVX2  | 3D AVX2  | 2D AVX512 | 3D AVX512 |\n|-----------|----------|----------|-----------|-----------|\n| 1 / 64    | 24.3 B/s | 14.3 B/s | 20.8 B/s  | 32.9 B/s  |\n| 1 / 48    | 22.0 B/s | 14.3 B/s | 18.5 B/s  | 33.0 B/s  |\n| 1 / 32    | 22.3 B/s | 14.6 B/s | 18.3 B/s  | 32.8 B/s  |\n| 1 / 24    | 19.7 B/s | 12.9 B/s | 16.2 B/s  | 26.5 B/s  |\n| 1 / 16    | 17.5 B/s | 13.2 B/s | 15.8 B/s  | 26.7 B/s  |\n| 1 / 8     | 12.7 B/s | 11.6 B/s | 14.2 B/s  | 17.5 B/s  |\n| 1 / 4     | 6.68 B/s | 6.56 B/s | 7.76 B/s  | 8.51 B/s  |\n\n## Batch Noise\n\nBatch noise processing is much more flexible than uniform grid, allowing for any arbitrary input and enabling\ntechniques such as domain warping, but at the cost of performance. Results are measured in millions of points per second.\n\n|   Perlin    | 2D AVX2 | 3D AVX2 | 2D AVX512 | 3D AVX512 |\n|-------------|---------|---------|-----------|-----------|\n| quick-noise | 645 M/s | 220 M/s | 1,810 M/s | 871 M/s   |\n| FastNoise2  | 425 M/s | 192 M/s | 942 M/s   | 678 M/s   |\n\n|    Value    | 2D AVX2   | 3D AVX2 | 2D AVX512 | 3D AVX512 |\n|-------------|-----------|---------|-----------|-----------|\n| quick-noise | 707 M/s   | 463 M/s | 2,265 M/s | 1,386 M/s |\n| FastNoise2  | 506 M/s   | 339 M/s | 1,193 M/s | 808 M/s   |\n\n|   Simplex   | 2D AVX2 | 3D AVX2 | 2D AVX512 | 3D AVX512 |\n|-------------|---------|---------|-----------|-----------|\n| quick-noise | 473 M/s | 232 M/s | 1,282 M/s | 816 M/s   |\n| FastNoise2  | 378 M/s | 211 M/s | 910 M/s   | 640 M/s   |\n\n|   Cellular  | 2D AVX2 | 3D AVX2  | 2D AVX512 | 3D AVX512 |\n|-------------|---------|----------|-----------|-----------|\n| quick-noise | 432 M/s | 123 M/s  | 1,196 M/s | 416 M/s   |\n| FastNoise2  | 140 M/s | 44.4 M/s | 397 M/s   | 149 M/s   |\n\n# Running\n\nHeight maps can be generated in `examples/basic.rs`. To run these examples, use:\n\n> cargo run --example basic --release --features=\"image\"\n\nIt is important that `RUSTFLAGS=\'-C target-cpu=native\'` and `--release` is used for the best performance.\n`target-cpu=native` is specified by default in this project, but if you use it in your project and use other flags\nyou may achieve worse performance.\n\nCriterion benches can be run with:\n\n> cargo bench\n\nTest modules can be run with:\n\n> cargo test --features=\"image\" --release\n\nmacOS users may have to comment out the simdnoise dev-dependency due to a Sse4.1 target error.\n"]

//! Maximum performance SIMD-accelerated procedural noise library
//! with up to 10x+ performance on uniform grids. Works on stable Rust.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;

mod api {


    pub mod configs {
        /// Comprehensive config for noise parameters, including lacunarity
        /// octave generation.
        pub struct NoiseConfig<const D : usize> {
            pub seed: u64,
            pub octaves: usize,
            pub frequency: f32,
            pub amplitude: f32,
            pub lacunarity: f32,
            pub persistence: f32,
            pub normalization: bool,
            pub initialize: bool,
            pub finalize: bool,
            pub magnification: f32,
            pub scaling: [f32; D],
        }
        #[automatically_derived]
        impl<const D : usize> ::core::marker::Copy for NoiseConfig<D> { }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl<const D : usize> ::core::clone::TrivialClone for
            NoiseConfig<D> {
        }
        #[automatically_derived]
        impl<const D : usize> ::core::clone::Clone for NoiseConfig<D> {
            #[inline]
            fn clone(&self) -> NoiseConfig<D> {
                let _: ::core::clone::AssertParamIsClone<u64>;
                let _: ::core::clone::AssertParamIsClone<usize>;
                let _: ::core::clone::AssertParamIsClone<f32>;
                let _: ::core::clone::AssertParamIsClone<bool>;
                let _: ::core::clone::AssertParamIsClone<[f32; D]>;
                *self
            }
        }
        #[automatically_derived]
        impl<const D : usize> ::core::fmt::Debug for NoiseConfig<D> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["seed", "octaves", "frequency", "amplitude", "lacunarity",
                                "persistence", "normalization", "initialize", "finalize",
                                "magnification", "scaling"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.seed, &self.octaves, &self.frequency,
                                &self.amplitude, &self.lacunarity, &self.persistence,
                                &self.normalization, &self.initialize, &self.finalize,
                                &self.magnification, &&self.scaling];
                ::core::fmt::Formatter::debug_struct_fields_finish(f,
                    "NoiseConfig", names, values)
            }
        }
        #[automatically_derived]
        impl<const D : usize> ::core::marker::StructuralPartialEq for
            NoiseConfig<D> {
        }
        #[automatically_derived]
        impl<const D : usize> ::core::cmp::PartialEq for NoiseConfig<D> {
            #[inline]
            fn eq(&self, other: &NoiseConfig<D>) -> bool {
                self.seed == other.seed && self.frequency == other.frequency
                                                    && self.amplitude == other.amplitude &&
                                                self.lacunarity == other.lacunarity &&
                                            self.persistence == other.persistence &&
                                        self.normalization == other.normalization &&
                                    self.initialize == other.initialize &&
                                self.finalize == other.finalize &&
                            self.magnification == other.magnification &&
                        self.octaves == other.octaves &&
                    self.scaling == other.scaling
            }
        }
        /// Comprehensive config for noise parameters without lacunarity
        /// octave generation.
        pub struct OctaveNoiseConfig<const D : usize> {
            pub seed: u64,
            pub amplitude: f32,
            pub normalization: bool,
            pub initialize: bool,
            pub finalize: bool,
            pub magnification: f32,
            pub scaling: [f32; D],
        }
        #[automatically_derived]
        impl<const D : usize> ::core::marker::Copy for OctaveNoiseConfig<D> {
        }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl<const D : usize> ::core::clone::TrivialClone for
            OctaveNoiseConfig<D> {
        }
        #[automatically_derived]
        impl<const D : usize> ::core::clone::Clone for OctaveNoiseConfig<D> {
            #[inline]
            fn clone(&self) -> OctaveNoiseConfig<D> {
                let _: ::core::clone::AssertParamIsClone<u64>;
                let _: ::core::clone::AssertParamIsClone<f32>;
                let _: ::core::clone::AssertParamIsClone<bool>;
                let _: ::core::clone::AssertParamIsClone<[f32; D]>;
                *self
            }
        }
        #[automatically_derived]
        impl<const D : usize> ::core::fmt::Debug for OctaveNoiseConfig<D> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                let names: &'static _ =
                    &["seed", "amplitude", "normalization", "initialize",
                                "finalize", "magnification", "scaling"];
                let values: &[&dyn ::core::fmt::Debug] =
                    &[&self.seed, &self.amplitude, &self.normalization,
                                &self.initialize, &self.finalize, &self.magnification,
                                &&self.scaling];
                ::core::fmt::Formatter::debug_struct_fields_finish(f,
                    "OctaveNoiseConfig", names, values)
            }
        }
        #[automatically_derived]
        impl<const D : usize> ::core::marker::StructuralPartialEq for
            OctaveNoiseConfig<D> {
        }
        #[automatically_derived]
        impl<const D : usize> ::core::cmp::PartialEq for OctaveNoiseConfig<D>
            {
            #[inline]
            fn eq(&self, other: &OctaveNoiseConfig<D>) -> bool {
                self.seed == other.seed && self.amplitude == other.amplitude
                                    && self.normalization == other.normalization &&
                                self.initialize == other.initialize &&
                            self.finalize == other.finalize &&
                        self.magnification == other.magnification &&
                    self.scaling == other.scaling
            }
        }
        /// Config specifying parameters of a grid.
        pub struct GridConfig<const D : usize> {
            pub grid_seed: u64,
            pub grid_size: [usize; D],
            pub position: [i32; D],
            pub tiling: [Option<u32>; D],
        }
        #[automatically_derived]
        impl<const D : usize> ::core::marker::Copy for GridConfig<D> { }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl<const D : usize> ::core::clone::TrivialClone for
            GridConfig<D> {
        }
        #[automatically_derived]
        impl<const D : usize> ::core::clone::Clone for GridConfig<D> {
            #[inline]
            fn clone(&self) -> GridConfig<D> {
                let _: ::core::clone::AssertParamIsClone<u64>;
                let _: ::core::clone::AssertParamIsClone<[usize; D]>;
                let _: ::core::clone::AssertParamIsClone<[i32; D]>;
                let _: ::core::clone::AssertParamIsClone<[Option<u32>; D]>;
                *self
            }
        }
        #[automatically_derived]
        impl<const D : usize> ::core::fmt::Debug for GridConfig<D> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(f,
                    "GridConfig", "grid_seed", &self.grid_seed, "grid_size",
                    &self.grid_size, "position", &self.position, "tiling",
                    &&self.tiling)
            }
        }
        #[automatically_derived]
        impl<const D : usize> ::core::marker::StructuralPartialEq for
            GridConfig<D> {
        }
        #[automatically_derived]
        impl<const D : usize> ::core::cmp::PartialEq for GridConfig<D> {
            #[inline]
            fn eq(&self, other: &GridConfig<D>) -> bool {
                self.grid_seed == other.grid_seed &&
                            self.grid_size == other.grid_size &&
                        self.position == other.position &&
                    self.tiling == other.tiling
            }
        }
        impl<const D : usize> NoiseConfig<D> {
            pub(crate) fn num_grid_octaves(&self) -> usize {
                let max_scaling =
                    self.scaling.iter().fold(0.0, |max, x| x.max(max));
                let mut cur_freq = self.frequency * max_scaling;
                if cur_freq >= 1.0 || self.lacunarity >= 1.0 {
                    for i in 0..self.octaves {
                        if cur_freq >= 1.0 { return i; }
                        cur_freq *= self.lacunarity;
                    }
                }
                self.octaves
            }
            pub(crate) fn normalize_amplitude(&self, amplitude: f32) -> f32 {
                let mut sum = 0.0;
                let mut cur = 1.0;
                for _ in 0..self.octaves {
                    sum += cur;
                    cur *= self.persistence;
                }
                amplitude / sum
            }
        }
    }
    pub mod defaults {
        use crate::api::configs::*;
        use crate::simd::static_simd::StaticSimd;
        impl<const D : usize> Default for NoiseConfig<D> {
            fn default() -> Self {
                Self {
                    seed: 0xD5E7B3C94F8A1E6B,
                    octaves: 1,
                    amplitude: 1.0,
                    frequency: 0.03125,
                    lacunarity: 2.0,
                    persistence: 0.5,
                    normalization: true,
                    initialize: true,
                    finalize: true,
                    magnification: 1.0,
                    scaling: [1.0; D],
                }
            }
        }
        impl<const D : usize> Default for OctaveNoiseConfig<D> {
            fn default() -> Self {
                Self {
                    seed: 0xD5E7B3C94F8A1E6B,
                    amplitude: 1.0,
                    normalization: true,
                    initialize: true,
                    finalize: true,
                    magnification: 1.0,
                    scaling: [1.0; D],
                }
            }
        }
        impl<const D : usize> Default for GridConfig<D> {
            fn default() -> Self {
                Self {
                    grid_size: [32; D],
                    grid_seed: 0xc4ceb9fe1a85ec53,
                    position: [0; D],
                    tiling: [None; D],
                }
            }
        }
        /// Empty iterator for default generics.
        pub struct EmptyIter;
        #[automatically_derived]
        impl ::core::default::Default for EmptyIter {
            #[inline]
            fn default() -> EmptyIter { EmptyIter {} }
        }
        impl Iterator for EmptyIter {
            type Item = StaticSimd<f32>;
            fn next(&mut self) -> Option<Self::Item> { None }
        }
        /// Zero Iter for blank noise output.
        pub struct ZeroIter<const N : usize> {
            index: usize,
        }
        #[automatically_derived]
        impl<const N : usize> ::core::default::Default for ZeroIter<N> {
            #[inline]
            fn default() -> ZeroIter<N> {
                ZeroIter { index: ::core::default::Default::default() }
            }
        }
        impl<const N : usize> Iterator for ZeroIter<N> {
            type Item = StaticSimd<f32>;
            fn next(&mut self) -> Option<Self::Item> {
                if self.index < N {
                    self.index += StaticSimd::<f32>::LANES;
                    Some(StaticSimd::zero())
                } else { None }
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                const LANES: usize = StaticSimd::<f32>::LANES;
                let left =
                    (N - self.index + LANES - 1) / StaticSimd::<f32>::LANES;
                (left, Some(left))
            }
        }
    }
    pub mod batch {
        pub mod sample {
            use std::array::from_fn;
            use crate::noise::combiners::Combiner;
            use crate::api::batch::interface::{
                BatchNoise, BatchGenerator, DimIter, DimTuple,
            };
            use crate::api::configs::*;
            use crate::api::seed::gen_octave_seed;
            use crate::simd::{Arch, Simd};
            const MAX_FBM_OCTAVES: usize = 32;
            impl<const D : usize, C: Combiner, G: BatchGenerator<D>>
                BatchNoise<D, C, G> {
                #[inline(always)]
                pub fn sample<A: Arch,
                    I: DimIter<A,
                    D>>(noise_config: NoiseConfig<D>,
                    combiner_config: C::Config, iters: I)
                    -> impl Iterator<Item = Simd<f32, A>> {
                    let octaves = noise_config.octaves;
                    let frequency: [_; D] =
                        from_fn(|i|
                                noise_config.scaling[i] * noise_config.frequency);
                    let weight =
                        if noise_config.normalization && octaves > 0 {
                            noise_config.normalize_amplitude(noise_config.amplitude)
                        } else { noise_config.amplitude };
                    let mut seeds = [0u32; MAX_FBM_OCTAVES];
                    let mut temp_freq = frequency;
                    for seed in seeds.iter_mut().take(octaves) {
                        *seed = gen_octave_seed(temp_freq, noise_config.seed);
                        temp_freq.iter_mut().for_each(|x|
                                *x *= noise_config.lacunarity);
                    }
                    let lacunarity = Simd::splat(noise_config.lacunarity);
                    let persistence = Simd::splat(noise_config.persistence);
                    iters.map(move |x|
                            {
                                let inputs = x.into_array();
                                if octaves == 0 { return Simd::zero(); }
                                let (mut state, mut sample): (C::State<A>, Simd<f32, A>) =
                                    Default::default();
                                let seed = seeds[0];
                                let mut weight = Simd::splat(weight);
                                let mut freq = from_fn(|i| Simd::splat(frequency[i]));
                                let new_sample =
                                    G::sample_batch(seed, inputs, freq) * weight;
                                if noise_config.initialize {
                                    (state, sample) =
                                        C::initialize_sample(&combiner_config, new_sample);
                                } else {
                                    (state, sample) =
                                        C::apply_sample(&combiner_config, state, sample,
                                            new_sample);
                                }
                                for seed in seeds.iter().take(octaves).skip(1) {
                                    freq.iter_mut().for_each(|x| *x *= lacunarity);
                                    if C::WEIGHT_DECAY { weight *= persistence; }
                                    let new_sample =
                                        G::sample_batch(*seed, inputs, freq) * weight;
                                    (state, sample) =
                                        C::apply_sample(&combiner_config, state, sample,
                                            new_sample);
                                }
                                if noise_config.finalize {
                                    C::finalize_sample(&combiner_config, state, sample)
                                } else { sample }
                            })
                }
            }
        }
        pub mod octaves_sample {
            use std::array::from_fn;
            use crate::BatchGenerator;
            use crate::api::batch::interface::{BatchNoise, DimIter, DimTuple};
            use crate::api::configs::*;
            use crate::api::octave::Octave;
            use crate::api::seed::gen_octave_seed;
            use crate::noise::combiners::Combiner;
            use crate::simd::{Arch, Simd};
            const MAX_CUSTOM_OCTAVES: usize = 32;
            fn get_max<const D : usize>(array: [f32; D]) -> f32 {
                array.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
            }
            /// Helper static function for custom noise.
            impl<const D : usize, C: Combiner, G: BatchGenerator<D>>
                BatchNoise<D, C, G> {
                #[inline(always)]
                pub fn sample_with_octaves<A: Arch,
                    I: DimIter<A,
                    D>>(noise_config: OctaveNoiseConfig<D>,
                    combiner_config: C::Config, octave_list: &[Octave<D>],
                    iters: I) -> impl Iterator<Item = Simd<f32, A>> {
                    let octaves = octave_list.len();
                    let total_weight =
                        octave_list.iter().filter(|x|
                                    get_max(x.frequency) <
                                        1.0).fold(0.0, |acc, x| acc + x.weight);
                    let weight_coef =
                        match (noise_config.normalization, total_weight == 0.0) {
                            (false, _) => noise_config.amplitude,
                            (true, false) => noise_config.amplitude / total_weight,
                            (true, true) => 0.0,
                        };
                    let mut seeds = [0u32; MAX_CUSTOM_OCTAVES];
                    for (i, octave) in octave_list.iter().enumerate() {
                        seeds[i] =
                            gen_octave_seed(octave.frequency, noise_config.seed);
                    }
                    let weight_coef = Simd::splat(weight_coef);
                    iters.map(move |x|
                            {
                                let inputs = x.into_array();
                                if octaves == 0 { return Simd::zero(); }
                                let (mut state, mut sample): (C::State<A>, Simd<f32, A>) =
                                    Default::default();
                                let freq =
                                    from_fn(|i| Simd::splat(octave_list[0].frequency[i]));
                                let seed = seeds[0];
                                let weight =
                                    Simd::splat(octave_list[0].weight) * weight_coef;
                                let new_sample =
                                    G::sample_batch(seed, inputs, freq) * weight;
                                if noise_config.initialize {
                                    (state, sample) =
                                        C::initialize_sample(&combiner_config, new_sample);
                                } else {
                                    (state, sample) =
                                        C::apply_sample(&combiner_config, state, sample,
                                            new_sample);
                                }
                                for (i, octave) in
                                    octave_list.iter().enumerate().skip(1).take(octaves.saturating_sub(2))
                                    {
                                    let freq = from_fn(|i| Simd::splat(octave.frequency[i]));
                                    let seed = seeds[i];
                                    let weight =
                                        Simd::splat(octave_list[0].weight) * weight_coef;
                                    let new_sample =
                                        G::sample_batch(seed, inputs, freq) * weight;
                                    (state, sample) =
                                        C::apply_sample(&combiner_config, state, sample,
                                            new_sample);
                                }
                                if noise_config.finalize {
                                    C::finalize_sample(&combiner_config, state, sample)
                                } else { sample }
                            })
                }
            }
        }
        pub mod builder {
            use std::marker::PhantomData;
            use itertools::{Zip, multizip};
            use crate::api::batch::interface::{
                BatchGenerator, BatchNoise, DimIter,
            };
            use crate::api::configs::*;
            use crate::api::parameters::*;
            use crate::math::random::Random;
            use crate::noise::combiners::Combiner;
            use crate::simd::{Arch, Simd, StaticArch, StaticSimd};
            use crate::{HybridMulti, PingPong, Ridged, Terrace};
            pub struct BatchNoiseBuilder<const D : usize, C: Combiner,
                G: BatchGenerator<D>, A: Arch, I: DimIter<A, D>> {
                pub(crate) noise_config: NoiseConfig<D>,
                pub(crate) combiner_config: C::Config,
                pub(crate) iters: I,
                pub(crate) _noise_type: PhantomData<G>,
                pub(crate) _arch: PhantomData<A>,
            }
            impl<const D : usize, C: Combiner, G: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> BatchNoiseBuilder<D, C, G, A, I> {
                /// Determines the psuedo-random values used in noise generation.
                /// Different seeds produce different noise.
                pub fn seed(mut self, seed: i64) -> Self {
                    self.noise_config.seed =
                        Random::mix_u64(seed as u64 ^ 0xD5E7B3C94F8A1E6B);
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
                pub fn initialize(mut self, initialize: bool) -> Self {
                    self.noise_config.initialize = initialize;
                    self
                }
                /// Determines whether or not to finalize the values after the final octave.
                /// This finalization uses what is defined by the [Fractal] type.
                pub fn finalize(mut self, finalize: bool) -> Self {
                    self.noise_config.finalize = finalize;
                    self
                }
            }
            impl<const D : usize, C: Combiner, G: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> BatchNoiseBuilder<D, C, G, A, I> {
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
            impl<const D : usize, C: Combiner<Config : Sized>,
                G: BatchGenerator<D>, A: Arch, I: DimIter<A, D>>
                BatchNoiseBuilder<D, C, G, A, I> {
                /// Configures the config for the combiner
                pub fn combiner_config(mut self, config: C::Config) -> Self {
                    self.combiner_config = config;
                    self
                }
            }
            impl<const D : usize, G: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> BatchNoiseBuilder<D, Ridged, G, A, I> {
                /// Controls how much the previous octave's ridge height is allowed to
                /// contribute.
                ///
                /// # Default
                /// `2.0`
                pub fn gain(mut self, gain: f32) -> Self {
                    self.combiner_config.gain = gain;
                    self
                }
            }
            impl<const D : usize, G: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> BatchNoiseBuilder<D, PingPong, G, A, I> {
                /// Controls how aggressively the noise output is folded.
                ///
                /// # Default
                /// `2.0`
                pub fn strength(mut self, strength: f32) -> Self {
                    self.combiner_config.strength = strength;
                    self
                }
            }
            impl<const D : usize, G: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> BatchNoiseBuilder<D, Terrace, G, A, I> {
                /// Controls how many steps the final noise output is quantized across.
                ///
                /// # Default
                /// `8.0`
                pub fn steps(mut self, steps: f32) -> Self {
                    self.combiner_config.steps = steps;
                    self.combiner_config.step_size = 1.0 / steps;
                    self
                }
            }
            impl<const D : usize, G: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> BatchNoiseBuilder<D, HybridMulti, G, A, I> {
                /// # Default
                /// `2.0`
                pub fn gain(mut self, gain: f32) -> Self {
                    self.combiner_config.gain = gain;
                    self
                }
                /// # Default
                /// `1.0`
                pub fn offset(mut self, offset: f32) -> Self {
                    self.combiner_config.offset = offset;
                    self
                }
            }
            impl<const D : usize, C: Combiner, G: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> BatchNoiseBuilder<D, C, G, A, I> {
                /// Determines the psuedo-random values used in noise generation.
                /// Can reproduce the same noise output as grid noise given the
                /// same grid seed + noise seed pair.
                pub fn seed_with_grid(mut self, grid_seed: i64,
                    noise_seed: i64) -> Self {
                    let grid_seed = Random::mix_u64(grid_seed as u64);
                    let noise_seed =
                        Random::mix_u64(noise_seed as u64 ^ 0xD5E7B3C94F8A1E6B);
                    self.noise_config.seed =
                        Random::mix_u64_pair(grid_seed, noise_seed);
                    self
                }
            }
            /// Controls how much each axis of the grid is 'stretched' in the noise
            /// sample space. Creates visible stretching in the noise output.
            /// The default values have no stretching.
            ///
            /// # Default
            ///  - `1.0`: x_scaling
            ///  - `1.0`: y_scaling
            impl<C: Combiner, G: BatchGenerator<2>, A: Arch, I: DimIter<A, 2>>
                BatchNoiseBuilder<2, C, G, A, I> {
                pub fn scaling(mut self, x_scaling: f32, y_scaling: f32)
                    -> Self {
                    self.noise_config.scaling = [x_scaling, y_scaling];
                    self
                }
            }
            impl<C: Combiner, G: BatchGenerator<3>, A: Arch, I: DimIter<A, 3>>
                BatchNoiseBuilder<3, C, G, A, I> {
                /// Controls how much each axis of the grid is 'stretched' in the noise
                /// sample space. Creates visible stretching in the noise output.
                /// The default values have no stretching.
                ///
                /// # Default
                ///  - `1.0`: x_scaling
                ///  - `1.0`: y_scaling
                ///  - `1.0`: z_scaling
                pub fn scaling(mut self, x_scaling: f32, y_scaling: f32,
                    z_scaling: f32) -> Self {
                    self.noise_config.scaling =
                        [x_scaling, y_scaling, z_scaling];
                    self
                }
            }
            impl<C: Combiner, G: BatchGenerator<2>> BatchNoise<2, C, G> {
                /// Creates a new builder to easily configure batches of noise.
                pub fn builder<A: Arch, X, Y>(x_iter: X, y_iter: Y)
                    -> BatchNoiseBuilder<2, C, G, A, Zip<(X, Y)>> where
                    X: Iterator<Item = Simd<f32, A>>,
                    Y: Iterator<Item = Simd<f32, A>>,
                    Zip<(X, Y)>: DimIter<A, 2> {
                    BatchNoiseBuilder::<2, C, G, A, _>::new(x_iter, y_iter)
                }
            }
            impl<G, C, A, X, Y> BatchNoiseBuilder<2, C, G, A, Zip<(X, Y)>>
                where G: BatchGenerator<2>, C: Combiner, A: Arch,
                X: Iterator<Item = Simd<f32, A>>,
                Y: Iterator<Item = Simd<f32, A>>, Zip<(X, Y)>: DimIter<A, 2> {
                pub fn new(x_iter: X, y_iter: Y) -> Self {
                    Self {
                        noise_config: Default::default(),
                        combiner_config: Default::default(),
                        iters: multizip((x_iter, y_iter)),
                        _noise_type: PhantomData::<G>,
                        _arch: PhantomData::<A>,
                    }
                }
                pub fn from_configs(noise_config: NoiseConfig<2>,
                    combiner_config: C::Config, x_iter: X, y_iter: Y) -> Self {
                    Self {
                        noise_config,
                        combiner_config,
                        iters: multizip((x_iter, y_iter)),
                        _noise_type: PhantomData::<G>,
                        _arch: PhantomData::<A>,
                    }
                }
            }
            impl<F: Combiner, S: BatchGenerator<3>> BatchNoise<3, F, S> {
                /// Creates a new builder to easily configure batches of noise.
                pub fn builder<A: Arch, X, Y,
                    Z>(x_iter: X, y_iter: Y, z_iter: Z)
                    -> BatchNoiseBuilder<3, F, S, A, Zip<(X, Y, Z)>> where
                    X: Iterator<Item = Simd<f32, A>>,
                    Y: Iterator<Item = Simd<f32, A>>,
                    Z: Iterator<Item = Simd<f32, A>>,
                    Zip<(X, Y, Z)>: DimIter<A, 3> {
                    BatchNoiseBuilder::<3, F, S, A,
                            _>::new(x_iter, y_iter, z_iter)
                }
            }
            impl<S, F, A, X, Y, Z>
                BatchNoiseBuilder<3, F, S, A, Zip<(X, Y, Z)>> where
                S: BatchGenerator<3>, F: Combiner, A: Arch,
                X: Iterator<Item = Simd<f32, A>>,
                Y: Iterator<Item = Simd<f32, A>>,
                Z: Iterator<Item = Simd<f32, A>>,
                Zip<(X, Y, Z)>: DimIter<A, 3> {
                pub fn new(x_iter: X, y_iter: Y, z_iter: Z) -> Self {
                    Self {
                        noise_config: Default::default(),
                        combiner_config: Default::default(),
                        iters: multizip((x_iter, y_iter, z_iter)),
                        _noise_type: PhantomData::<S>,
                        _arch: PhantomData::<A>,
                    }
                }
                pub fn from_configs(noise_config: NoiseConfig<3>,
                    combiner_config: F::Config, x_iter: X, y_iter: Y, z_iter: Z)
                    -> Self {
                    Self {
                        noise_config,
                        combiner_config,
                        iters: multizip((x_iter, y_iter, z_iter)),
                        _noise_type: PhantomData::<S>,
                        _arch: PhantomData::<A>,
                    }
                }
            }
            impl<const D : usize, F: Combiner, S: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> BatchNoiseBuilder<D, F, S, A, I> {
                /// Creates the noise and puts the result in a given array.
                pub fn fill(self, output: &mut [f32]) {
                    if self.noise_config.initialize {
                        for (i, x) in self.into_iter().enumerate() {
                            x.copy_to_slice(&mut output[i *
                                                StaticSimd::<f32>::LANES..]);
                        }
                    } else {
                        for (i, x) in self.into_iter().enumerate() {
                            let index = i * Simd::<f32, A>::LANES;
                            let cur = Simd::from_slice(&output[index..]);
                            let x = cur + x;
                            x.copy_to_slice(&mut output[i..]);
                        }
                    }
                }
                /// Creates the noise and returns the result in an output array.
                ///
                /// Needs to know the length of the output SimdArray because
                /// const generic expr is not yet available in stable Rust when this was
                /// created.
                pub fn build(self) -> Vec<f32> { self.into_iter().collect() }
                /// Returns an iterator containing chunks of the noise output.
                /// Ideal for managing streams of noise without unnecessary read/writes.
                #[allow(clippy :: should_implement_trait)]
                pub fn into_iter(self)
                    -> impl Iterator<Item = crate::simd::Simd<f32, A>> {
                    BatchNoise::<D, F,
                            S>::sample(self.noise_config, self.combiner_config,
                        self.iters)
                }
            }
        }
        pub mod octaves_builder {
            use std::marker::PhantomData;
            use itertools::{Zip, multizip};
            use crate::api::batch::interface::{BatchNoise, DimIter};
            use crate::api::configs::*;
            use crate::api::octave::Octave;
            use crate::api::parameters::*;
            use crate::math::random::Random;
            use crate::noise::combiners::Combiner;
            use crate::simd::static_simd::StaticSimd;
            use crate::simd::{Arch, Simd};
            use crate::{
                BatchGenerator, HybridMulti, PingPong, Ridged, Terrace,
            };
            pub struct OctaveBatchNoiseBuilder<'a, const D : usize,
                C: Combiner, G: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>> {
                noise_config: OctaveNoiseConfig<D>,
                combiner_config: C::Config,
                octave_list: &'a [Octave<D>],
                iters: I,
                _noise_type: PhantomData<G>,
                _arch: PhantomData<A>,
            }
            impl<'a, const D : usize, C: Combiner, G: BatchGenerator<D>,
                A: Arch, I: DimIter<A, D>>
                OctaveBatchNoiseBuilder<'a, D, C, G, A, I> {
                /// Determines the psuedo-random values used in noise generation.
                /// Different seeds produce different noise.
                pub fn seed(mut self, seed: i64) -> Self {
                    self.noise_config.seed =
                        Random::mix_u64(seed as u64 ^ 0xD5E7B3C94F8A1E6B);
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
                pub fn initialize(mut self, initialize: bool) -> Self {
                    self.noise_config.initialize = initialize;
                    self
                }
                /// Determines whether or not to finalize the values after the final octave.
                /// This finalization uses what is defined by the [Fractal] type.
                pub fn finalize(mut self, finalize: bool) -> Self {
                    self.noise_config.finalize = finalize;
                    self
                }
            }
            impl<'a, const D : usize, C: Combiner, G: BatchGenerator<D>,
                A: Arch, I: DimIter<A, D>>
                OctaveBatchNoiseBuilder<'a, D, C, G, A, I> {
                /// Determines the psuedo-random values used in noise generation.
                /// Can reproduce the same noise output as grid noise given the
                /// same grid seed + noise seed pair.
                pub fn seed_with_grid(mut self, grid_seed: i64,
                    noise_seed: i64) -> Self {
                    let grid_seed = Random::mix_u64(grid_seed as u64);
                    let noise_seed =
                        Random::mix_u64(noise_seed as u64 ^ 0xD5E7B3C94F8A1E6B);
                    self.noise_config.seed =
                        Random::mix_u64_pair(grid_seed, noise_seed);
                    self
                }
            }
            impl<'a, const D : usize, T: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>>
                OctaveBatchNoiseBuilder<'a, D, Ridged, T, A, I> {
                /// Controls how much the previous octave's ridge height is allowed to
                /// contribute.
                ///
                /// # Default
                /// `2.0`
                pub fn gain(mut self, gain: f32) -> Self {
                    self.combiner_config.gain = gain;
                    self
                }
            }
            impl<'a, const D : usize, C: Combiner<Config : Sized>,
                G: BatchGenerator<D>, A: Arch, I: DimIter<A, D>>
                OctaveBatchNoiseBuilder<'a, D, C, G, A, I> {
                /// Configures the config for the combiner
                pub fn combiner_config(mut self, config: C::Config) -> Self {
                    self.combiner_config = config;
                    self
                }
            }
            impl<'a, const D : usize, T: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>>
                OctaveBatchNoiseBuilder<'a, D, PingPong, T, A, I> {
                /// Controls how aggressively the noise output is folded.
                ///
                /// # Default
                /// `2.0`
                pub fn strength(mut self, strength: f32) -> Self {
                    self.combiner_config.strength = strength;
                    self
                }
            }
            impl<'a, const D : usize, T: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>>
                OctaveBatchNoiseBuilder<'a, D, Terrace, T, A, I> {
                /// Controls how many steps the final noise output is quantized across.
                ///
                /// # Default
                /// `8.0`
                pub fn steps(mut self, steps: f32) -> Self {
                    self.combiner_config.steps = steps;
                    self.combiner_config.step_size = 1.0 / steps;
                    self
                }
            }
            impl<'a, const D : usize, T: BatchGenerator<D>, A: Arch,
                I: DimIter<A, D>>
                OctaveBatchNoiseBuilder<'a, D, HybridMulti, T, A, I> {
                /// # Default
                /// `2.0`
                pub fn gain(mut self, gain: f32) -> Self {
                    self.combiner_config.gain = gain;
                    self
                }
                /// # Default
                /// `1.0`
                pub fn offset(mut self, offset: f32) -> Self {
                    self.combiner_config.offset = offset;
                    self
                }
            }
            /// Controls how much each axis of the grid is 'stretched' in the noise
            /// sample space. Creates visible stretching in the noise output.
            /// The default values have no stretching.
            ///
            /// # Default
            ///  - `1.0`: x_scaling
            ///  - `1.0`: y_scaling
            impl<'a, C: Combiner, G: BatchGenerator<2>, A: Arch,
                I: DimIter<A, 2>> OctaveBatchNoiseBuilder<'a, 2, C, G, A, I> {
                pub fn scaling(mut self, x_scaling: f32, y_scaling: f32)
                    -> Self {
                    self.noise_config.scaling = [x_scaling, y_scaling];
                    self
                }
            }
            impl<'a, C: Combiner, G: BatchGenerator<3>, A: Arch,
                I: DimIter<A, 3>> OctaveBatchNoiseBuilder<'a, 3, C, G, A, I> {
                /// Controls how much each axis of the grid is 'stretched' in the noise
                /// sample space. Creates visible stretching in the noise output.
                /// The default values have no stretching.
                ///
                /// # Default
                ///  - `1.0`: x_scaling
                ///  - `1.0`: y_scaling
                ///  - `1.0`: z_scaling
                pub fn scaling(mut self, x_scaling: f32, y_scaling: f32,
                    z_scaling: f32) -> Self {
                    self.noise_config.scaling =
                        [x_scaling, y_scaling, z_scaling];
                    self
                }
            }
            impl<F: Combiner, S: BatchGenerator<2>> BatchNoise<2, F, S> {
                /// Creates a new builder using a custom octave list to configure
                /// batches of noise.
                pub fn builder_with_octaves<'a, A, X,
                    Y>(octave_list: &'a [Octave<2>], x_iter: X, y_iter: Y)
                    -> OctaveBatchNoiseBuilder<'a, 2, F, S, A, Zip<(X, Y)>>
                    where A: Arch, X: Iterator<Item = Simd<f32, A>>,
                    Y: Iterator<Item = Simd<f32, A>>,
                    Zip<(X, Y)>: DimIter<A, 2> {
                    OctaveBatchNoiseBuilder::<'a, 2, F, S, A,
                            _>::new(octave_list, x_iter, y_iter)
                }
            }
            impl<'a, S, F, A, X, Y>
                OctaveBatchNoiseBuilder<'a, 2, F, S, A, Zip<(X, Y)>> where
                S: BatchGenerator<2>, F: Combiner, A: Arch,
                X: Iterator<Item = Simd<f32, A>>,
                Y: Iterator<Item = Simd<f32, A>>, Zip<(X, Y)>: DimIter<A, 2> {
                pub fn new(octave_list: &'a [Octave<2>], x_iter: X, y_iter: Y)
                    -> Self {
                    Self {
                        noise_config: Default::default(),
                        combiner_config: Default::default(),
                        octave_list,
                        iters: multizip((x_iter, y_iter)),
                        _noise_type: PhantomData::<S>,
                        _arch: PhantomData::<A>,
                    }
                }
                pub fn from_configs(noise_config: OctaveNoiseConfig<2>,
                    combiner_config: F::Config, octave_list: &'a [Octave<2>],
                    x_iter: X, y_iter: Y) -> Self {
                    Self {
                        noise_config,
                        combiner_config,
                        octave_list,
                        iters: multizip((x_iter, y_iter)),
                        _noise_type: PhantomData::<S>,
                        _arch: PhantomData::<A>,
                    }
                }
            }
            impl<F: Combiner, S: BatchGenerator<3>> BatchNoise<3, F, S> {
                /// Creates a new builder using a custom octave list to configure
                /// batches of noise.
                pub fn builder_with_octaves<'a, A, X, Y,
                    Z>(octave_list: &'a [Octave<3>], x_iter: X, y_iter: Y,
                    z_iter: Z)
                    -> OctaveBatchNoiseBuilder<'a, 3, F, S, A, Zip<(X, Y, Z)>>
                    where A: Arch, X: Iterator<Item = Simd<f32, A>>,
                    Y: Iterator<Item = Simd<f32, A>>,
                    Z: Iterator<Item = Simd<f32, A>>,
                    Zip<(X, Y, Z)>: DimIter<A, 3> {
                    OctaveBatchNoiseBuilder::<3, F, S, A,
                            _>::new(octave_list, x_iter, y_iter, z_iter)
                }
            }
            impl<'a, S, F, A, X, Y, Z>
                OctaveBatchNoiseBuilder<'a, 3, F, S, A, Zip<(X, Y, Z)>> where
                S: BatchGenerator<3>, F: Combiner, A: Arch,
                X: Iterator<Item = Simd<f32, A>>,
                Y: Iterator<Item = Simd<f32, A>>,
                Z: Iterator<Item = Simd<f32, A>>,
                Zip<(X, Y, Z)>: DimIter<A, 3> {
                pub fn new(octave_list: &'a [Octave<3>], x_iter: X, y_iter: Y,
                    z_iter: Z) -> Self {
                    Self {
                        noise_config: Default::default(),
                        combiner_config: Default::default(),
                        octave_list,
                        iters: multizip((x_iter, y_iter, z_iter)),
                        _noise_type: PhantomData::<S>,
                        _arch: PhantomData::<A>,
                    }
                }
                pub fn from_configs(noise_config: OctaveNoiseConfig<3>,
                    combiner_config: F::Config, octave_list: &'a [Octave<3>],
                    x_iter: X, y_iter: Y, z_iter: Z) -> Self {
                    Self {
                        noise_config,
                        combiner_config,
                        octave_list,
                        iters: multizip((x_iter, y_iter, z_iter)),
                        _noise_type: PhantomData::<S>,
                        _arch: PhantomData::<A>,
                    }
                }
            }
            impl<'a, const D : usize, C: Combiner, G: BatchGenerator<D>,
                A: Arch, I: DimIter<A, D>>
                OctaveBatchNoiseBuilder<'a, D, C, G, A, I> {
                /// Creates the noise and puts the result in a given array.
                pub fn fill(self, output: &mut [f32]) {
                    if self.noise_config.initialize {
                        for (i, x) in self.into_iter().enumerate() {
                            x.copy_to_slice(&mut output[i *
                                                StaticSimd::<f32>::LANES..]);
                        }
                    } else {
                        for (i, x) in self.into_iter().enumerate() {
                            let index = i * Simd::<f32, A>::LANES;
                            let cur = Simd::from_slice(&output[index..]);
                            let x = cur + x;
                            x.copy_to_slice(&mut output[i..]);
                        }
                    }
                }
                /// Creates the noise and returns the result in an output array.
                ///
                /// Needs to know the length of the output SimdArray because
                /// const generic expr is not yet available in stable Rust when this was
                /// created.
                pub fn build(self) -> Vec<f32> { self.into_iter().collect() }
                /// Returns an iterator containing chunks of the noise output.
                /// Ideal for managing streams of noise without unnecessary read/writes.
                #[allow(clippy :: should_implement_trait)]
                pub fn into_iter(self)
                    -> impl Iterator<Item = crate::simd::Simd<f32, A>> {
                    BatchNoise::<D, C,
                            G>::sample_with_octaves(self.noise_config,
                        self.combiner_config, self.octave_list, self.iters)
                }
            }
        }
        pub mod interface {
            use std::marker::PhantomData;
            use crate::noise::combiners::Combiner;
            use crate::simd::{Arch, Simd};
            /// Zipped iterator type used for generically handling
            /// `D` iterators.
            pub trait DimTuple<A: Arch, const D : usize> {
                /// Converts the internal iterator representation
                /// into a generically sized array.
                fn into_array(self)
                -> [Simd<f32, A>; D];
            }
            type S<A: Arch> = Simd<f32, A>;
            impl<A: Arch> DimTuple<A, 1> for S<A> {
                fn into_array(self) -> [Simd<f32, A>; 1] { [self] }
            }
            impl<A: Arch> DimTuple<A, 2> for (S<A>, S<A>) {
                fn into_array(self) -> [Simd<f32, A>; 2] { [self.0, self.1] }
            }
            impl<A: Arch> DimTuple<A, 3> for (S<A>, S<A>, S<A>) {
                fn into_array(self) -> [Simd<f32, A>; 3] {
                    [self.0, self.1, self.2]
                }
            }
            impl<A: Arch> DimTuple<A, 4> for (S<A>, S<A>, S<A>, S<A>) {
                fn into_array(self) -> [Simd<f32, A>; 4] {
                    [self.0, self.1, self.2, self.3]
                }
            }
            impl<A: Arch> DimTuple<A, 5> for (S<A>, S<A>, S<A>, S<A>, S<A>) {
                fn into_array(self) -> [Simd<f32, A>; 5] {
                    [self.0, self.1, self.2, self.3, self.4]
                }
            }
            pub trait DimIter<A: Arch, const D :
                usize>: Iterator<Item : DimTuple<A, D>> {}
            impl<A: Arch, const D : usize, T: DimTuple<A, D>,
                I: Iterator<Item = T>> DimIter<A, D> for I {}
            pub trait BatchGenerator<const D : usize> {
                /// Generates noise using simd registers.
                ///
                /// # Parameters
                /// - `seed`: Configures the deterministic randomness of the noise
                /// - `input`: Array of input values for each dimension
                /// - `freq`: Array of frequency values for each dimension
                fn sample_batch<A: Arch>(seed: u32, input: [Simd<f32, A>; D],
                freq: [Simd<f32, A>; D])
                -> Simd<f32, A>;
            }
            ///  struct for sampling batch noise.
            ///
            /// # Example
            /// ```
            /// use quick_noise::{Grid, BatchNoise, Fbm, Perlin};
            ///
            /// let grid = Grid::<2>::new(32, 32);
            ///
            /// let noise = BatchNoise::<2, Fbm, Perlin>::builder(grid.x_iter(), grid.y_iter())
            ///     .octaves(1)
            ///     .frequency(1.0 / 32.0)
            ///     .build();
            /// ```
            pub struct BatchNoise<const D : usize, F: Combiner,
                S: BatchGenerator<D>> {
                _fractal: PhantomData<F>,
                _sampler: PhantomData<S>,
            }
            #[automatically_derived]
            impl<const D : usize, F: ::core::default::Default + Combiner,
                S: ::core::default::Default + BatchGenerator<D>>
                ::core::default::Default for BatchNoise<D, F, S> {
                #[inline]
                fn default() -> BatchNoise<D, F, S> {
                    BatchNoise {
                        _fractal: ::core::default::Default::default(),
                        _sampler: ::core::default::Default::default(),
                    }
                }
            }
        }
        pub use builder::BatchNoiseBuilder;
        pub use octaves_builder::OctaveBatchNoiseBuilder;
    }
    pub mod grid {
        pub mod builder {
            use std::marker::PhantomData;
            use crate::api::configs::*;
            use crate::api::grid::interface::{GridGenerator, GridNoise};
            use crate::api::parameters::*;
            use crate::math::random::Random;
            use crate::simd::{StaticArch, StaticSimd};
            use crate::simd::register::iters::IntoSimdIterator;
            use crate::{Combiner, HybridMulti, PingPong, Ridged, Terrace};
            /// A struct for creating FBM noise set on a uniform grid.
            /// The most performant way to generate Perlin noise.
            pub struct GridNoiseBuilder<const D : usize, C: Combiner,
                G: GridGenerator<D>> {
                grid_config: GridConfig<D>,
                noise_config: NoiseConfig<D>,
                combiner_config: C::Config,
                _noise_type: PhantomData<G>,
            }
            #[automatically_derived]
            impl<const D : usize, C: ::core::default::Default + Combiner,
                G: ::core::default::Default + GridGenerator<D>>
                ::core::default::Default for GridNoiseBuilder<D, C, G> where
                C::Config: ::core::default::Default {
                #[inline]
                fn default() -> GridNoiseBuilder<D, C, G> {
                    GridNoiseBuilder {
                        grid_config: ::core::default::Default::default(),
                        noise_config: ::core::default::Default::default(),
                        combiner_config: ::core::default::Default::default(),
                        _noise_type: ::core::default::Default::default(),
                    }
                }
            }
            #[automatically_derived]
            impl<const D : usize, C: ::core::marker::Copy + Combiner,
                G: ::core::marker::Copy + GridGenerator<D>>
                ::core::marker::Copy for GridNoiseBuilder<D, C, G> where
                C::Config: ::core::marker::Copy {
            }
            #[automatically_derived]
            impl<const D : usize, C: ::core::clone::Clone + Combiner,
                G: ::core::clone::Clone + GridGenerator<D>>
                ::core::clone::Clone for GridNoiseBuilder<D, C, G> where
                C::Config: ::core::clone::Clone {
                #[inline]
                fn clone(&self) -> GridNoiseBuilder<D, C, G> {
                    GridNoiseBuilder {
                        grid_config: ::core::clone::Clone::clone(&self.grid_config),
                        noise_config: ::core::clone::Clone::clone(&self.noise_config),
                        combiner_config: ::core::clone::Clone::clone(&self.combiner_config),
                        _noise_type: ::core::clone::Clone::clone(&self._noise_type),
                    }
                }
            }
            impl<const D : usize, C: Combiner, G: GridGenerator<D>>
                GridNoiseBuilder<D, C, G> {
                /// Determines the psuedo-random values used in noise generation.
                /// Different seeds produce different noise.
                pub fn seed(mut self, seed: i64) -> Self {
                    self.noise_config.seed =
                        Random::mix_u64(seed as u64 ^ 0xD5E7B3C94F8A1E6B);
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
                pub fn initialize(mut self, initialize: bool) -> Self {
                    self.noise_config.initialize = initialize;
                    self
                }
                /// Determines whether or not to finalize the values after the final octave.
                /// This finalization uses what is defined by the [Fractal] type.
                pub fn finalize(mut self, finalize: bool) -> Self {
                    self.noise_config.finalize = finalize;
                    self
                }
            }
            impl<const D : usize, C: Combiner, G: GridGenerator<D>>
                GridNoiseBuilder<D, C, G> {
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
            impl<const D : usize, C: Combiner<Config : Sized>,
                G: GridGenerator<D>> GridNoiseBuilder<D, C, G> {
                /// Configures the config for the combiner
                pub fn combiner_config(mut self, config: C::Config) -> Self {
                    self.combiner_config = config;
                    self
                }
            }
            impl<const D : usize, G: GridGenerator<D>>
                GridNoiseBuilder<D, Ridged, G> {
                /// Controls how much the previous octave's ridge height is allowed to
                /// contribute.
                ///
                /// # Default
                /// `2.0`
                pub fn gain(mut self, gain: f32) -> Self {
                    self.combiner_config.gain = gain;
                    self
                }
            }
            impl<const D : usize, G: GridGenerator<D>>
                GridNoiseBuilder<D, PingPong, G> {
                /// Controls how aggressively the noise output is folded.
                ///
                /// # Default
                /// `2.0`
                pub fn strength(mut self, strength: f32) -> Self {
                    self.combiner_config.strength = strength;
                    self
                }
            }
            impl<const D : usize, G: GridGenerator<D>>
                GridNoiseBuilder<D, Terrace, G> {
                /// Controls how many steps the final noise output is quantized across.
                ///
                /// # Default
                /// `8.0`
                pub fn steps(mut self, steps: f32) -> Self {
                    self.combiner_config.steps = steps;
                    self.combiner_config.step_size = 1.0 / steps;
                    self
                }
            }
            impl<const D : usize, G: GridGenerator<D>>
                GridNoiseBuilder<D, HybridMulti, G> {
                /// # Default
                /// `2.0`
                pub fn gain(mut self, gain: f32) -> Self {
                    self.combiner_config.gain = gain;
                    self
                }
                /// # Default
                /// `1.0`
                pub fn offset(mut self, offset: f32) -> Self {
                    self.combiner_config.offset = offset;
                    self
                }
            }
            /// Controls how much each axis of the grid is 'stretched' in the noise
            /// sample space. Creates visible stretching in the noise output.
            /// The default values have no stretching.
            ///
            /// # Default
            ///  - `1.0`: x_scaling
            ///  - `1.0`: y_scaling
            impl<C: Combiner, G: GridGenerator<2>> GridNoiseBuilder<2, C, G> {
                pub fn scaling(mut self, x_scaling: f32, y_scaling: f32)
                    -> Self {
                    self.noise_config.scaling = [x_scaling, y_scaling];
                    self
                }
            }
            impl<C: Combiner, G: GridGenerator<3>> GridNoiseBuilder<3, C, G> {
                /// Controls how much each axis of the grid is 'stretched' in the noise
                /// sample space. Creates visible stretching in the noise output.
                /// The default values have no stretching.
                ///
                /// # Default
                ///  - `1.0`: x_scaling
                ///  - `1.0`: y_scaling
                ///  - `1.0`: z_scaling
                pub fn scaling(mut self, x_scaling: f32, y_scaling: f32,
                    z_scaling: f32) -> Self {
                    self.noise_config.scaling =
                        [x_scaling, y_scaling, z_scaling];
                    self
                }
            }
            impl<const D : usize, C: Combiner, G: GridGenerator<D>>
                GridNoiseBuilder<D, C, G> {
                #[inline(always)]
                pub(crate) fn from_config(grid_config: GridConfig<D>)
                    -> Self {
                    Self { grid_config, ..Default::default() }
                }
                /// Creates the noise and returns the result in an output array.
                ///
                /// Needs to know the length of the output SimdArray because
                /// const generic expr is not yet available in stable Rust when this was
                /// created.
                pub fn build(self) -> Vec<f32> {
                    let size = self.grid_config.grid_size.iter().product();
                    let mut result = ::alloc::vec::from_elem(0.0, size);
                    GridNoise::<D, C,
                            G>::sample::<StaticArch>(&self.grid_config,
                        &self.noise_config, &self.combiner_config,
                        result.as_mut_slice());
                    result
                }
                /// Creates the noise and puts the result in a given array.
                pub fn fill(self, result: &mut [f32]) {
                    GridNoise::<D, C,
                            G>::sample::<StaticArch>(&self.grid_config,
                        &self.noise_config, &self.combiner_config, result);
                }
                /// Returns an iterator containing chunks of the noise output.
                /// Ideal for managing streams of noise without unnecessary read/writes.
                #[allow(clippy :: should_implement_trait)]
                pub fn into_iter(self)
                    ->
                        impl Iterator<Item = crate::simd::Simd<f32, StaticArch>> {
                    self.build().into_simd_iter()
                }
            }
        }
        pub mod interface {
            use std::marker::PhantomData;
            use crate::api::configs::GridConfig;
            use crate::api::grid::builder::GridNoiseBuilder;
            use crate::api::grid::octaves_builder::OctaveGridNoiseBuilder;
            use crate::math::random::Random;
            use crate::simd::Arch;
            use crate::{Combiner, Octave};
            /// Handles raw parameters for grid noise generators
            pub struct GridNoiseParams<const D : usize> {
                pub seed: u32,
                pub grid_size: [usize; D],
                pub position: [i32; D],
                pub frequency: [f32; D],
                pub weight: f32,
                pub magnification: f32,
                pub tiling: [Option<u32>; D],
            }
            #[automatically_derived]
            impl<const D : usize> ::core::marker::Copy for GridNoiseParams<D>
                {
            }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl<const D : usize> ::core::clone::TrivialClone for
                GridNoiseParams<D> {
            }
            #[automatically_derived]
            impl<const D : usize> ::core::clone::Clone for GridNoiseParams<D>
                {
                #[inline]
                fn clone(&self) -> GridNoiseParams<D> {
                    let _: ::core::clone::AssertParamIsClone<u32>;
                    let _: ::core::clone::AssertParamIsClone<[usize; D]>;
                    let _: ::core::clone::AssertParamIsClone<[i32; D]>;
                    let _: ::core::clone::AssertParamIsClone<[f32; D]>;
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    let _: ::core::clone::AssertParamIsClone<[Option<u32>; D]>;
                    *self
                }
            }
            #[automatically_derived]
            impl<const D : usize> ::core::marker::StructuralPartialEq for
                GridNoiseParams<D> {
            }
            #[automatically_derived]
            impl<const D : usize> ::core::cmp::PartialEq for
                GridNoiseParams<D> {
                #[inline]
                fn eq(&self, other: &GridNoiseParams<D>) -> bool {
                    self.seed == other.seed && self.weight == other.weight &&
                                        self.magnification == other.magnification &&
                                    self.grid_size == other.grid_size &&
                                self.position == other.position &&
                            self.frequency == other.frequency &&
                        self.tiling == other.tiling
                }
            }
            #[automatically_derived]
            impl<const D : usize> ::core::fmt::Debug for GridNoiseParams<D> {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    let names: &'static _ =
                        &["seed", "grid_size", "position", "frequency", "weight",
                                    "magnification", "tiling"];
                    let values: &[&dyn ::core::fmt::Debug] =
                        &[&self.seed, &self.grid_size, &self.position,
                                    &self.frequency, &self.weight, &self.magnification,
                                    &&self.tiling];
                    ::core::fmt::Formatter::debug_struct_fields_finish(f,
                        "GridNoiseParams", names, values)
                }
            }
            pub trait GridGenerator<const D : usize>: Default + Copy + Clone +
                PartialEq {
                /// Generates noise for a grid region.
                ///
                /// # Type Parameters
                /// - `C`: Type of combiner used to layer noise
                /// - `INIT`: Whether or not the generator should initialize dst
                /// - `FINAL`: Whether or not the generator should finalize the final octave
                ///
                /// # Runtime Parameters
                /// - `params`: Config specifying general noise parameters
                /// - `combiner_config`: Config specifying combiner parameters
                /// - `state`: Buffer containing sample information across octaves
                /// - `dst`: Buffer to insert the results into
                fn sample_grid<F: Arch, C: Combiner, const INIT : bool, const
                FINAL :
                bool>(params: GridNoiseParams<D>, combiner_config: C::Config,
                state: &mut [f32], dst: &mut [f32]);
            }
            /// Static struct for sampling grid noise.
            pub struct GridNoise<const D : usize, C: Combiner,
                S: GridGenerator<D>> {
                _fractal: PhantomData<C>,
                _sampler: PhantomData<S>,
            }
            /// An interface struct for creating grid noise.
            ///
            /// # Type Parameters
            /// * `D: NoiseDimension` - Determines how many dimensions the grid has.
            ///
            /// # Example
            /// ```
            /// use quick_noise::Grid;
            ///
            /// // Subject to change.
            /// let grid = Grid::<2>::new(32, 32)
            ///     .grid_position(0, 0)
            ///     .seed(1);
            /// ```
            pub struct Grid<const D : usize> {
                pub(crate) config: GridConfig<D>,
            }
            #[automatically_derived]
            impl<const D : usize> ::core::default::Default for Grid<D> {
                #[inline]
                fn default() -> Grid<D> {
                    Grid { config: ::core::default::Default::default() }
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl<const D : usize> ::core::clone::TrivialClone for
                Grid<D> {
            }
            #[automatically_derived]
            impl<const D : usize> ::core::clone::Clone for Grid<D> {
                #[inline]
                fn clone(&self) -> Grid<D> {
                    let _: ::core::clone::AssertParamIsClone<GridConfig<D>>;
                    *self
                }
            }
            #[automatically_derived]
            impl<const D : usize> ::core::marker::Copy for Grid<D> { }
            #[automatically_derived]
            impl<const D : usize> ::core::fmt::Debug for Grid<D> {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field1_finish(f,
                        "Grid", "config", &&self.config)
                }
            }
            #[automatically_derived]
            impl<const D : usize> ::core::marker::StructuralPartialEq for
                Grid<D> {
            }
            #[automatically_derived]
            impl<const D : usize> ::core::cmp::PartialEq for Grid<D> {
                #[inline]
                fn eq(&self, other: &Grid<D>) -> bool {
                    self.config == other.config
                }
            }
            impl<const D : usize> Grid<D> {
                /// Determines the psuedo-random values used in noise generation called
                /// on this grid. Different seeds produce different noise.
                pub fn seed(mut self, seed: i64) -> Self {
                    self.config.grid_seed = Random::mix_u64(seed as u64);
                    self
                }
            }
            impl Grid<2> {
                /// Creates an anchor for a grid region that can be used for call noise.
                ///
                /// # Parameters
                /// -`x`: Length of the grid region along the x-axis
                /// -`y`: Length of the grid region along the y-axis
                pub fn new(x: usize, y: usize) -> Self {
                    let config =
                        GridConfig { grid_size: [x, y], ..Default::default() };
                    Self { config }
                }
                /// Determines the position values provided to noise calls. This value represents
                /// the position of this grid region in grid units determined by its grid_size.
                /// A 32x32 grid at position `{ 1, 2 }` covers samples in the range `{ 32..64, 64..96 }`.
                ///
                /// # Default:
                /// `0`: x
                /// `0`: y
                pub fn grid_position(mut self, x: i32, y: i32) -> Self {
                    self.config.position =
                        [x * self.config.grid_size[0] as i32,
                                y * self.config.grid_size[1] as i32];
                    self
                }
                /// Determines the position values provided to noise calls. This value represents
                /// the position of first sample in each dimension. A 32x32 given the sample position
                /// `{ 32, 16 }` covers samples in the range `{ 32..64, 16..48 }`.
                ///
                /// # Default:
                /// `0`: x
                /// `0`: y
                pub fn sample_position(mut self, x: i32, y: i32) -> Self {
                    self.config.position = [x, y];
                    self
                }
                /// Determines the distance the sample space has until it starts repeating noise
                /// seamlessly. When values are left as None, noise does not repeat.
                ///
                /// # Default:
                /// - `x`: None
                /// - `y`: None
                pub fn tiling(mut self, x: Option<u32>, y: Option<u32>)
                    -> Self {
                    self.config.tiling = [x, y];
                    self
                }
            }
            impl Grid<3> {
                /// Creates an anchor for a grid region that can be used for call noise.
                ///
                /// # Parameters
                /// -`x`: Length of the grid region along the x-axis
                /// -`y`: Length of the grid region along the y-axis
                /// -`z`: Length of the grid region along the z-axis
                pub fn new(x: usize, y: usize, z: usize) -> Self {
                    let config =
                        GridConfig { grid_size: [x, y, z], ..Default::default() };
                    Self { config }
                }
                /// Determines the position values provided to noise calls. This value represents
                /// the position of this grid region in grid units determined by its grid_size.
                /// A 32x32x32 grid at position `{ 1, 2, 3 }` covers samples in the range
                /// `{ 32..64, 64..96, 96..128 }`.
                ///
                /// # Default:
                /// `0`: x
                /// `0`: y
                /// `0`: z
                pub fn grid_position(mut self, x: i32, y: i32, z: i32)
                    -> Self {
                    self.config.position =
                        [x * self.config.grid_size[0] as i32,
                                y * self.config.grid_size[1] as i32,
                                z * self.config.grid_size[2] as i32];
                    self
                }
                /// Determines the position values provided to noise calls. This value represents
                /// the position of first sample in each dimension. A 32x32x32 given the sample position
                /// `{ 32, 16, 0 }` covers samples in the range `{ 32..64, 16..48, 0..32 }`.
                ///
                /// # Default:
                /// `0`: x
                /// `0`: y
                /// `0`: z
                pub fn sample_position(mut self, x: i32, y: i32, z: i32)
                    -> Self {
                    self.config.position = [x, y, z];
                    self
                }
                /// Determines the distance the sample space has until it starts repeating noise
                /// seamlessly. When values are left as None, noise does not repeat.
                ///
                /// # Default:
                /// - `x`: None
                /// - `y`: None
                /// - `z`: None
                pub fn tiling(mut self, x: Option<u32>, y: Option<u32>,
                    z: Option<u32>) -> Self {
                    self.config.tiling = [x, y, z];
                    self
                }
            }
            impl<const D : usize> Grid<D> {
                /// Loads a config a config to create a grid.
                pub fn from_config(config: GridConfig<D>) -> Self {
                    Self { config }
                }
                /// Creates a new builder to easily configure a grid region of noise.
                pub fn builder<F: Combiner, T: GridGenerator<D>>(&self)
                    -> GridNoiseBuilder<D, F, T> {
                    GridNoiseBuilder::from_config(self.config)
                }
                /// Creates a new builder using a custom octave list to configure
                /// a grid region of noise.
                pub fn builder_with_octaves<'a, F: Combiner,
                    T: GridGenerator<D>>(&self, octave_list: &'a [Octave<D>])
                    -> OctaveGridNoiseBuilder<'a, D, F, T> {
                    OctaveGridNoiseBuilder::new(self.config, octave_list)
                }
            }
        }
        pub mod iters {
            use std::marker::PhantomData;
            use crate::Grid;
            use crate::simd::{Arch, Mask, Simd, StaticArch};
            use crate::simd::array_trait::Array;
            impl Grid<2> {
                #[inline(always)]
                pub fn x_iter(&self) -> RowIter<StaticArch> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    RowIter::new(dim[0], dim[1], pos[0] as f32)
                }
                #[inline(always)]
                pub fn y_iter(&self) -> SliceIter<StaticArch> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    SliceIter::new(dim[0], dim[1], 1, pos[1] as f32)
                }
                #[inline(always)]
                pub fn x_iter_with_arch<A: Arch>(&self) -> RowIter<A> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    RowIter::new(dim[0], dim[1], pos[0] as f32)
                }
                #[inline(always)]
                pub fn y_iter_with_arch<A: Arch>(&self) -> SliceIter<A> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    SliceIter::new(dim[0], dim[1], 1, pos[1] as f32)
                }
            }
            impl Grid<3> {
                #[inline(always)]
                pub fn x_iter(&self) -> RowIter<StaticArch> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    RowIter::new(dim[0], dim[1] * dim[2], pos[0] as f32)
                }
                #[inline(always)]
                pub fn y_iter(&self) -> SliceIter<StaticArch> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    SliceIter::new(dim[0], dim[1], dim[2], pos[1] as f32)
                }
                #[inline(always)]
                pub fn z_iter(&self) -> SliceIter<StaticArch> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    SliceIter::new(dim[0] * dim[1], dim[2], 1, pos[2] as f32)
                }
                #[inline(always)]
                pub fn x_iter_with_arch<A: Arch>(&self) -> RowIter<A> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    RowIter::new(dim[0], dim[1] * dim[2], pos[0] as f32)
                }
                #[inline(always)]
                pub fn y_iter_with_arch<A: Arch>(&self) -> SliceIter<A> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    SliceIter::new(dim[0], dim[1], dim[2], pos[1] as f32)
                }
                #[inline(always)]
                pub fn z_iter_with_arch<A: Arch>(&self) -> SliceIter<A> {
                    let pos = self.config.position;
                    let dim = self.config.grid_size;
                    SliceIter::new(dim[0] * dim[1], dim[2], 1, pos[2] as f32)
                }
            }
            pub struct RowIter<A: Arch> {
                row_size: usize,
                left_in_row: usize,
                rows_left: usize,
                cur_vec: Simd<f32, A>,
                start_vec: Simd<f32, A>,
                _arch: PhantomData<A>,
            }
            #[automatically_derived]
            impl<A: ::core::fmt::Debug + Arch> ::core::fmt::Debug for
                RowIter<A> {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    let names: &'static _ =
                        &["row_size", "left_in_row", "rows_left", "cur_vec",
                                    "start_vec", "_arch"];
                    let values: &[&dyn ::core::fmt::Debug] =
                        &[&self.row_size, &self.left_in_row, &self.rows_left,
                                    &self.cur_vec, &self.start_vec, &&self._arch];
                    ::core::fmt::Formatter::debug_struct_fields_finish(f,
                        "RowIter", names, values)
                }
            }
            impl<A: Arch> RowIter<A> {
                fn new(row_size: usize, num_rows: usize, start_val: f32)
                    -> Self {
                    Self {
                        row_size,
                        left_in_row: row_size,
                        rows_left: num_rows - 1,
                        cur_vec: Simd::iota(start_val),
                        start_vec: Simd::iota(start_val),
                        _arch: PhantomData::<A>,
                    }
                }
            }
            impl<A: Arch> Iterator for RowIter<A> {
                type Item = Simd<f32, A>;
                #[inline(always)]
                fn next(&mut self) -> Option<Self::Item> {
                    if self.row_size < Simd::<f32, A>::LANES {
                        if self.left_in_row == 0 && self.rows_left == 0 {
                            return None;
                        }
                        let mut cur = self.cur_vec.to_array()[0];
                        let start = self.start_vec.to_array()[0];
                        let array =
                            A::Array32::<f32>::from_fn(|_|
                                    {
                                        if self.left_in_row > 0 {
                                            let next = cur;
                                            cur += 1.0;
                                            self.left_in_row -= 1;
                                            next
                                        } else if self.rows_left > 0 {
                                            cur = start + 1.0;
                                            self.rows_left -= 1;
                                            self.left_in_row = self.row_size - 1;
                                            start
                                        } else { 0.0 }
                                    });
                        self.cur_vec = Simd::iota(cur);
                        return Some(Simd::from_slice(array.as_slice()));
                    }
                    if self.left_in_row >= Simd::<f32, A>::LANES {
                        let next = self.cur_vec;
                        self.cur_vec += Simd::splat(Simd::<f32, A>::LANES as f32);
                        self.left_in_row -= Simd::<f32, A>::LANES;
                        return Some(next);
                    }
                    if self.rows_left > 0 {
                        let mask = Mask::first_n_true(self.left_in_row as u32);
                        let old = self.cur_vec;
                        let next =
                            self.start_vec - Simd::splat(self.left_in_row as f32);
                        self.left_in_row += self.row_size - Simd::<f32, A>::LANES;
                        self.rows_left -= 1;
                        self.cur_vec =
                            next + Simd::splat(Simd::<f32, A>::LANES as f32);
                        return Some(mask.select(old, next));
                    }
                    if self.left_in_row > 0 {
                        let mask = Mask::first_n_true(self.left_in_row as u32);
                        let vec = self.cur_vec;
                        self.left_in_row = 0;
                        return Some(mask.select(vec, Simd::zero()));
                    }
                    None
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    let left =
                        self.left_in_row + self.rows_left * self.row_size;
                    let chunks_left = left.div_ceil(Simd::<f32, A>::LANES);
                    (chunks_left, Some(chunks_left))
                }
            }
            pub struct SliceIter<A: Arch> {
                row_size: usize,
                slice_size: usize,
                left_in_row: usize,
                left_in_slice: usize,
                slices_left: usize,
                cur_val: f32,
                start_val: f32,
                _arch: PhantomData<A>,
            }
            #[automatically_derived]
            impl<A: ::core::fmt::Debug + Arch> ::core::fmt::Debug for
                SliceIter<A> {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    let names: &'static _ =
                        &["row_size", "slice_size", "left_in_row", "left_in_slice",
                                    "slices_left", "cur_val", "start_val", "_arch"];
                    let values: &[&dyn ::core::fmt::Debug] =
                        &[&self.row_size, &self.slice_size, &self.left_in_row,
                                    &self.left_in_slice, &self.slices_left, &self.cur_val,
                                    &self.start_val, &&self._arch];
                    ::core::fmt::Formatter::debug_struct_fields_finish(f,
                        "SliceIter", names, values)
                }
            }
            impl<A: Arch> SliceIter<A> {
                pub fn new(row_size: usize, slice_size: usize,
                    num_slices: usize, start_val: f32) -> Self {
                    Self {
                        row_size,
                        slice_size,
                        left_in_row: row_size,
                        left_in_slice: slice_size - 1,
                        slices_left: num_slices - 1,
                        cur_val: start_val,
                        start_val,
                        _arch: PhantomData::<A>,
                    }
                }
            }
            impl<A: Arch> Iterator for SliceIter<A> {
                type Item = Simd<f32, A>;
                #[inline(always)]
                fn next(&mut self) -> Option<Self::Item> {
                    if self.row_size < Simd::<f32, A>::LANES {
                        if self.left_in_row == 0 && self.left_in_slice == 0 &&
                                self.slices_left == 0 {
                            return None;
                        }
                        let array =
                            A::Array32::<f32>::from_fn(|_|
                                    {
                                        if self.left_in_row > 0 {
                                            self.left_in_row -= 1;
                                            self.cur_val
                                        } else if self.left_in_slice > 0 {
                                            self.cur_val += 1.0;
                                            self.left_in_row = self.row_size - 1;
                                            self.left_in_slice -= 1;
                                            self.cur_val
                                        } else if self.slices_left > 0 {
                                            self.cur_val = self.start_val;
                                            self.left_in_row = self.row_size - 1;
                                            self.left_in_slice = self.slice_size - 1;
                                            self.slices_left -= 1;
                                            self.start_val
                                        } else { 0.0 }
                                    });
                        return Some(Simd::from_slice(array.as_slice()));
                    }
                    if self.left_in_row >= Simd::<f32, A>::LANES {
                        self.left_in_row -= Simd::<f32, A>::LANES;
                        return Some(Simd::splat(self.cur_val));
                    }
                    if self.left_in_slice > 0 {
                        let old = Simd::splat(self.cur_val);
                        self.cur_val += 1.0;
                        let new = Simd::splat(self.cur_val);
                        let mask = Mask::first_n_true(self.left_in_row as u32);
                        self.left_in_row += self.row_size - Simd::<f32, A>::LANES;
                        self.left_in_slice -= 1;
                        return Some(mask.select(old, new));
                    }
                    if self.slices_left > 0 {
                        let old = Simd::splat(self.cur_val);
                        let new = Simd::splat(self.start_val);
                        self.cur_val = self.start_val;
                        let mask = Mask::first_n_true(self.left_in_row as u32);
                        self.left_in_row += self.row_size - Simd::<f32, A>::LANES;
                        self.left_in_slice = self.slice_size - 1;
                        self.slices_left -= 1;
                        return Some(mask.select(old, new));
                    }
                    if self.left_in_row > 0 {
                        let old = Simd::splat(self.cur_val);
                        let mask = Mask::first_n_true(self.left_in_row as u32);
                        self.left_in_row = 0;
                        return Some(mask.select(old, Simd::zero()));
                    }
                    None
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    let left_in_slice =
                        self.left_in_row + self.left_in_slice * self.row_size;
                    let left_after_slice =
                        self.slices_left * self.row_size * self.slice_size;
                    let left = left_in_slice + left_after_slice;
                    let chunks_left = left.div_ceil(Simd::<f32, A>::LANES);
                    (chunks_left, Some(chunks_left))
                }
            }
        }
        pub mod octaves_builder {
            use std::marker::PhantomData;
            use crate::api::configs::*;
            use crate::api::grid::interface::GridNoise;
            use crate::api::octave::Octave;
            use crate::api::parameters::*;
            use crate::math::random::Random;
            use crate::simd::StaticArch;
            use crate::simd::register::iters::IntoSimdIterator;
            use crate::{
                Combiner, GridGenerator, HybridMulti, PingPong, Ridged,
                Terrace,
            };
            /// A struct for creating 2D Perlin noise set on a uniform grid with
            /// a custom list of octaves. Uses the performant perlin algorithm.
            pub struct OctaveGridNoiseBuilder<'a, const D : usize,
                C: Combiner, G: GridGenerator<D>> {
                grid_config: GridConfig<D>,
                noise_config: NoiseConfig<D>,
                combiner_config: C::Config,
                octave_list: &'a [Octave<D>],
                _noise_type: PhantomData<G>,
            }
            #[automatically_derived]
            impl<'a, const D : usize, C: ::core::default::Default + Combiner,
                G: ::core::default::Default + GridGenerator<D>>
                ::core::default::Default for
                OctaveGridNoiseBuilder<'a, D, C, G> where
                C::Config: ::core::default::Default {
                #[inline]
                fn default() -> OctaveGridNoiseBuilder<'a, D, C, G> {
                    OctaveGridNoiseBuilder {
                        grid_config: ::core::default::Default::default(),
                        noise_config: ::core::default::Default::default(),
                        combiner_config: ::core::default::Default::default(),
                        octave_list: ::core::default::Default::default(),
                        _noise_type: ::core::default::Default::default(),
                    }
                }
            }
            impl<'a, const D : usize, C: Combiner, G: GridGenerator<D>>
                OctaveGridNoiseBuilder<'a, D, C, G> {
                /// Determines the psuedo-random values used in noise generation.
                /// Different seeds produce different noise.
                pub fn seed(mut self, seed: i64) -> Self {
                    self.noise_config.seed =
                        Random::mix_u64(seed as u64 ^ 0xD5E7B3C94F8A1E6B);
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
                pub fn initialize(mut self, initialize: bool) -> Self {
                    self.noise_config.initialize = initialize;
                    self
                }
                /// Determines whether or not to finalize the values after the final octave.
                /// This finalization uses what is defined by the [Fractal] type.
                pub fn finalize(mut self, finalize: bool) -> Self {
                    self.noise_config.finalize = finalize;
                    self
                }
            }
            impl<'a, const D : usize, C: Combiner<Config : Sized>,
                G: GridGenerator<D>> OctaveGridNoiseBuilder<'a, D, C, G> {
                /// Configures the config for the combiner
                pub fn combiner_config(mut self, config: C::Config) -> Self {
                    self.combiner_config = config;
                    self
                }
            }
            impl<'a, const D : usize, G: GridGenerator<D>>
                OctaveGridNoiseBuilder<'a, D, Ridged, G> {
                /// Controls how much the previous octave's ridge height is allowed to
                /// contribute.
                ///
                /// # Default
                /// `2.0`
                pub fn gain(mut self, gain: f32) -> Self {
                    self.combiner_config.gain = gain;
                    self
                }
            }
            impl<'a, const D : usize, G: GridGenerator<D>>
                OctaveGridNoiseBuilder<'a, D, PingPong, G> {
                /// Controls how aggressively the noise output is folded.
                ///
                /// # Default
                /// `2.0`
                pub fn strength(mut self, strength: f32) -> Self {
                    self.combiner_config.strength = strength;
                    self
                }
            }
            impl<'a, const D : usize, G: GridGenerator<D>>
                OctaveGridNoiseBuilder<'a, D, Terrace, G> {
                /// Controls how many steps the final noise output is quantized across.
                ///
                /// # Default
                /// `8.0`
                pub fn steps(mut self, steps: f32) -> Self {
                    self.combiner_config.steps = steps;
                    self.combiner_config.step_size = 1.0 / steps;
                    self
                }
            }
            impl<'a, const D : usize, G: GridGenerator<D>>
                OctaveGridNoiseBuilder<'a, D, HybridMulti, G> {
                /// # Default
                /// `2.0`
                pub fn gain(mut self, gain: f32) -> Self {
                    self.combiner_config.gain = gain;
                    self
                }
                /// # Default
                /// `1.0`
                pub fn offset(mut self, offset: f32) -> Self {
                    self.combiner_config.offset = offset;
                    self
                }
            }
            /// Controls how much each axis of the grid is 'stretched' in the noise
            /// sample space. Creates visible stretching in the noise output.
            /// The default values have no stretching.
            ///
            /// # Default
            ///  - `1.0`: x_scaling
            ///  - `1.0`: y_scaling
            impl<'a, C: Combiner, G: GridGenerator<2>>
                OctaveGridNoiseBuilder<'a, 2, C, G> {
                pub fn scaling(mut self, x_scaling: f32, y_scaling: f32)
                    -> Self {
                    self.noise_config.scaling = [x_scaling, y_scaling];
                    self
                }
            }
            impl<'a, C: Combiner, G: GridGenerator<3>>
                OctaveGridNoiseBuilder<'a, 3, C, G> {
                /// Controls how much each axis of the grid is 'stretched' in the noise
                /// sample space. Creates visible stretching in the noise output.
                /// The default values have no stretching.
                ///
                /// # Default
                ///  - `1.0`: x_scaling
                ///  - `1.0`: y_scaling
                ///  - `1.0`: z_scaling
                pub fn scaling(mut self, x_scaling: f32, y_scaling: f32,
                    z_scaling: f32) -> Self {
                    self.noise_config.scaling =
                        [x_scaling, y_scaling, z_scaling];
                    self
                }
            }
            impl<'a, const D : usize, C: Combiner, G: GridGenerator<D>>
                OctaveGridNoiseBuilder<'a, D, C, G> {
                pub(crate) fn new(grid_config: GridConfig<D>,
                    octave_list: &'a [Octave<D>]) -> Self {
                    Self { grid_config, octave_list, ..Default::default() }
                }
                /// Creates the noise and returns the result in an output array.
                ///
                /// Needs to know the length of the output SimdArray because
                /// const generic expr is not yet available in stable Rust when this was
                /// created.
                pub fn build(self) -> Vec<f32> {
                    let size = self.grid_config.grid_size.iter().product();
                    let mut result = ::alloc::vec::from_elem(0.0, size);
                    GridNoise::<D, C,
                            G>::sample_with_octaves::<StaticArch>(&self.grid_config,
                        &self.noise_config, &self.combiner_config, self.octave_list,
                        result.as_mut_slice());
                    result
                }
                /// Creates the noise and puts the result in a given array.
                pub fn fill(self, result: &mut [f32]) {
                    GridNoise::<D, C,
                            G>::sample_with_octaves::<StaticArch>(&self.grid_config,
                        &self.noise_config, &self.combiner_config, self.octave_list,
                        result);
                }
                /// Returns an iterator containing chunks of the noise output.
                /// Ideal for managing streams of noise without unnecessary read/writes.
                #[allow(clippy :: should_implement_trait)]
                pub fn into_iter(self)
                    ->
                        impl Iterator<Item = crate::simd::Simd<f32, StaticArch>> {
                    self.build().into_simd_iter()
                }
            }
        }
        pub mod octaves_sample {
            use crate::api::configs::*;
            use crate::api::grid::interface::{GridNoise, GridNoiseParams};
            use crate::api::octave::Octave;
            use crate::api::seed::gen_octave_seed;
            use crate::noise::util::grid_helpers::{Arena, ArenaBuffer};
            use crate::math::random::Random;
            use crate::simd::Arch;
            use crate::{Combiner, CombinerState, GridGenerator};
            fn get_max<const D : usize>(array: [f32; D]) -> f32 {
                array.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
            }
            impl<const D : usize, C: Combiner, G: GridGenerator<D>>
                GridNoise<D, C, G> {
                #[inline(always)]
                pub fn sample_with_octaves<A: Arch>(grid_config:
                        &GridConfig<D>, noise_config: &NoiseConfig<D>,
                    combiner_config: &C::Config, octave_list: &[Octave<D>],
                    dst: &mut [f32]) {
                    let seed =
                        Random::mix_u64_pair(noise_config.seed,
                            grid_config.grid_seed);
                    let (num_octaves, total_weight): (usize, f32) =
                        octave_list.iter().filter(|x|
                                    get_max(x.frequency) <
                                        1.0).fold((0, 0.0), |t, x| (t.0 + 1, t.1 + x.weight));
                    let weight_coef =
                        match (noise_config.normalization, total_weight == 0.0) {
                            (false, _) => noise_config.amplitude,
                            (true, false) => noise_config.amplitude / total_weight,
                            (true, true) => 0.0,
                        };
                    if weight_coef == 0.0 { return; }
                    let mut params =
                        GridNoiseParams {
                            seed: 0,
                            grid_size: grid_config.grid_size,
                            position: grid_config.position,
                            frequency: [0.0; D],
                            weight: 0.0,
                            magnification: noise_config.magnification,
                            tiling: grid_config.tiling,
                        };
                    let total_size: usize =
                        grid_config.grid_size.iter().product();
                    let needed_state_size = total_size * C::State::STATE_SIZE;
                    let mut state_cache =
                        ArenaBuffer::with_capacity(needed_state_size);
                    let mut arena = Arena::with_cache(&mut state_cache);
                    let state = arena.allocate(needed_state_size);
                    let state = unsafe { state.assume_init_mut() };
                    let f_config = *combiner_config;
                    let mut octave_iter =
                        octave_list.iter().filter(|x| get_max(x.frequency) < 1.0);
                    if let Some(octave) = octave_iter.next() {
                        params.seed = gen_octave_seed(octave.frequency, seed);
                        params.frequency = octave.frequency;
                        params.weight = octave.weight * weight_coef;
                        match (noise_config.initialize,
                                noise_config.finalize && num_octaves == 1) {
                            (true, true) =>
                                G::sample_grid::<A, C, true,
                                        true>(params, f_config, state, dst),
                            (false, true) =>
                                G::sample_grid::<A, C, false,
                                        true>(params, f_config, state, dst),
                            (true, false) =>
                                G::sample_grid::<A, C, true,
                                        false>(params, f_config, state, dst),
                            (false, false) =>
                                G::sample_grid::<A, C, false,
                                        false>(params, f_config, state, dst),
                        }
                    }
                    for octave in
                        octave_iter.by_ref().take(num_octaves.saturating_sub(2)) {
                        params.seed = gen_octave_seed(octave.frequency, seed);
                        params.frequency = octave.frequency;
                        params.weight = octave.weight * weight_coef;
                        G::sample_grid::<A, C, false,
                                false>(params, f_config, state, dst);
                    }
                    if let Some(octave) = octave_iter.next() {
                        params.seed = gen_octave_seed(octave.frequency, seed);
                        params.frequency = octave.frequency;
                        params.weight = octave.weight * weight_coef;
                        match noise_config.finalize {
                            true =>
                                G::sample_grid::<A, C, false,
                                        true>(params, f_config, state, dst),
                            false =>
                                G::sample_grid::<A, C, false,
                                        false>(params, f_config, state, dst),
                        }
                    }
                }
            }
        }
        pub mod sample {
            use crate::api::configs::*;
            use crate::api::grid::interface::{
                GridGenerator, GridNoise, GridNoiseParams,
            };
            use crate::api::seed::gen_octave_seed;
            use crate::math::random::Random;
            use crate::noise::util::grid_helpers::{Arena, ArenaBuffer};
            use crate::simd::Arch;
            use crate::{Combiner, CombinerState};
            impl<const D : usize, C: Combiner, G: GridGenerator<D>>
                GridNoise<D, C, G> {
                #[inline(always)]
                pub fn sample<A: Arch>(grid_config: &GridConfig<D>,
                    noise_config: &NoiseConfig<D>, combiner_config: &C::Config,
                    result: &mut [f32]) {
                    let octaves = noise_config.num_grid_octaves();
                    if octaves == 0 {
                        if noise_config.initialize { result.fill(0.0) }
                        return;
                    }
                    let base_seed =
                        Random::mix_u64_pair(grid_config.grid_seed,
                            noise_config.seed);
                    let frequency =
                        std::array::from_fn(|i|
                                noise_config.scaling[i] * noise_config.frequency);
                    let weight =
                        if noise_config.normalization && C::WEIGHT_DECAY {
                            noise_config.normalize_amplitude(noise_config.amplitude)
                        } else { noise_config.amplitude };
                    let mut params =
                        GridNoiseParams {
                            seed: gen_octave_seed(frequency, base_seed),
                            grid_size: grid_config.grid_size,
                            position: grid_config.position,
                            magnification: noise_config.magnification,
                            tiling: grid_config.tiling,
                            frequency,
                            weight,
                        };
                    let total_size: usize =
                        grid_config.grid_size.iter().product();
                    let needed_state_size = total_size * C::State::STATE_SIZE;
                    let mut state_cache =
                        ArenaBuffer::with_capacity(needed_state_size);
                    let mut arena = Arena::with_cache(&mut state_cache);
                    let state = arena.allocate(needed_state_size);
                    let state = unsafe { state.assume_init_mut() };
                    let f_config = *combiner_config;
                    match (noise_config.initialize,
                            noise_config.finalize && octaves == 1) {
                        (false, false) =>
                            G::sample_grid::<A, C, false,
                                    false>(params, f_config, state, result),
                        (true, false) =>
                            G::sample_grid::<A, C, true,
                                    false>(params, f_config, state, result),
                        (false, true) =>
                            G::sample_grid::<A, C, false,
                                    true>(params, f_config, state, result),
                        (true, true) =>
                            G::sample_grid::<A, C, true,
                                    true>(params, f_config, state, result),
                    }
                    for _ in 1..(octaves.saturating_sub(2)) {
                        if C::WEIGHT_DECAY {
                            params.weight *= noise_config.persistence;
                        }
                        params.frequency =
                            std::array::from_fn(|i|
                                    params.frequency[i] * noise_config.lacunarity);
                        params.seed = gen_octave_seed(params.frequency, base_seed);
                        G::sample_grid::<A, C, false,
                                false>(params, f_config, state, result);
                    }
                    if octaves > 1 {
                        params.weight *= noise_config.persistence;
                        params.frequency =
                            std::array::from_fn(|i|
                                    params.frequency[i] * noise_config.lacunarity);
                        params.seed = gen_octave_seed(params.frequency, base_seed);
                        match noise_config.finalize {
                            true =>
                                G::sample_grid::<A, C, false,
                                        true>(params, f_config, state, result),
                            false =>
                                G::sample_grid::<A, C, false,
                                        false>(params, f_config, state, result),
                        }
                    }
                }
            }
        }
        pub mod warp {
            use std::iter::zip;
            use std::marker::PhantomData;
            use itertools::multizip;
            use crate::api::batch::interface::DimIter;
            use crate::simd::static_simd::StaticSimd;
            use crate::simd::{Arch, Simd};
            use crate::{BatchGenerator, BatchNoiseBuilder, Combiner, Grid};
            impl Grid<2> {
                pub fn warp_builder<C: Combiner, G: BatchGenerator<2>,
                    A: Arch>(&self, warp_strength: f32,
                    x_iter: impl Iterator<Item = Simd<f32, A>>,
                    y_iter: impl Iterator<Item = Simd<f32, A>>)
                    -> BatchNoiseBuilder<2, C, G, A, impl DimIter<A, 2>> {
                    let strength = Simd::splat(warp_strength);
                    let x_iter =
                        zip(x_iter,
                                self.x_iter_with_arch::<A>()).map(move |(x, grid)|
                                x.mul_add(strength, grid));
                    let y_iter =
                        zip(y_iter,
                                self.y_iter()).map(move |(y, grid)|
                                y.mul_add(strength, grid));
                    BatchNoiseBuilder {
                        iters: multizip((x_iter, y_iter)),
                        noise_config: Default::default(),
                        combiner_config: Default::default(),
                        _noise_type: PhantomData::<G>,
                        _arch: PhantomData::<A>,
                    }
                }
            }
            impl Grid<3> {
                pub fn warp_builder<C: Combiner,
                    G: BatchGenerator<3>>(&self, warp_strength: f32,
                    x_iter: impl Iterator<Item = StaticSimd<f32>>,
                    y_iter: impl Iterator<Item = StaticSimd<f32>>,
                    z_iter: impl Iterator<Item = StaticSimd<f32>>)
                    -> BatchNoiseBuilder<3, C, G, impl DimIter<3>> {
                    let strength = StaticSimd::splat(warp_strength);
                    let x_iter =
                        zip(x_iter,
                                self.x_iter()).map(move |(x, grid)|
                                x.mul_add(strength, grid));
                    let y_iter =
                        zip(y_iter,
                                self.y_iter()).map(move |(y, grid)|
                                y.mul_add(strength, grid));
                    let z_iter =
                        zip(z_iter,
                                self.z_iter()).map(move |(z, grid)|
                                z.mul_add(strength, grid));
                    BatchNoiseBuilder {
                        iters: multizip((x_iter, y_iter, z_iter)),
                        noise_config: Default::default(),
                        combiner_config: Default::default(),
                        _noise_type: PhantomData::<G>,
                    }
                }
            }
        }
        pub use builder::GridNoiseBuilder;
        pub use octaves_builder::OctaveGridNoiseBuilder;
    }
    pub mod octave {
        pub struct Octave<const D : usize> {
            pub weight: f32,
            pub frequency: [f32; D],
        }
        #[automatically_derived]
        impl<const D : usize> ::core::marker::Copy for Octave<D> { }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl<const D : usize> ::core::clone::TrivialClone for Octave<D>
            {
        }
        #[automatically_derived]
        impl<const D : usize> ::core::clone::Clone for Octave<D> {
            #[inline]
            fn clone(&self) -> Octave<D> {
                let _: ::core::clone::AssertParamIsClone<f32>;
                let _: ::core::clone::AssertParamIsClone<[f32; D]>;
                *self
            }
        }
        impl<const D : usize> Octave<D> {
            pub fn new(frequency: [f32; D], weight: f32) -> Self {
                Self { frequency, weight }
            }
            pub fn splat(frequency: f32, weight: f32) -> Self {
                let frequency = [frequency; D];
                Self { frequency, weight }
            }
        }
    }
    pub mod parameters {
        /// Common interface for execucting all noise builders.
        ///
        /// All builders support these three execution methods:
        ///  - `build()`: Creates a new Vec
        ///  - `into_iter()`: Get a lazy iterator
        ///  - `fill()`: insert data into an existing slice
        macro_rules! declare_build {
            ($self:ident, $body:tt) =>
            {
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
            ($arch:ident, $self:ident, $body:tt) =>
            {
                /// Returns an iterator containing chunks of the noise output.
                /// Ideal for managing streams of noise without unnecessary read/writes.
                #[allow(clippy::should_implement_trait)] pub fn
                into_iter($self) -> impl Iterator<Item =
                crate::simd::Simd<f32, $arch>> $body
            };
        }
        pub(crate) use declare_into_iter;
        macro_rules! declare_fill {
            ($self:ident, $result:ident, $body:tt) =>
            {
                /// Creates the noise and puts the result in a given array.
                pub fn fill($self, $result: &mut [f32]) $body
            };
        }
        pub(crate) use declare_fill;
        macro_rules! params_grid_seed_builder {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// Determines the psuedo-random values used in noise generation.
                    /// Can reproduce the same noise output as grid noise given the
                    /// same grid seed + noise seed pair.
                    pub fn
                    seed_with_grid(mut self, grid_seed: i64, noise_seed: i64) ->
                    Self
                    {
                        let grid_seed = Random::mix_u64(grid_seed as u64); let
                        noise_seed =
                        Random::mix_u64(noise_seed as u64 ^ 0xD5E7B3C94F8A1E6B);
                        self.noise_config.seed =
                        Random::mix_u64_pair(grid_seed, noise_seed); self
                    }
                }
            };
        }
        pub(crate) use params_grid_seed_builder;
        macro_rules! params_noise_builder {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// Determines the psuedo-random values used in noise generation.
                    /// Different seeds produce different noise.
                    pub fn seed(mut self, seed: i64) -> Self
                    {
                        self.noise_config.seed =
                        Random::mix_u64(seed as u64 ^ 0xD5E7B3C94F8A1E6B); self
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
                    pub fn amplitude(mut self, amplitude: f32) -> Self
                    { self.noise_config.amplitude = amplitude; self }
                    /// Controls the magnification of the noise output. For most use cases,
                    /// this value can be ignored. Useful for LODs or multi-quality noise
                    /// generation.
                    ///
                    /// # Default
                    /// `1.0`
                    pub fn magnification(mut self, magnification: f32) -> Self
                    { self.noise_config.magnification = magnification; self }
                    /// Controls whether or not normalization is performed. This ensures the noise
                    /// output is clamped according to the amplitude. When set to false, output
                    /// can be above the specified amplitude. For batched noise, normalization
                    /// can be expensive.
                    ///
                    /// # Default
                    /// `true`
                    pub fn normalization(mut self, normalization: bool) -> Self
                    { self.noise_config.normalization = normalization; self }
                    /// Determines whether or not to overwrite the values in the given slice.
                    /// When set to true, the current values are treated as previous octave samples.
                    ///
                    /// # Default
                    /// `true`
                    pub fn initialize(mut self, initialize: bool) -> Self
                    { self.noise_config.initialize = initialize; self }
                    /// Determines whether or not to finalize the values after the final octave.
                    /// This finalization uses what is defined by the [Fractal] type.
                    pub fn finalize(mut self, finalize: bool) -> Self
                    { self.noise_config.finalize = finalize; self }
                }
            };
        }
        pub(crate) use params_noise_builder;
        macro_rules! params_lacunarity_builder {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// Determines the number of perlin noise passes layered ontop of one another.
                    /// More octaves generally leads to more natural-appearing noise.
                    ///
                    /// # Default
                    /// `1`
                    pub fn octaves(mut self, octaves: usize) -> Self
                    { self.noise_config.octaves = octaves; self }
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
                    pub fn frequency(mut self, frequency: f32) -> Self
                    { self.noise_config.frequency = frequency; self }
                    /// Controls how the frequency changes after each subsequenct octave
                    /// (noise layer). The next octave's frequency is the previous octave's
                    /// frequency multiplied by the lacunarity.
                    ///
                    /// # Default
                    /// `2.0`
                    pub fn lacunarity(mut self, lacunarity: f32) -> Self
                    { self.noise_config.lacunarity = lacunarity; self }
                    /// Controls how much each subsequenct octave (noise layer) impacts
                    /// the final noise result. The next octave's weight is the previous octave's
                    /// frequency multiplied by the persistence.
                    ///
                    /// # Default
                    /// `0.5`
                    pub fn persistence(mut self, persistence: f32) -> Self
                    { self.noise_config.persistence = persistence; self }
                }
            };
        }
        pub(crate) use params_lacunarity_builder;
        macro_rules! params_noise_scaling_2d {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                /// Controls how much each axis of the grid is 'stretched' in the noise
                /// sample space. Creates visible stretching in the noise output.
                /// The default values have no stretching.
                ///
                /// # Default
                ///  - `1.0`: x_scaling
                ///  - `1.0`: y_scaling
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    pub fn scaling(mut self, x_scaling: f32, y_scaling: f32) ->
                    Self
                    { self.noise_config.scaling = [x_scaling, y_scaling]; self }
                }
            };
        }
        pub(crate) use params_noise_scaling_2d;
        macro_rules! params_noise_scaling_3d {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// Controls how much each axis of the grid is 'stretched' in the noise
                    /// sample space. Creates visible stretching in the noise output.
                    /// The default values have no stretching.
                    ///
                    /// # Default
                    ///  - `1.0`: x_scaling
                    ///  - `1.0`: y_scaling
                    ///  - `1.0`: z_scaling
                    pub fn
                    scaling(mut self, x_scaling: f32, y_scaling: f32, z_scaling:
                    f32) -> Self
                    {
                        self.noise_config.scaling =
                        [x_scaling, y_scaling, z_scaling]; self
                    }
                }
            };
        }
        pub(crate) use params_noise_scaling_3d;
        macro_rules! params_combiner_builder {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// Configures the config for the combiner
                    pub fn combiner_config(mut self, config: C::Config) -> Self
                    { self.combiner_config = config; self }
                }
            };
        }
        pub(crate) use params_combiner_builder;
        macro_rules! params_ridged_builder {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// Controls how much the previous octave's ridge height is allowed to
                    /// contribute.
                    ///
                    /// # Default
                    /// `2.0`
                    pub fn gain(mut self, gain: f32) -> Self
                    { self.combiner_config.gain = gain; self }
                }
            };
        }
        pub(crate) use params_ridged_builder;
        macro_rules! params_ping_pong_builder {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// Controls how aggressively the noise output is folded.
                    ///
                    /// # Default
                    /// `2.0`
                    pub fn strength(mut self, strength: f32) -> Self
                    { self.combiner_config.strength = strength; self }
                }
            };
        }
        pub(crate) use params_ping_pong_builder;
        macro_rules! params_terrace_builder {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// Controls how many steps the final noise output is quantized across.
                    ///
                    /// # Default
                    /// `8.0`
                    pub fn steps(mut self, steps: f32) -> Self
                    {
                        self.combiner_config.steps = steps;
                        self.combiner_config.step_size = 1.0 / steps; self
                    }
                }
            };
        }
        pub(crate) use params_terrace_builder;
        macro_rules! params_hybrid_multi_builder {
            ($name:ident, [$($full_generics:tt)*], [$($short_generics:tt)*])
            =>
            {
                impl< $($full_generics)* > $name< $($short_generics)* >
                {
                    /// # Default
                    /// `2.0`
                    pub fn gain(mut self, gain: f32) -> Self
                    { self.combiner_config.gain = gain; self } /// # Default
                    /// `1.0`
                    pub fn offset(mut self, offset: f32) -> Self
                    { self.combiner_config.offset = offset; self }
                }
            };
        }
        pub(crate) use params_hybrid_multi_builder;
    }
    pub mod seed {
        use crate::math::random::Random;
        /// Generates a psuedo-random seed for a single octave.
        pub fn gen_octave_seed<const D :
            usize>(frequencies: [f32; D], seed: u64) -> u32 {
            match D {
                0..2 => seed as u32,
                2 =>
                    Random::mix_u64_pair(seed.wrapping_mul(frequencies[0].to_bits()
                                    as u64), seed.wrapping_mul(frequencies[1].to_bits() as u64))
                        as u32,
                3 =>
                    Random::mix_u64_triple(seed.wrapping_mul(frequencies[0].to_bits()
                                    as u64), seed.wrapping_mul(frequencies[1].to_bits() as u64),
                            seed.wrapping_mul(frequencies[2].to_bits() as u64)) as u32,
                4.. => {
                    let mut cur_freq = frequencies[0].to_bits() as u64;
                    for new_freq in frequencies.iter().skip(1) {
                        cur_freq =
                            Random::mix_u64_pair(seed.wrapping_mul(cur_freq),
                                seed.wrapping_mul(new_freq.to_bits() as u64));
                    }
                    cur_freq as u32
                }
            }
        }
    }
    pub use batch::{BatchNoiseBuilder, OctaveBatchNoiseBuilder};
    pub use grid::{GridNoiseBuilder, OctaveGridNoiseBuilder};
}
pub mod math {
    pub mod random {
        /// Tiny module for fast psuedo-random bit mixing.
        pub struct Random {}
        impl Random {
            pub fn mix_u64(mut data: u64) -> u64 {
                data ^= 0xB820ABC04DB1A623;
                data ^= data >> 33;
                data = data.wrapping_mul(0xff51afd7ed558ccd);
                data ^= data >> 33;
                data = data.wrapping_mul(0xc4ceb9fe1a85ec53);
                data ^= data >> 33;
                data
            }
            pub fn mix_u64_pair(mut data1: u64, data2: u64) -> u64 {
                data1 ^= 0xB820ABC04DB1A623;
                data1 ^= data1 >> 33;
                data1 = data1.wrapping_mul(0xff51afd7ed558ccd ^ data2);
                data1 ^= data1 >> 33;
                data1 = data1.wrapping_mul(0xc4ceb9fe1a85ec53 ^ data2);
                data1 ^= data1 >> 33;
                data1
            }
            pub fn mix_u64_triple(mut data1: u64, data2: u64, data3: u64)
                -> u64 {
                data1 ^= 0xB820ABC04DB1A623;
                data1 ^= data1 >> 33;
                data1 = data1.wrapping_mul(0xff51afd7ed558ccd ^ data2);
                data1 ^= data1 >> 33;
                data1 = data1.wrapping_mul(0xc4ceb9fe1a85ec53 ^ data3);
                data1 ^= data1 >> 33;
                data1 = data1.wrapping_mul(0xff51afd7ed558ccd ^ data2);
                data1 ^= data1 >> 33;
                data1
            }
            pub fn mix_u32(mut data: u32) -> u32 {
                data ^= 0x7A019853;
                data ^= data >> 16;
                data = data.wrapping_mul(0x85ebca6b);
                data ^= data >> 13;
                data = data.wrapping_mul(0xc2b2ae35);
                data ^= data >> 16;
                data
            }
        }
    }
}
mod noise {
    pub mod generators {
        pub struct Perlin {}
        #[automatically_derived]
        impl ::core::default::Default for Perlin {
            #[inline]
            fn default() -> Perlin { Perlin {} }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Perlin { }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for Perlin { }
        #[automatically_derived]
        impl ::core::clone::Clone for Perlin {
            #[inline]
            fn clone(&self) -> Perlin { *self }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Perlin { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Perlin {
            #[inline]
            fn eq(&self, other: &Perlin) -> bool { true }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Perlin {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "Perlin")
            }
        }
        pub mod perlin {
            pub mod batch_2d {
                use std::f32::consts::SQRT_2;
                use crate::api::batch::interface::BatchGenerator;
                use crate::noise::generators::Perlin;
                use crate::simd::{Arch, Simd};
                pub const X_GRADIENTS_2D: [f32; 8] =
                    [SQRT_2, 1.0000000000000000, 0.0000000000000000,
                            -1.0000000000000000, -SQRT_2, -1.0000000000000000,
                            0.0000000000000000, 1.0000000000000000];
                pub const Y_GRADIENTS_2D: [f32; 8] =
                    [0.0000000000000000, 1.0000000000000000, SQRT_2,
                            1.0000000000000000, 0.0000000000000000, -1.0000000000000000,
                            -SQRT_2, -1.0000000000000000];
                impl BatchGenerator<2> for Perlin {
                    fn sample_batch<A: Arch>(seed: u32,
                        input: [Simd<f32, A>; 2], freq: [Simd<f32, A>; 2])
                        -> Simd<f32, A> {
                        let six: Simd<f32, A> = Simd::splat(6.0);
                        let ten: Simd<f32, A> = Simd::splat(10.0);
                        let fifteen: Simd<f32, A> = Simd::splat(15.0);
                        let one: Simd<f32, A> = Simd::splat(1.0);
                        const BYTE_SHUFFLE: [u8; 64] =
                            [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                    2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                    7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                    6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                        let shuffle_indices =
                            Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                        let channel_seed = Simd::splat(seed);
                        let prime = Simd::splat(0x85ebca6b_u32);
                        let x_scaled = input[0] * freq[0];
                        let y_scaled = input[1] * freq[1];
                        let x_scaled_floored = x_scaled.floor();
                        let y_scaled_floored = y_scaled.floor();
                        let x_grid_lo = x_scaled_floored.cast_int_trunc();
                        let y_grid_lo = y_scaled_floored.cast_int_trunc();
                        let x_dist_lo = x_scaled - x_scaled_floored;
                        let y_dist_lo = y_scaled - y_scaled_floored;
                        let x_dist_hi = x_dist_lo - one;
                        let y_dist_hi = y_dist_lo - one;
                        let t = x_dist_lo;
                        let s = y_dist_lo;
                        let x_lerp =
                            t * t * t * t.mul_add(t.mul_sub(six, fifteen), ten);
                        let y_lerp =
                            s * s * s * s.mul_add(s.mul_sub(six, fifteen), ten);
                        let x1: Simd<u32, A> = x_grid_lo.raw_cast() * channel_seed;
                        let y1: Simd<u32, A> = y_grid_lo.raw_cast() * channel_seed;
                        let x2 = x1 + channel_seed;
                        let y2 = y1 + channel_seed;
                        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
                        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
                        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
                        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
                        let mix_tl = x1_shuf * y1_shuf;
                        let mix_tr = x1_shuf * y2_shuf;
                        let mix_bl = x2_shuf * y1_shuf;
                        let mix_br = x2_shuf * y2_shuf;
                        let indices_tl = mix_tl >> 29;
                        let indices_tr = mix_tr >> 29;
                        let indices_bl = mix_bl >> 29;
                        let indices_br = mix_br >> 29;
                        let x_grads_tl = indices_tl.gather(&X_GRADIENTS_2D);
                        let y_grads_tl = indices_tl.gather(&Y_GRADIENTS_2D);
                        let x_grads_tr = indices_tr.gather(&X_GRADIENTS_2D);
                        let y_grads_tr = indices_tr.gather(&Y_GRADIENTS_2D);
                        let x_grads_bl = indices_bl.gather(&X_GRADIENTS_2D);
                        let y_grads_bl = indices_bl.gather(&Y_GRADIENTS_2D);
                        let x_grads_br = indices_br.gather(&X_GRADIENTS_2D);
                        let y_grads_br = indices_br.gather(&Y_GRADIENTS_2D);
                        let prod_tl =
                            x_grads_tl.mul_add(x_dist_lo, y_grads_tl * y_dist_lo);
                        let prod_tr =
                            x_grads_tr.mul_add(x_dist_lo, y_grads_tr * y_dist_hi);
                        let top_lerp = y_lerp.mul_add(prod_tr - prod_tl, prod_tl);
                        let prod_bl =
                            x_grads_bl.mul_add(x_dist_hi, y_grads_bl * y_dist_lo);
                        let prod_br =
                            x_grads_br.mul_add(x_dist_hi, y_grads_br * y_dist_hi);
                        let bottom_lerp =
                            y_lerp.mul_add(prod_br - prod_bl, prod_bl);
                        x_lerp.mul_add(bottom_lerp - top_lerp, top_lerp)
                    }
                }
            }
            pub mod batch_3d {
                use crate::api::batch::interface::BatchGenerator;
                use crate::noise::generators::Perlin;
                use crate::simd::{Arch, Simd};
                impl BatchGenerator<3> for Perlin {
                    fn sample_batch<A: Arch>(seed: u32,
                        input: [Simd<f32, A>; 3], freq: [Simd<f32, A>; 3])
                        -> Simd<f32, A> {
                        let six: Simd<f32, A> = Simd::splat(6.0);
                        let ten: Simd<f32, A> = Simd::splat(10.0);
                        let fifteen: Simd<f32, A> = Simd::splat(15.0);
                        let one: Simd<f32, A> = Simd::splat(1.0);
                        let three_int: Simd<u32, A> = Simd::splat(3);
                        let c1: Simd<u32, A> = Simd::splat(0x90A5A500);
                        let c2: Simd<u32, A> = Simd::splat(0xA59900A5);
                        let c3: Simd<u32, A> = Simd::splat(0x09009999);
                        const BYTE_SHUFFLE: [u8; 64] =
                            [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                    2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                    7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                    6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                        const GRAD_TABLE: [f32; 4] = [0.0, 1.0, -1.0, 0.0];
                        let shuffle_indices =
                            Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                        let channel_seed = Simd::splat(seed);
                        let prime = Simd::splat(0x85ebca6b_u32);
                        let x_scaled = input[0] * freq[0];
                        let y_scaled = input[1] * freq[1];
                        let z_scaled = input[2] * freq[2];
                        let x_scaled_floored = x_scaled.floor();
                        let y_scaled_floored = y_scaled.floor();
                        let z_scaled_floored = z_scaled.floor();
                        let x_grid_lo = x_scaled_floored.cast_int_trunc();
                        let y_grid_lo = y_scaled_floored.cast_int_trunc();
                        let z_grid_lo = z_scaled_floored.cast_int_trunc();
                        let x_dist_lo = x_scaled - x_scaled_floored;
                        let y_dist_lo = y_scaled - y_scaled_floored;
                        let z_dist_lo = z_scaled - z_scaled_floored;
                        let x_dist_hi = x_dist_lo - one;
                        let y_dist_hi = y_dist_lo - one;
                        let z_dist_hi = z_dist_lo - one;
                        let t = x_dist_lo;
                        let s = y_dist_lo;
                        let u = z_dist_lo;
                        let x_lerp =
                            t * t * t * t.mul_add(t.mul_sub(six, fifteen), ten);
                        let y_lerp =
                            s * s * s * s.mul_add(s.mul_sub(six, fifteen), ten);
                        let z_lerp =
                            u * u * u * u.mul_add(u.mul_sub(six, fifteen), ten);
                        let x1: Simd<u32, A> = x_grid_lo.raw_cast() * channel_seed;
                        let y1: Simd<u32, A> = y_grid_lo.raw_cast() * channel_seed;
                        let z1: Simd<u32, A> = z_grid_lo.raw_cast() * channel_seed;
                        let x2 = x1 + channel_seed;
                        let y2 = y1 + channel_seed;
                        let z2 = z1 + channel_seed;
                        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
                        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
                        let z1_shuf = z1.permute_8(shuffle_indices) ^ prime;
                        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
                        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
                        let z2_shuf = z2.permute_8(shuffle_indices) ^ prime;
                        let mix_tlf = x1_shuf * y1_shuf * z1_shuf;
                        let mix_trf = x1_shuf * y1_shuf * z2_shuf;
                        let mix_blf = x1_shuf * y2_shuf * z1_shuf;
                        let mix_brf = x1_shuf * y2_shuf * z2_shuf;
                        let mix_tlb = x2_shuf * y1_shuf * z1_shuf;
                        let mix_trb = x2_shuf * y1_shuf * z2_shuf;
                        let mix_blb = x2_shuf * y2_shuf * z1_shuf;
                        let mix_brb = x2_shuf * y2_shuf * z2_shuf;
                        let indices_tlf = (mix_tlf >> 28) << 1;
                        let indices_trf = (mix_trf >> 28) << 1;
                        let indices_blf = (mix_blf >> 28) << 1;
                        let indices_brf = (mix_brf >> 28) << 1;
                        let indices_tlb = (mix_tlb >> 28) << 1;
                        let indices_trb = (mix_trb >> 28) << 1;
                        let indices_blb = (mix_blb >> 28) << 1;
                        let indices_brb = (mix_brb >> 28) << 1;
                        let x_grads_tlf =
                            ((c1 >> indices_tlf) & three_int).gather(&GRAD_TABLE);
                        let x_grads_trf =
                            ((c1 >> indices_trf) & three_int).gather(&GRAD_TABLE);
                        let x_grads_blf =
                            ((c1 >> indices_blf) & three_int).gather(&GRAD_TABLE);
                        let x_grads_brf =
                            ((c1 >> indices_brf) & three_int).gather(&GRAD_TABLE);
                        let x_grads_tlb =
                            ((c1 >> indices_tlb) & three_int).gather(&GRAD_TABLE);
                        let x_grads_trb =
                            ((c1 >> indices_trb) & three_int).gather(&GRAD_TABLE);
                        let x_grads_blb =
                            ((c1 >> indices_blb) & three_int).gather(&GRAD_TABLE);
                        let x_grads_brb =
                            ((c1 >> indices_brb) & three_int).gather(&GRAD_TABLE);
                        let y_grads_tlf =
                            ((c2 >> indices_tlf) & three_int).gather(&GRAD_TABLE);
                        let y_grads_trf =
                            ((c2 >> indices_trf) & three_int).gather(&GRAD_TABLE);
                        let y_grads_blf =
                            ((c2 >> indices_blf) & three_int).gather(&GRAD_TABLE);
                        let y_grads_brf =
                            ((c2 >> indices_brf) & three_int).gather(&GRAD_TABLE);
                        let y_grads_tlb =
                            ((c2 >> indices_tlb) & three_int).gather(&GRAD_TABLE);
                        let y_grads_trb =
                            ((c2 >> indices_trb) & three_int).gather(&GRAD_TABLE);
                        let y_grads_blb =
                            ((c2 >> indices_blb) & three_int).gather(&GRAD_TABLE);
                        let y_grads_brb =
                            ((c2 >> indices_brb) & three_int).gather(&GRAD_TABLE);
                        let z_grads_tlf =
                            ((c3 >> indices_tlf) & three_int).gather(&GRAD_TABLE);
                        let z_grads_trf =
                            ((c3 >> indices_trf) & three_int).gather(&GRAD_TABLE);
                        let z_grads_blf =
                            ((c3 >> indices_blf) & three_int).gather(&GRAD_TABLE);
                        let z_grads_brf =
                            ((c3 >> indices_brf) & three_int).gather(&GRAD_TABLE);
                        let z_grads_tlb =
                            ((c3 >> indices_tlb) & three_int).gather(&GRAD_TABLE);
                        let z_grads_trb =
                            ((c3 >> indices_trb) & three_int).gather(&GRAD_TABLE);
                        let z_grads_blb =
                            ((c3 >> indices_blb) & three_int).gather(&GRAD_TABLE);
                        let z_grads_brb =
                            ((c3 >> indices_brb) & three_int).gather(&GRAD_TABLE);
                        let prod_tlf =
                            x_grads_tlf.mul_add(x_dist_lo,
                                y_grads_tlf.mul_add(y_dist_lo, z_grads_tlf * z_dist_lo));
                        let prod_trf =
                            x_grads_trf.mul_add(x_dist_lo,
                                y_grads_trf.mul_add(y_dist_lo, z_grads_trf * z_dist_hi));
                        let prod_blf =
                            x_grads_blf.mul_add(x_dist_lo,
                                y_grads_blf.mul_add(y_dist_hi, z_grads_blf * z_dist_lo));
                        let prod_brf =
                            x_grads_brf.mul_add(x_dist_lo,
                                y_grads_brf.mul_add(y_dist_hi, z_grads_brf * z_dist_hi));
                        let prod_tlb =
                            x_grads_tlb.mul_add(x_dist_hi,
                                y_grads_tlb.mul_add(y_dist_lo, z_grads_tlb * z_dist_lo));
                        let prod_trb =
                            x_grads_trb.mul_add(x_dist_hi,
                                y_grads_trb.mul_add(y_dist_lo, z_grads_trb * z_dist_hi));
                        let prod_blb =
                            x_grads_blb.mul_add(x_dist_hi,
                                y_grads_blb.mul_add(y_dist_hi, z_grads_blb * z_dist_lo));
                        let prod_brb =
                            x_grads_brb.mul_add(x_dist_hi,
                                y_grads_brb.mul_add(y_dist_hi, z_grads_brb * z_dist_hi));
                        let lerp_tf = z_lerp.mul_add(prod_trf - prod_tlf, prod_tlf);
                        let lerp_bf = z_lerp.mul_add(prod_brf - prod_blf, prod_blf);
                        let lerp_tb = z_lerp.mul_add(prod_trb - prod_tlb, prod_tlb);
                        let lerp_bb = z_lerp.mul_add(prod_brb - prod_blb, prod_blb);
                        let lerp_front = y_lerp.mul_add(lerp_bf - lerp_tf, lerp_tf);
                        let lerp_back = y_lerp.mul_add(lerp_bb - lerp_tb, lerp_tb);
                        x_lerp.mul_add(lerp_back - lerp_front, lerp_front)
                    }
                }
            }
            pub mod grid_2d {
                use std::f32::consts::SQRT_2;
                use std::fmt;
                use std::mem::MaybeUninit;
                use std::ops::Range;
                use crate::api::grid::interface::GridNoiseParams;
                use crate::noise::combiners::{Combiner, CombinerState};
                use crate::noise::util::grid_data::{GridData, Lerp};
                use crate::noise::util::grid_helpers::{
                    Arena, ArenaBuffer, InterpolationConfig,
                    MaybeUninitSliceSimdExt, assume_init_slice, maybe_tail_load,
                    maybe_tail_store, pad_grid_size, validate_grid_size,
                    validate_state_size,
                };
                use crate::simd::Arch;
                use crate::simd::register::Simd;
                use crate::{GridGenerator, Perlin};
                pub const GRADIENTS_2D: [[f32; 2]; 8] =
                    [[SQRT_2, 0.0], [1.0, 1.0], [0.0, SQRT_2], [-1.0, 1.0],
                            [-SQRT_2, 0.0], [-1.0, -1.0], [0.0, -SQRT_2], [1.0, -1.0]];
                pub struct PerlinGradients2D<'a> {
                    pub tl: [&'a mut [MaybeUninit<f32>]; 2],
                    pub tr: [&'a mut [MaybeUninit<f32>]; 2],
                    pub bl: [&'a mut [MaybeUninit<f32>]; 2],
                    pub br: [&'a mut [MaybeUninit<f32>]; 2],
                }
                impl<'a> PerlinGradients2D<'a> {
                    #[inline(always)]
                    pub fn new(arena: &'a mut Arena, size: usize) -> Self {
                        Self {
                            tl: [arena.allocate(size), arena.allocate(size)],
                            tr: [arena.allocate(size), arena.allocate(size)],
                            bl: [arena.allocate(size), arena.allocate(size)],
                            br: [arena.allocate(size), arena.allocate(size)],
                        }
                    }
                    #[inline(always)]
                    pub fn swap_top_bottom(&mut self) {
                        std::mem::swap(&mut self.tl, &mut self.bl);
                        std::mem::swap(&mut self.tr, &mut self.br);
                    }
                }
                impl<'a> fmt::Debug for PerlinGradients2D<'a> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        unsafe {
                            f.debug_struct("GridData").field("tl.x",
                                                                &assume_init_slice(self.tl[0])).field("tr.x",
                                                            &assume_init_slice(self.tr[0])).field("bl.x",
                                                        &assume_init_slice(self.bl[0])).field("br.x",
                                                    &assume_init_slice(self.br[0])).field("tl.y",
                                                &assume_init_slice(self.tl[1])).field("tr.y",
                                            &assume_init_slice(self.tr[1])).field("bl.y",
                                        &assume_init_slice(self.bl[1])).field("br.y",
                                    &assume_init_slice(self.br[1])).finish()
                        }
                    }
                }
                const LERP: u8 = Lerp::Quintic as u8;
                impl GridGenerator<2> for Perlin {
                    fn sample_grid<A: Arch, C: Combiner, const INIT : bool,
                        const FINAL :
                        bool>(params: GridNoiseParams<2>, fractal_config: C::Config,
                        state: &mut [f32], dst: &mut [f32]) {
                        validate_grid_size(params.grid_size, dst.len());
                        validate_state_size::<C, _>(params.grid_size, state.len());
                        let padded_size = pad_grid_size::<A, _>(params.grid_size);
                        let required_cache =
                            padded_size[1] * 3 + padded_size[0] * 12;
                        let mut cache =
                            ArenaBuffer::<A>::with_capacity(required_cache);
                        let mut arena = Arena::with_cache(&mut cache);
                        let num_blocks = A::NUM_SIMD_REG / 8;
                        let bilerp_config =
                            InterpolationConfig::<A>::new(num_blocks,
                                params.grid_size[0]);
                        let mut sub_arena =
                            arena.allocate_arena(padded_size[0] * 3 +
                                    padded_size[1] * 3);
                        let mut grid_data =
                            GridData::new::<A,
                                    LERP>(&params, &mut sub_arena, &padded_size);
                        let grad_scratch = arena.allocate(padded_size[0]);
                        let mut gradients =
                            PerlinGradients2D::new(&mut arena, padded_size[0]);
                        grid_gradients_2d::<A>(&params, &mut grid_data,
                            grad_scratch, &mut gradients.tl, &mut gradients.tr, 0);
                        let mut y_cur_index = 0;
                        for y_it in 0..grid_data.num_loops[1] {
                            let y_next_index =
                                unsafe {
                                    grid_data.grid_indices[1].get_unchecked(y_it).assume_init()
                                        as usize
                                };
                            grid_gradients_2d::<A>(&params, &mut grid_data,
                                grad_scratch, &mut gradients.bl, &mut gradients.br,
                                y_it + 1);
                            let y_range = y_cur_index..y_next_index;
                            grid_dotted_bilerp::<A, C, INIT,
                                    FINAL>(&bilerp_config, &fractal_config, &grid_data,
                                &gradients, y_range, (state, dst));
                            gradients.swap_top_bottom();
                            y_cur_index = y_next_index;
                        }
                    }
                }
                #[inline(always)]
                pub(super) fn grid_gradients_2d<'a,
                    A: Arch>(params: &GridNoiseParams<2>,
                    grid_data: &mut GridData<2>,
                    grad_buffer: &mut [MaybeUninit<u32>],
                    left: &mut [&'a mut [MaybeUninit<f32>]; 2],
                    right: &mut [&'a mut [MaybeUninit<f32>]; 2], y_it: usize) {
                    let lanes = Simd::<f32, A>::LANES;
                    let y_start = grid_data.grid_start[1] + y_it as i32;
                    let y_rem =
                        grid_data.octave_tiling[1].map_or(y_start,
                            |t| y_start.rem_euclid(t as i32));
                    let y_vec =
                        Simd::<u32,
                                A>::splat((y_rem as u32).wrapping_mul(params.seed));
                    let prime = Simd::<u32, A>::splat(0x85ebca6b_u32);
                    const BYTE_SHUFFLE: [u8; 64] =
                        [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                    let shuffle_indices =
                        unsafe {
                            Simd::<u8, A>::from_slice_unchecked(&BYTE_SHUFFLE[..])
                        };
                    let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;
                    if let Some(x_tiling) = grid_data.octave_tiling[0] {
                        let x_tiling = Simd::<f32, A>::splat(x_tiling as f32);
                        let mut x_vec =
                            Simd::<_, A>::splat(grid_data.grid_start[0]) +
                                Simd::<_, A>::iota(0);
                        let x_vec_stride = Simd::<_, A>::splat(lanes as i32);
                        let seed_vec = Simd::<_, A>::splat(params.seed);
                        let end_index = grid_data.num_loops[0] + 1;
                        for i in (0..end_index).step_by(lanes) {
                            let x_floats = x_vec.cast_float();
                            let x_rem =
                                x_floats - (x_floats / x_tiling).floor() * x_tiling;
                            let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;
                            let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
                            let indices: Simd<u32, A> = (y_shuf * x_shuf) >> 29;
                            unsafe { grad_buffer.write_simd_aligned(i, indices) };
                            x_vec += x_vec_stride;
                        }
                    } else {
                        let iota_vec =
                            Simd::<u32, A>::iota(0) *
                                Simd::<u32, A>::splat(params.seed);
                        let mut x_vec =
                            Simd::<u32,
                                        A>::splat((grid_data.grid_start[0] as
                                                u32).wrapping_mul(params.seed)) + iota_vec;
                        let x_vec_stride =
                            Simd::<u32,
                                    A>::splat((lanes as u32).wrapping_mul(params.seed));
                        let end_index = grid_data.num_loops[0] + 1;
                        for i in (0..end_index).step_by(lanes) {
                            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
                            let indices: Simd<u32, A> = (y_shuf * x_shuf) >> 29;
                            unsafe { grad_buffer.write_simd_aligned(i, indices) };
                            x_vec += x_vec_stride;
                        }
                    }
                    let mut x_cur_index = 0;
                    for x_it in 0..grid_data.num_loops[0] {
                        let x_next_index =
                            unsafe {
                                grid_data.grid_indices[0].get_unchecked(x_it).assume_init()
                            };
                        let mut amount = (x_next_index - x_cur_index) as isize;
                        unsafe {
                            let l =
                                grad_buffer.get_unchecked(x_it).assume_init() as usize;
                            let r =
                                grad_buffer.get_unchecked(x_it + 1).assume_init() as usize;
                            let ly =
                                Simd::<f32, A>::splat(GRADIENTS_2D.get_unchecked(l)[1]);
                            let lx =
                                Simd::<f32, A>::splat(GRADIENTS_2D.get_unchecked(l)[0]);
                            let ry =
                                Simd::<f32, A>::splat(GRADIENTS_2D.get_unchecked(r)[1]);
                            let rx =
                                Simd::<f32, A>::splat(GRADIENTS_2D.get_unchecked(r)[0]);
                            let mut index = x_cur_index as usize;
                            while amount > 0 {
                                left[1].write_simd(index, ly);
                                left[0].write_simd(index, lx);
                                right[1].write_simd(index, ry);
                                right[0].write_simd(index, rx);
                                amount -= lanes as isize;
                                index += lanes;
                            }
                        }
                        x_cur_index = x_next_index;
                    }
                    for i in (0..params.grid_size[0]).step_by(lanes) {
                        unsafe {
                            let cur_dist: Simd<f32, A> =
                                grid_data.distances[0].load_simd_aligned(i);
                            let cur_left = left[0].load_simd_aligned(i);
                            let cur_right = right[0].load_simd_aligned(i);
                            left[0].write_simd_aligned(i, cur_left * cur_dist);
                            right[0].write_simd_aligned(i,
                                cur_right.mul_sub(cur_dist, cur_right));
                        }
                    }
                }
                /// Handles interpolation execution state and fills
                /// the dst slice with interpolated values from gradient dot produtcts.
                pub(crate) struct DottedBilerpExecuter<'a, A: Arch,
                    C: Combiner, const INIT : bool, const FINAL : bool> {
                    config: &'a InterpolationConfig<A>,
                    fractal_config: &'a C::Config,
                    grid_data: &'a GridData<'a, 2>,
                    gradients: &'a PerlinGradients2D<'a>,
                    y_range: Range<usize>,
                    top: A::Block2<f32>,
                    dif: A::Block2<f32>,
                    d_top: A::Block2<f32>,
                    d_dif: A::Block2<f32>,
                    weight_vec: Simd<f32, A>,
                    y_weighted_increment: Simd<f32, A>,
                    y_upper_increment: Simd<f32, A>,
                    y_lower_increment: Simd<f32, A>,
                }
                /// Fills the dst slice with interpolated dot products from gradients.
                #[inline(always)]
                pub(super) fn grid_dotted_bilerp<A: Arch, C: Combiner, const
                    INIT : bool, const FINAL :
                    bool>(config: &InterpolationConfig<A>,
                    fractal_config: &C::Config, grid_data: &GridData<2>,
                    gradients: &PerlinGradients2D, y_range: Range<usize>,
                    output: (&mut [f32], &mut [f32])) {
                    let y_frac_start =
                        unsafe {
                            grid_data.distances[1].get_unchecked(y_range.start).assume_init()
                        };
                    let mut executer =
                        DottedBilerpExecuter::<A, C, INIT,
                            FINAL> {
                            config,
                            fractal_config,
                            grid_data,
                            gradients,
                            y_range,
                            top: Default::default(),
                            dif: Default::default(),
                            d_top: Default::default(),
                            d_dif: Default::default(),
                            weight_vec: Simd::splat(grid_data.weight),
                            y_weighted_increment: Simd::splat(grid_data.increment[1] *
                                    grid_data.weight),
                            y_upper_increment: Simd::splat(y_frac_start),
                            y_lower_increment: Simd::splat(y_frac_start - 1.0),
                        };
                    let (state, dst) = output;
                    if config.has_block_head {
                        executer.interpolate::<false>(state, dst);
                    }
                    if config.has_block_tail {
                        executer.interpolate::<true>(state, dst);
                        std::hint::cold_path();
                    }
                }
                impl<'a, A: Arch, C: Combiner, const INIT : bool, const FINAL
                    : bool> DottedBilerpExecuter<'a, A, C, INIT, FINAL> {
                    #[inline(always)]
                    pub fn interpolate<const IS_TAIL :
                        bool>(&mut self, state: &mut [f32], dst: &mut [f32]) {
                        let range =
                            if IS_TAIL {
                                self.config.block_tail_start..self.grid_data.grid_size[0]
                            } else { 0..self.config.block_tail_start };
                        for x in range.step_by(self.config.block_lanes) {
                            self.initialize_factors::<IS_TAIL>(x);
                            let mut y = self.y_range.start;
                            while y < self.y_range.end {
                                if y + 4 > self.y_range.end {
                                    self.process_factors::<IS_TAIL>(x, y, state, dst);
                                    y += 1;
                                } else {
                                    self.process_factors::<IS_TAIL>(x, y, state, dst);
                                    self.process_factors::<IS_TAIL>(x, y + 1, state, dst);
                                    self.process_factors::<IS_TAIL>(x, y + 2, state, dst);
                                    self.process_factors::<IS_TAIL>(x, y + 3, state, dst);
                                    y += 4;
                                }
                            }
                        }
                    }
                    #[inline(always)]
                    fn initialize_factors<const IS_TAIL :
                        bool>(&mut self, x: usize) {
                        let num_blocks =
                            if IS_TAIL {
                                self.config.block_tail_size
                            } else { self.config.num_blocks };
                        for block in 0..num_blocks {
                            let index = x + Simd::<f32, A>::LANES * block;
                            let x_lerp =
                                unsafe {
                                    self.grid_data.fade_factors[0].load_simd_aligned(index)
                                };
                            let x_tl =
                                unsafe { self.gradients.tl[0].load_simd_aligned(index) };
                            let x_tr =
                                unsafe { self.gradients.tr[0].load_simd_aligned(index) };
                            let x_bl =
                                unsafe { self.gradients.bl[0].load_simd_aligned(index) };
                            let x_br =
                                unsafe { self.gradients.br[0].load_simd_aligned(index) };
                            let y_tl =
                                unsafe { self.gradients.tl[1].load_simd_aligned(index) };
                            let y_tr =
                                unsafe { self.gradients.tr[1].load_simd_aligned(index) };
                            let y_bl =
                                unsafe { self.gradients.bl[1].load_simd_aligned(index) };
                            let y_br =
                                unsafe { self.gradients.br[1].load_simd_aligned(index) };
                            let prod_sum_tl =
                                y_tl.mul_add(self.y_upper_increment, x_tl);
                            let prod_sum_tr =
                                y_tr.mul_add(self.y_upper_increment, x_tr);
                            let prod_sum_bl =
                                y_bl.mul_add(self.y_lower_increment, x_bl);
                            let prod_sum_br =
                                y_br.mul_add(self.y_lower_increment, x_br);
                            let prod_sum_top_dif = prod_sum_tr - prod_sum_tl;
                            let prod_sum_low_dif = prod_sum_br - prod_sum_bl;
                            self.top[block] =
                                x_lerp.mul_add(prod_sum_top_dif, prod_sum_tl) *
                                    self.weight_vec;
                            let base_lerp_bottom =
                                x_lerp.mul_add(prod_sum_low_dif, prod_sum_bl) *
                                    self.weight_vec;
                            self.dif[block] = base_lerp_bottom - self.top[block];
                            self.d_top[block] =
                                x_lerp.mul_add(y_tr - y_tl, y_tl) *
                                    self.y_weighted_increment;
                            let y_offset_lerp_bottom =
                                x_lerp.mul_add(y_br - y_bl, y_bl) *
                                    self.y_weighted_increment;
                            self.d_dif[block] =
                                y_offset_lerp_bottom - self.d_top[block];
                        }
                    }
                    #[inline(always)]
                    fn process_factors<const IS_TAIL :
                        bool>(&mut self, x: usize, y: usize, state: &mut [f32],
                        dst: &mut [f32]) {
                        let y_lerp =
                            Simd::splat(unsafe {
                                    self.grid_data.fade_factors[1].get_unchecked(y).assume_init()
                                });
                        let range =
                            if IS_TAIL {
                                0..self.config.block_tail_size
                            } else { 0..self.config.num_blocks };
                        let index = y * self.grid_data.grid_size[0] + x;
                        let tail_end = index + self.config.tail_size;
                        for block in range {
                            let index = index + block * Simd::<f32, A>::LANES;
                            let output =
                                y_lerp.mul_add(self.dif[block], self.top[block]);
                            let (cur_state, mut result) =
                                if INIT {
                                    C::initialize_sample(self.fractal_config, output)
                                } else {
                                    let mut cur_state = C::State::<A>::default();
                                    for i in 0..C::State::STATE_SIZE {
                                        let offset = i * self.grid_data.total_size;
                                        let index = index + offset;
                                        let tail_end = tail_end + offset;
                                        cur_state[i] =
                                            unsafe {
                                                maybe_tail_load::<IS_TAIL>(index..tail_end, state)
                                            };
                                    }
                                    let cur_result =
                                        unsafe { maybe_tail_load::<IS_TAIL>(index..tail_end, dst) };
                                    C::apply_sample(self.fractal_config, cur_state, cur_result,
                                        output)
                                };
                            if !FINAL {
                                for i in 0..C::State::STATE_SIZE {
                                    let offset = i * self.grid_data.total_size;
                                    let index = index + offset;
                                    let tail_end = tail_end + offset;
                                    unsafe {
                                        maybe_tail_store::<IS_TAIL>(index..tail_end, cur_state[i],
                                            state)
                                    };
                                }
                            }
                            if FINAL {
                                result =
                                    C::finalize_sample(self.fractal_config, cur_state, result);
                            }
                            unsafe {
                                maybe_tail_store::<IS_TAIL>(index..tail_end, result, dst)
                            };
                            self.dif[block] += self.d_dif[block];
                            self.top[block] += self.d_top[block];
                        }
                    }
                }
            }
            pub mod grid_3d {
                use std::array::from_fn;
                use std::fmt;
                use std::mem::MaybeUninit;
                use std::ops::Range;
                use crate::api::grid::interface::GridNoiseParams;
                use crate::noise::combiners::{Combiner, CombinerState};
                use crate::noise::util::grid_data::{GridData, Lerp};
                use crate::noise::util::grid_helpers::{
                    Arena, ArenaBuffer, InterpolationConfig,
                    MaybeUninitSliceSimdExt, assume_init_slice, maybe_tail_load,
                    maybe_tail_store, pad_grid_size, validate_grid_size,
                    validate_state_size,
                };
                use crate::simd::{Arch, Simd};
                use crate::{GridGenerator, Perlin};
                pub const GRADIENTS_3D: [[f32; 3]; 16] =
                    [[1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0],
                            [-1.0, -1.0, 0.0], [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0],
                            [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0], [0.0, 1.0, 1.0],
                            [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, -1.0, -1.0],
                            [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [0.0, -1.0, 1.0],
                            [0.0, -1.0, -1.0]];
                pub struct PerlinGradients3D<'a> {
                    pub tlf: [&'a mut [MaybeUninit<f32>]; 3],
                    pub trf: [&'a mut [MaybeUninit<f32>]; 3],
                    pub blf: [&'a mut [MaybeUninit<f32>]; 3],
                    pub brf: [&'a mut [MaybeUninit<f32>]; 3],
                    pub tlb: [&'a mut [MaybeUninit<f32>]; 3],
                    pub trb: [&'a mut [MaybeUninit<f32>]; 3],
                    pub blb: [&'a mut [MaybeUninit<f32>]; 3],
                    pub brb: [&'a mut [MaybeUninit<f32>]; 3],
                    pub scratch: [&'a mut [MaybeUninit<u32>]; 2],
                }
                impl<'a> PerlinGradients3D<'a> {
                    #[inline(always)]
                    pub fn new(arena: &'a mut Arena, size: usize) -> Self {
                        Self {
                            tlf: from_fn(|_| arena.allocate(size)),
                            trf: from_fn(|_| arena.allocate(size)),
                            blf: from_fn(|_| arena.allocate(size)),
                            brf: from_fn(|_| arena.allocate(size)),
                            tlb: from_fn(|_| arena.allocate(size)),
                            trb: from_fn(|_| arena.allocate(size)),
                            blb: from_fn(|_| arena.allocate(size)),
                            brb: from_fn(|_| arena.allocate(size)),
                            scratch: from_fn(|_| arena.allocate(size)),
                        }
                    }
                    #[inline(always)]
                    pub fn swap_top_bottom(&mut self) {
                        std::mem::swap(&mut self.tlf, &mut self.blf);
                        std::mem::swap(&mut self.trf, &mut self.brf);
                        std::mem::swap(&mut self.tlb, &mut self.blb);
                        std::mem::swap(&mut self.trb, &mut self.brb);
                    }
                }
                impl<'a> fmt::Debug for PerlinGradients3D<'a> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        unsafe {
                            f.debug_struct("PerlinGradients3D").field("tl.x",
                                                                &assume_init_slice(self.tlf[0])).field("tr.x",
                                                            &assume_init_slice(self.trf[0])).field("bl.x",
                                                        &assume_init_slice(self.blf[0])).field("br.x",
                                                    &assume_init_slice(self.brf[0])).field("tl.y",
                                                &assume_init_slice(self.tlf[1])).field("tr.y",
                                            &assume_init_slice(self.trf[1])).field("bl.y",
                                        &assume_init_slice(self.blf[1])).field("br.y",
                                    &assume_init_slice(self.brf[1])).finish()
                        }
                    }
                }
                const LERP: u8 = Lerp::Quintic as u8;
                impl GridGenerator<3> for Perlin {
                    fn sample_grid<A: Arch, C: Combiner, const INIT : bool,
                        const FINAL :
                        bool>(params: GridNoiseParams<3>, fractal_config: C::Config,
                        state: &mut [f32], dst: &mut [f32]) {
                        validate_grid_size(params.grid_size, dst.len());
                        validate_state_size::<C, _>(params.grid_size, state.len());
                        let padded_size = pad_grid_size(params.grid_size);
                        let required_cache =
                            padded_size[0] * 41 + padded_size[1] * 3 +
                                padded_size[2] * 3;
                        let mut cache = ArenaBuffer::with_capacity(required_cache);
                        let mut arena = Arena::with_cache(&mut cache);
                        let mut data_arena =
                            arena.allocate_arena(padded_size.iter().fold(0,
                                    |n, x| n + 3 * x));
                        let mut trilerp_arena =
                            arena.allocate_arena(padded_size[0] * 12);
                        let num_blocks = A::NUM_SIMD_REG / 8;
                        let bilerp_config =
                            InterpolationConfig::new(num_blocks, params.grid_size[0]);
                        let grid_data =
                            GridData::new::<LERP>(&params, &mut data_arena,
                                &padded_size);
                        let mut trilerp_buffers =
                            DottedTrilerpBuffers::new(&mut trilerp_arena,
                                padded_size[0]);
                        let mut gradients =
                            PerlinGradients3D::new(&mut arena, padded_size[0]);
                        let mut z_cur_index = 0;
                        for z_it in 0..grid_data.num_loops[2] {
                            let z_next_index =
                                unsafe {
                                    grid_data.grid_indices[2].get_unchecked(z_it).assume_init()
                                        as usize
                                };
                            let z_range = z_cur_index..z_next_index;
                            grid_gradients_3d(&params, &grid_data, &mut gradients, 0,
                                z_it);
                            gradients.swap_top_bottom();
                            let mut y_cur_index = 0;
                            for y_it in 0..grid_data.num_loops[1] {
                                let y_next_index =
                                    unsafe {
                                        grid_data.grid_indices[1].get_unchecked(y_it).assume_init()
                                            as usize
                                    };
                                let y_range = y_cur_index..y_next_index;
                                grid_gradients_3d(&params, &grid_data, &mut gradients,
                                    y_it + 1, z_it);
                                grid_dotted_trilerp::<C, INIT,
                                        FINAL>(&mut trilerp_buffers, &bilerp_config,
                                    &fractal_config, &grid_data, &gradients,
                                    (y_range, z_range.clone()), (state, dst));
                                gradients.swap_top_bottom();
                                y_cur_index = y_next_index;
                            }
                            z_cur_index = z_next_index;
                        }
                    }
                }
                #[inline(always)]
                pub(super) fn grid_gradients_3d<'a,
                    A: Arch>(params: &GridNoiseParams<3>,
                    grid_data: &GridData<3>,
                    gradients: &mut PerlinGradients3D<'a>, y_it: usize,
                    z_it: usize) {
                    let lanes = Simd::<f32, A>::LANES;
                    let y_start = y_it as i32 + grid_data.grid_start[1];
                    let z_start = z_it as i32 + grid_data.grid_start[2];
                    let (z1, z2) =
                        match grid_data.octave_tiling[2] {
                            None =>
                                ((z_start as u32).wrapping_mul(params.seed),
                                    (z_start as
                                                    u32).wrapping_mul(params.seed).wrapping_add(params.seed)),
                            Some(t) =>
                                ((z_start.rem_euclid(t as i32)) as u32,
                                    ((z_start + 1).rem_euclid(t as i32)) as u32),
                        };
                    let z_vec = [Simd::splat(z1), Simd::splat(z2)];
                    let y_rem =
                        grid_data.octave_tiling[1].map_or(y_start,
                            |t| y_start.rem_euclid(t as i32));
                    let y_vec =
                        Simd::splat((y_rem as u32).wrapping_mul(params.seed));
                    const BYTE_SHUFFLE: [u8; 64] =
                        [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                    let shuffle_indices =
                        Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                    let prime = Simd::splat(0x85ebca6b_u32);
                    let z_shuf: [_; 2] =
                        from_fn(|i| z_vec[i].permute_8(shuffle_indices) ^ prime);
                    let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;
                    let zy_mix: [_; 2] = from_fn(|i| z_shuf[i] * y_shuf);
                    let end_index = grid_data.num_loops[0] + 1;
                    if let Some(x_tiling) = grid_data.octave_tiling[0] {
                        let x_tiling = Simd::splat(x_tiling as f32);
                        let mut x_vec =
                            Simd::splat(grid_data.grid_start[0]) + Simd::iota(0);
                        let x_vec_stride = Simd::splat(lanes as i32);
                        let seed_vec = Simd::splat(params.seed);
                        for i in (0..end_index).step_by(lanes) {
                            let x_floats = x_vec.cast_float();
                            let x_rem =
                                x_floats - (x_floats / x_tiling).floor() * x_tiling;
                            let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;
                            let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
                            let grads: [_; 2] = from_fn(|i| (zy_mix[i] * x_shuf) >> 28);
                            unsafe {
                                gradients.scratch[0].write_simd_aligned(i, grads[0]);
                                gradients.scratch[1].write_simd_aligned(i, grads[1]);
                            };
                            x_vec += x_vec_stride;
                        }
                    } else {
                        let iota_vec = Simd::iota(0) * Simd::splat(params.seed);
                        let x_start_seeded =
                            (grid_data.grid_start[0] as u32).wrapping_mul(params.seed);
                        let mut x_vec = Simd::splat(x_start_seeded) + iota_vec;
                        let x_vec_stride =
                            Simd::splat((lanes as u32).wrapping_mul(params.seed));
                        for i in (0..end_index).step_by(lanes) {
                            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
                            let grads: [_; 2] = from_fn(|i| (zy_mix[i] * x_shuf) >> 28);
                            unsafe {
                                gradients.scratch[0].write_simd_aligned(i, grads[0]);
                                gradients.scratch[1].write_simd_aligned(i, grads[1]);
                            };
                            x_vec += x_vec_stride;
                        }
                    }
                    grid_gradients_3d_set_loop::<true>(grid_data, gradients);
                    grid_gradients_3d_set_loop::<false>(grid_data, gradients);
                    for i in (0..params.grid_size[0]).step_by(lanes) {
                        unsafe {
                            let cur_dist = grid_data.distances[0].load_simd_aligned(i);
                            let lf = gradients.blf[0].load_simd_aligned(i);
                            let rf = gradients.brf[0].load_simd_aligned(i);
                            let lb = gradients.blb[0].load_simd_aligned(i);
                            let rb = gradients.brb[0].load_simd_aligned(i);
                            gradients.blf[0].write_simd_aligned(i, lf * cur_dist);
                            gradients.brf[0].write_simd_aligned(i,
                                rf.mul_sub(cur_dist, rf));
                            gradients.blb[0].write_simd_aligned(i, lb * cur_dist);
                            gradients.brb[0].write_simd_aligned(i,
                                rb.mul_sub(cur_dist, rb));
                        }
                    }
                }
                #[inline(always)]
                pub(super) fn grid_gradients_3d_set_loop<'a, A: Arch, const
                    IS_FRONT :
                    bool>(grid_data: &GridData<3>,
                    gradients: &mut PerlinGradients3D<'a>) {
                    let (grad_buffer, left, right) =
                        if IS_FRONT {
                            (&mut gradients.scratch[0], &mut gradients.blf,
                                &mut gradients.brf)
                        } else {
                            (&mut gradients.scratch[1], &mut gradients.blb,
                                &mut gradients.brb)
                        };
                    let mut x_cur_index = 0;
                    for x_it in 0..grid_data.num_loops[0] {
                        let x_next_index =
                            unsafe {
                                grid_data.grid_indices[0].get_unchecked(x_it).assume_init()
                            };
                        let mut amount = (x_next_index - x_cur_index) as isize;
                        unsafe {
                            let l =
                                grad_buffer.get_unchecked(x_it).assume_init() as usize;
                            let r =
                                grad_buffer.get_unchecked(x_it + 1).assume_init() as usize;
                            let l = GRADIENTS_3D.get_unchecked(l);
                            let r = GRADIENTS_3D.get_unchecked(r);
                            let lx = Simd::splat(l[0]);
                            let ly = Simd::splat(l[1]);
                            let lz = Simd::splat(l[2]);
                            let rx = Simd::splat(r[0]);
                            let ry = Simd::splat(r[1]);
                            let rz = Simd::splat(r[2]);
                            let mut index = x_cur_index as usize;
                            while amount > 0 {
                                left[0].write_simd(index, lx);
                                left[1].write_simd(index, ly);
                                left[2].write_simd(index, lz);
                                right[0].write_simd(index, rx);
                                right[1].write_simd(index, ry);
                                right[2].write_simd(index, rz);
                                amount -= Simd::<f32, A>::LANES as isize;
                                index += Simd::<f32, A>::LANES;
                            }
                        }
                        x_cur_index = x_next_index;
                    }
                }
                pub(crate) struct DottedTrilerpBuffers<'a> {
                    y_tf_offset: &'a mut [MaybeUninit<f32>],
                    y_bf_offset: &'a mut [MaybeUninit<f32>],
                    y_top_offset_dif: &'a mut [MaybeUninit<f32>],
                    y_bottom_offset_dif: &'a mut [MaybeUninit<f32>],
                    z_tf_offset: &'a mut [MaybeUninit<f32>],
                    z_bf_offset: &'a mut [MaybeUninit<f32>],
                    z_top_offset_dif: &'a mut [MaybeUninit<f32>],
                    z_bottom_offset_dif: &'a mut [MaybeUninit<f32>],
                    tf_base: &'a mut [MaybeUninit<f32>],
                    bf_base: &'a mut [MaybeUninit<f32>],
                    top_base_dif: &'a mut [MaybeUninit<f32>],
                    bottom_base_dif: &'a mut [MaybeUninit<f32>],
                }
                impl<'a> DottedTrilerpBuffers<'a> {
                    #[inline(always)]
                    pub fn new(arena: &'a mut Arena, x_size: usize) -> Self {
                        Self {
                            y_tf_offset: arena.allocate(x_size),
                            y_bf_offset: arena.allocate(x_size),
                            y_top_offset_dif: arena.allocate(x_size),
                            y_bottom_offset_dif: arena.allocate(x_size),
                            z_tf_offset: arena.allocate(x_size),
                            z_bf_offset: arena.allocate(x_size),
                            z_top_offset_dif: arena.allocate(x_size),
                            z_bottom_offset_dif: arena.allocate(x_size),
                            tf_base: arena.allocate(x_size),
                            bf_base: arena.allocate(x_size),
                            top_base_dif: arena.allocate(x_size),
                            bottom_base_dif: arena.allocate(x_size),
                        }
                    }
                }
                /// Handles interpolation execution state and fills
                /// the dst slice with interpolated values from gradient dot produtcts.
                pub(crate) struct DottedTrilerpExecuter<'a, A: Arch,
                    C: Combiner, const INIT : bool, const FINAL : bool> {
                    config: &'a InterpolationConfig<A>,
                    fractal_config: &'a C::Config,
                    grid_data: &'a GridData<'a, 3>,
                    gradients: &'a PerlinGradients3D<'a>,
                    y_range: Range<usize>,
                    z_range: Range<usize>,
                    top: A::Block4<f32>,
                    dif: A::Block4<f32>,
                    d_top: A::Block4<f32>,
                    d_dif: A::Block4<f32>,
                    weight: Simd<f32, A>,
                    y_inc_weighted: Simd<f32, A>,
                    y_inc_hi: Simd<f32, A>,
                    y_inc_lo: Simd<f32, A>,
                    z_inc_weighted: Simd<f32, A>,
                    z_inc_hi: Simd<f32, A>,
                    z_inc_lo: Simd<f32, A>,
                }
                /// Fills the dst slice with interpolated dot products from gradients.
                #[inline(always)]
                pub(super) fn grid_dotted_trilerp<A: Arch, C: Combiner, const
                    INIT : bool, const FINAL :
                    bool>(buffers: &mut DottedTrilerpBuffers,
                    config: &InterpolationConfig<A>, fractal_config: &C::Config,
                    grid_data: &GridData<3>, gradients: &PerlinGradients3D,
                    ranges: (Range<usize>, Range<usize>),
                    output: (&mut [f32], &mut [f32])) {
                    let y_frac_start =
                        unsafe {
                            grid_data.distances[1].get_unchecked(ranges.0.start).assume_init()
                        };
                    let z_frac_start =
                        unsafe {
                            grid_data.distances[2].get_unchecked(ranges.1.start).assume_init()
                        };
                    let mut executer =
                        DottedTrilerpExecuter::<A, C, INIT,
                            FINAL> {
                            config,
                            fractal_config,
                            grid_data,
                            gradients,
                            y_range: ranges.0,
                            z_range: ranges.1,
                            top: Default::default(),
                            dif: Default::default(),
                            d_top: Default::default(),
                            d_dif: Default::default(),
                            weight: Simd::splat(grid_data.weight),
                            y_inc_weighted: Simd::splat(grid_data.increment[1] *
                                    grid_data.weight),
                            y_inc_hi: Simd::splat(y_frac_start),
                            y_inc_lo: Simd::splat(y_frac_start - 1.0),
                            z_inc_weighted: Simd::splat(grid_data.increment[2] *
                                    grid_data.weight),
                            z_inc_hi: Simd::splat(z_frac_start),
                            z_inc_lo: Simd::splat(z_frac_start - 1.0),
                        };
                    executer.initialize_trilerp_buffers(buffers);
                    let (state, dst) = output;
                    if config.has_block_head {
                        executer.interpolate::<false>(buffers, state, dst);
                    }
                    if config.has_block_tail {
                        executer.interpolate::<true>(buffers, state, dst);
                        std::hint::cold_path();
                    }
                }
                impl<'a, A: Arch, C: Combiner, const INIT : bool, const FINAL
                    : bool> DottedTrilerpExecuter<'a, A, C, INIT, FINAL> {
                    #[inline(always)]
                    pub fn interpolate<const IS_TAIL :
                        bool>(&mut self, buffers: &DottedTrilerpBuffers,
                        state: &mut [f32], dst: &mut [f32]) {
                        let range =
                            if IS_TAIL {
                                self.config.block_tail_start..self.grid_data.grid_size[0]
                            } else { 0..self.config.block_tail_start };
                        let mut z_cur = Simd::splat(0.0);
                        let z_hop =
                            self.grid_data.grid_size[0] * self.grid_data.grid_size[1];
                        let y_hop = self.grid_data.grid_size[0];
                        for z in self.z_range.start..self.z_range.end {
                            let z_lerp =
                                unsafe { self.grid_data.fade_factors[2].get_unchecked(z) };
                            let z_lerp = unsafe { z_lerp.assume_init() };
                            let z_lerp = Simd::splat(z_lerp);
                            for x in range.clone().step_by(self.config.block_lanes) {
                                self.intialize_factors::<IS_TAIL>(buffers, x, z_cur,
                                    z_lerp);
                                let index = z * z_hop + x;
                                let mut y = self.y_range.start;
                                while y < self.y_range.end {
                                    let index = index + y * y_hop;
                                    if y + 4 > self.y_range.end {
                                        self.process_factors::<IS_TAIL>(index, y, state, dst);
                                        y += 1;
                                    } else {
                                        self.process_factors::<IS_TAIL>(index, y, state, dst);
                                        self.process_factors::<IS_TAIL>(index + y_hop, y + 1, state,
                                            dst);
                                        self.process_factors::<IS_TAIL>(index + 2 * y_hop, y + 2,
                                            state, dst);
                                        self.process_factors::<IS_TAIL>(index + 3 * y_hop, y + 3,
                                            state, dst);
                                        y += 4;
                                    }
                                }
                            }
                            z_cur += Simd::splat(1.0);
                        }
                    }
                    #[inline(always)]
                    fn initialize_trilerp_buffers(&mut self,
                        buffers: &mut DottedTrilerpBuffers) {
                        for x in
                            (0..self.grid_data.grid_size[0]).step_by(Simd::<f32,
                                    A>::LANES) {
                            unsafe {
                                let x_lerp =
                                    self.grid_data.fade_factors[0].load_simd_aligned(x);
                                let x_tlf = self.gradients.tlf[0].load_simd_aligned(x);
                                let x_trf = self.gradients.trf[0].load_simd_aligned(x);
                                let x_blf = self.gradients.blf[0].load_simd_aligned(x);
                                let x_brf = self.gradients.brf[0].load_simd_aligned(x);
                                let x_tlb = self.gradients.tlb[0].load_simd_aligned(x);
                                let x_trb = self.gradients.trb[0].load_simd_aligned(x);
                                let x_blb = self.gradients.blb[0].load_simd_aligned(x);
                                let x_brb = self.gradients.brb[0].load_simd_aligned(x);
                                let y_tlf = self.gradients.tlf[1].load_simd_aligned(x);
                                let y_trf = self.gradients.trf[1].load_simd_aligned(x);
                                let y_blf = self.gradients.blf[1].load_simd_aligned(x);
                                let y_brf = self.gradients.brf[1].load_simd_aligned(x);
                                let y_tlb = self.gradients.tlb[1].load_simd_aligned(x);
                                let y_trb = self.gradients.trb[1].load_simd_aligned(x);
                                let y_blb = self.gradients.blb[1].load_simd_aligned(x);
                                let y_brb = self.gradients.brb[1].load_simd_aligned(x);
                                let z_tlf = self.gradients.tlf[2].load_simd_aligned(x);
                                let z_trf = self.gradients.trf[2].load_simd_aligned(x);
                                let z_blf = self.gradients.blf[2].load_simd_aligned(x);
                                let z_brf = self.gradients.brf[2].load_simd_aligned(x);
                                let z_tlb = self.gradients.tlb[2].load_simd_aligned(x);
                                let z_trb = self.gradients.trb[2].load_simd_aligned(x);
                                let z_blb = self.gradients.blb[2].load_simd_aligned(x);
                                let z_brb = self.gradients.brb[2].load_simd_aligned(x);
                                let calc_prod_sum =
                                    |z_inc: Simd<f32>, y_inc: Simd<f32>, z, y, x|
                                        { z_inc.mul_add(z, y_inc.mul_add(y, x)) };
                                let sum_prod_tlf =
                                    calc_prod_sum(self.z_inc_hi, self.y_inc_hi, z_tlf, y_tlf,
                                        x_tlf);
                                let sum_prod_trf =
                                    calc_prod_sum(self.z_inc_hi, self.y_inc_hi, z_trf, y_trf,
                                        x_trf);
                                let sum_prod_blf =
                                    calc_prod_sum(self.z_inc_hi, self.y_inc_lo, z_blf, y_blf,
                                        x_blf);
                                let sum_prod_brf =
                                    calc_prod_sum(self.z_inc_hi, self.y_inc_lo, z_brf, y_brf,
                                        x_brf);
                                let sum_prod_tlb =
                                    calc_prod_sum(self.z_inc_lo, self.y_inc_hi, z_tlb, y_tlb,
                                        x_tlb);
                                let sum_prod_trb =
                                    calc_prod_sum(self.z_inc_lo, self.y_inc_hi, z_trb, y_trb,
                                        x_trb);
                                let sum_prod_blb =
                                    calc_prod_sum(self.z_inc_lo, self.y_inc_lo, z_blb, y_blb,
                                        x_blb);
                                let sum_prod_brb =
                                    calc_prod_sum(self.z_inc_lo, self.y_inc_lo, z_brb, y_brb,
                                        x_brb);
                                let z_tf_offset =
                                    x_lerp.mul_add(z_trf - z_tlf, z_tlf) * self.z_inc_weighted;
                                let z_bf_offset =
                                    x_lerp.mul_add(z_brf - z_blf, z_blf) * self.z_inc_weighted;
                                let z_tb_offset =
                                    x_lerp.mul_add(z_trb - z_tlb, z_tlb) * self.z_inc_weighted;
                                let z_bb_offset =
                                    x_lerp.mul_add(z_brb - z_blb, z_blb) * self.z_inc_weighted;
                                let y_tf_offset =
                                    x_lerp.mul_add(y_trf - y_tlf, y_tlf) * self.y_inc_weighted;
                                let y_bf_offset =
                                    x_lerp.mul_add(y_brf - y_blf, y_blf) * self.y_inc_weighted;
                                let y_hi_offset_dif =
                                    x_lerp.mul_add(y_trb - y_tlb,
                                            y_tlb).mul_sub(self.y_inc_weighted, y_tf_offset);
                                let y_lo_offset_dif =
                                    x_lerp.mul_add(y_brb - y_blb,
                                            y_blb).mul_sub(self.y_inc_weighted, y_bf_offset);
                                let tf_base =
                                    x_lerp.mul_add(sum_prod_trf - sum_prod_tlf, sum_prod_tlf) *
                                        self.weight;
                                let bf_base =
                                    x_lerp.mul_add(sum_prod_brf - sum_prod_blf, sum_prod_blf) *
                                        self.weight;
                                let hi_base_dif =
                                    x_lerp.mul_add(sum_prod_trb - sum_prod_tlb,
                                            sum_prod_tlb).mul_sub(self.weight, tf_base);
                                let lo_base_dif =
                                    x_lerp.mul_add(sum_prod_brb - sum_prod_blb,
                                            sum_prod_blb).mul_sub(self.weight, bf_base);
                                buffers.z_tf_offset.write_simd_aligned(x, z_tf_offset);
                                buffers.z_bf_offset.write_simd_aligned(x, z_bf_offset);
                                buffers.z_top_offset_dif.write_simd_aligned(x,
                                    z_tb_offset - z_tf_offset);
                                buffers.z_bottom_offset_dif.write_simd_aligned(x,
                                    z_bb_offset - z_bf_offset);
                                buffers.y_tf_offset.write_simd_aligned(x, y_tf_offset);
                                buffers.y_bf_offset.write_simd_aligned(x, y_bf_offset);
                                buffers.y_top_offset_dif.write_simd_aligned(x,
                                    y_hi_offset_dif);
                                buffers.y_bottom_offset_dif.write_simd_aligned(x,
                                    y_lo_offset_dif);
                                buffers.tf_base.write_simd_aligned(x, tf_base);
                                buffers.bf_base.write_simd_aligned(x, bf_base);
                                buffers.top_base_dif.write_simd_aligned(x, hi_base_dif);
                                buffers.bottom_base_dif.write_simd_aligned(x, lo_base_dif);
                            }
                        }
                    }
                    #[inline(always)]
                    fn intialize_factors<const IS_TAIL :
                        bool>(&mut self, buffers: &DottedTrilerpBuffers, x: usize,
                        z_vec: Simd<f32, A>, z_lerp: Simd<f32, A>) {
                        let num_blocks =
                            if IS_TAIL {
                                self.config.block_tail_size
                            } else { self.config.num_blocks };
                        for block in 0..num_blocks {
                            unsafe {
                                let index = x + Simd::<f32, A>::LANES * block;
                                let z_tf_offset =
                                    buffers.z_tf_offset.load_simd_aligned(index);
                                let z_bf_offset =
                                    buffers.z_bf_offset.load_simd_aligned(index);
                                let z_top_offset_dif =
                                    buffers.z_top_offset_dif.load_simd_aligned(index);
                                let z_bottom_offset_dif =
                                    buffers.z_bottom_offset_dif.load_simd_aligned(index);
                                let y_tf_offset =
                                    buffers.y_tf_offset.load_simd_aligned(index);
                                let y_bf_offset =
                                    buffers.y_bf_offset.load_simd_aligned(index);
                                let y_top_offset_dif =
                                    buffers.y_top_offset_dif.load_simd_aligned(index);
                                let y_bottom_offset_dif =
                                    buffers.y_bottom_offset_dif.load_simd_aligned(index);
                                let tf_base_vec = buffers.tf_base.load_simd_aligned(index);
                                let bf_base_vec = buffers.bf_base.load_simd_aligned(index);
                                let top_base_dif_vec =
                                    buffers.top_base_dif.load_simd_aligned(index);
                                let bottom_base_dif_vec =
                                    buffers.bottom_base_dif.load_simd_aligned(index);
                                let z_top_offset =
                                    z_lerp.mul_add(z_top_offset_dif, z_tf_offset);
                                let z_bottom_offset =
                                    z_lerp.mul_add(z_bottom_offset_dif, z_bf_offset);
                                self.top[block] =
                                    z_vec.mul_add(z_top_offset,
                                        z_lerp.mul_add(top_base_dif_vec, tf_base_vec));
                                let bottom_base =
                                    z_vec.mul_add(z_bottom_offset,
                                        z_lerp.mul_add(bottom_base_dif_vec, bf_base_vec));
                                self.dif[block] = bottom_base - self.top[block];
                                self.d_top[block] =
                                    z_lerp.mul_add(y_top_offset_dif, y_tf_offset);
                                let y_bottom_offset =
                                    z_lerp.mul_add(y_bottom_offset_dif, y_bf_offset);
                                self.d_dif[block] = y_bottom_offset - self.d_top[block];
                            }
                        }
                    }
                    #[inline(always)]
                    fn process_factors<const IS_TAIL :
                        bool>(&mut self, index: usize, y: usize, state: &mut [f32],
                        dst: &mut [f32]) {
                        let y_lerp =
                            Simd::splat(unsafe {
                                    self.grid_data.fade_factors[1].get_unchecked(y).assume_init()
                                });
                        let range =
                            if IS_TAIL {
                                0..self.config.block_tail_size
                            } else { 0..self.config.num_blocks };
                        let tail_end = index + self.config.tail_size;
                        for block in range {
                            let index = index + block * Simd::<f32, A>::LANES;
                            let output =
                                y_lerp.mul_add(self.dif[block], self.top[block]);
                            let (cur_state, mut result) =
                                if INIT {
                                    C::initialize_sample(self.fractal_config, output)
                                } else {
                                    let mut cur_state = C::State::default();
                                    for i in 0..C::State::STATE_SIZE {
                                        let index = index + i * self.grid_data.total_size;
                                        cur_state[i] =
                                            unsafe {
                                                maybe_tail_load::<IS_TAIL>(index..tail_end, state)
                                            };
                                    }
                                    let cur_result =
                                        unsafe { maybe_tail_load::<IS_TAIL>(index..tail_end, dst) };
                                    C::apply_sample(self.fractal_config, cur_state, cur_result,
                                        output)
                                };
                            if !FINAL {
                                for i in 0..C::State::STATE_SIZE {
                                    let offset = i * self.grid_data.total_size;
                                    let index = index + offset;
                                    let tail_end = tail_end + offset;
                                    unsafe {
                                        maybe_tail_store::<IS_TAIL>(index..tail_end, cur_state[i],
                                            state)
                                    };
                                }
                            }
                            if FINAL {
                                result =
                                    C::finalize_sample(self.fractal_config, cur_state, result);
                            }
                            unsafe {
                                maybe_tail_store::<IS_TAIL>(index..tail_end, result, dst)
                            };
                            self.dif[block] += self.d_dif[block];
                            self.top[block] += self.d_top[block];
                        }
                    }
                }
            }
        }
        pub struct Value {}
        #[automatically_derived]
        impl ::core::default::Default for Value {
            #[inline]
            fn default() -> Value { Value {} }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Value { }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for Value { }
        #[automatically_derived]
        impl ::core::clone::Clone for Value {
            #[inline]
            fn clone(&self) -> Value { *self }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Value { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Value {
            #[inline]
            fn eq(&self, other: &Value) -> bool { true }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Value {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "Value")
            }
        }
        pub mod value {
            pub mod batch_2d {
                use crate::api::batch::interface::BatchGenerator;
                use crate::noise::generators::Value;
                use crate::simd::{Arch, Simd};
                impl BatchGenerator<2> for Value {
                    fn sample_batch<A: Arch>(seed: u32,
                        input: [Simd<f32, A>; 2], freq: [Simd<f32, A>; 2])
                        -> Simd<f32, A> {
                        let neg_two: Simd<f32, A> = Simd::splat(-2.0);
                        let three: Simd<f32, A> = Simd::splat(3.0);
                        let hash_mask: Simd<u32, A> = Simd::splat(0x007FFFFF);
                        let exp_bits: Simd<u32, A> = Simd::splat(0x40000000);
                        const BYTE_SHUFFLE: [u8; 64] =
                            [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                    2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                    7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                    6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                        let shuffle_indices =
                            Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                        let channel_seed = Simd::splat(seed);
                        let prime = Simd::splat(0x85ebca6b);
                        let x_scaled = input[0] * freq[0];
                        let y_scaled = input[1] * freq[1];
                        let x_scaled_floored = x_scaled.floor();
                        let y_scaled_floored = y_scaled.floor();
                        let x_grid_lo = x_scaled_floored.cast_int_trunc();
                        let y_grid_lo = y_scaled_floored.cast_int_trunc();
                        let x_dist_lo = x_scaled - x_scaled_floored;
                        let y_dist_lo = y_scaled - y_scaled_floored;
                        let t = x_dist_lo;
                        let s = y_dist_lo;
                        let x_lerp = t * t * t.mul_add(neg_two, three);
                        let y_lerp = s * s * s.mul_add(neg_two, three);
                        let x1: Simd<u32, A> = x_grid_lo.raw_cast() * channel_seed;
                        let y1: Simd<u32, A> = y_grid_lo.raw_cast() * channel_seed;
                        let x2 = x1 + channel_seed;
                        let y2 = y1 + channel_seed;
                        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
                        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
                        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
                        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
                        let hash_tl = x1_shuf * y1_shuf * y1_shuf;
                        let hash_tr = x1_shuf * y2_shuf * y2_shuf;
                        let hash_bl = x2_shuf * y1_shuf * y1_shuf;
                        let hash_br = x2_shuf * y2_shuf * y2_shuf;
                        let val_tl =
                            ((hash_tl & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_tr =
                            ((hash_tr & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_bl =
                            ((hash_bl & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_br =
                            ((hash_br & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let top_lerp = y_lerp.mul_add(val_tr - val_tl, val_tl);
                        let bottom_lerp = y_lerp.mul_add(val_br - val_bl, val_bl);
                        x_lerp.mul_add(bottom_lerp - top_lerp, top_lerp)
                    }
                }
            }
            pub mod batch_3d {
                use crate::api::batch::interface::BatchGenerator;
                use crate::noise::generators::Value;
                use crate::simd::{Arch, Simd};
                impl BatchGenerator<3> for Value {
                    fn sample_batch<A: Arch>(seed: u32,
                        input: [Simd<f32, A>; 3], freq: [Simd<f32, A>; 3])
                        -> Simd<f32, A> {
                        let neg_two: Simd<f32, A> = Simd::splat(-2.0);
                        let three: Simd<f32, A> = Simd::splat(3.0);
                        let hash_mask: Simd<u32, A> = Simd::splat(0x007FFFFF);
                        let exp_bits: Simd<u32, A> = Simd::splat(0x40000000);
                        const BYTE_SHUFFLE: [u8; 64] =
                            [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                    2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                    7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                    6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                        let shuffle_indices =
                            Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                        let channel_seed = Simd::splat(seed);
                        let prime = Simd::splat(0x85ebca6b);
                        let x_scaled = input[0] * freq[0];
                        let y_scaled = input[1] * freq[1];
                        let z_scaled = input[2] * freq[2];
                        let x_scaled_floored = x_scaled.floor();
                        let y_scaled_floored = y_scaled.floor();
                        let z_scaled_floored = z_scaled.floor();
                        let x_grid_lo = x_scaled_floored.cast_int_trunc();
                        let y_grid_lo = y_scaled_floored.cast_int_trunc();
                        let z_grid_lo = z_scaled_floored.cast_int_trunc();
                        let x_dist_lo = x_scaled - x_scaled_floored;
                        let y_dist_lo = y_scaled - y_scaled_floored;
                        let z_dist_lo = z_scaled - z_scaled_floored;
                        let t = x_dist_lo;
                        let s = y_dist_lo;
                        let u = z_dist_lo;
                        let x_lerp = t * t * t.mul_add(neg_two, three);
                        let y_lerp = s * s * s.mul_add(neg_two, three);
                        let z_lerp = u * u * u.mul_add(neg_two, three);
                        let x1: Simd<u32, A> = x_grid_lo.raw_cast() * channel_seed;
                        let y1: Simd<u32, A> = y_grid_lo.raw_cast() * channel_seed;
                        let z1: Simd<u32, A> = z_grid_lo.raw_cast() * channel_seed;
                        let x2 = x1 + channel_seed;
                        let y2 = y1 + channel_seed;
                        let z2 = z1 + channel_seed;
                        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
                        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
                        let z1_shuf = z1.permute_8(shuffle_indices) ^ prime;
                        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
                        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
                        let z2_shuf = z2.permute_8(shuffle_indices) ^ prime;
                        let hash_tlf = x1_shuf * y1_shuf + z1_shuf * y1_shuf;
                        let hash_trf = x1_shuf * y1_shuf + z2_shuf * y1_shuf;
                        let hash_blf = x1_shuf * y2_shuf + z1_shuf * y2_shuf;
                        let hash_brf = x1_shuf * y2_shuf + z2_shuf * y2_shuf;
                        let hash_tlb = x2_shuf * y1_shuf + z1_shuf * y1_shuf;
                        let hash_trb = x2_shuf * y1_shuf + z2_shuf * y1_shuf;
                        let hash_blb = x2_shuf * y2_shuf + z1_shuf * y2_shuf;
                        let hash_brb = x2_shuf * y2_shuf + z2_shuf * y2_shuf;
                        let val_tlf =
                            ((hash_tlf & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_trf =
                            ((hash_trf & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_blf =
                            ((hash_blf & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_brf =
                            ((hash_brf & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_tlb =
                            ((hash_tlb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_trb =
                            ((hash_trb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_blb =
                            ((hash_blb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let val_brb =
                            ((hash_brb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                three;
                        let lerp_tf = z_lerp.mul_add(val_trf - val_tlf, val_tlf);
                        let lerp_bf = z_lerp.mul_add(val_brf - val_blf, val_blf);
                        let lerp_tb = z_lerp.mul_add(val_trb - val_tlb, val_tlb);
                        let lerp_bb = z_lerp.mul_add(val_brb - val_blb, val_blb);
                        let lerp_front = y_lerp.mul_add(lerp_bf - lerp_tf, lerp_tf);
                        let lerp_back = y_lerp.mul_add(lerp_bb - lerp_tb, lerp_tb);
                        x_lerp.mul_add(lerp_back - lerp_front, lerp_front)
                    }
                }
            }
            pub mod grid_2d {
                use std::fmt;
                use std::mem::MaybeUninit;
                use std::ops::Range;
                use crate::GridGenerator;
                use crate::api::grid::interface::GridNoiseParams;
                use crate::generators::Value;
                use crate::noise::combiners::{Combiner, CombinerState};
                use crate::noise::util::grid_data::{GridData, Lerp};
                use crate::noise::util::grid_helpers::{
                    Arena, ArenaBuffer, InterpolationConfig,
                    MaybeUninitSliceSimdExt, assume_init_slice, maybe_tail_load,
                    maybe_tail_store, pad_grid_size, validate_grid_size,
                    validate_state_size,
                };
                use crate::simd::{Arch, Simd};
                pub struct ValueGradients2D<'a> {
                    pub tl: &'a mut [MaybeUninit<f32>],
                    pub tr: &'a mut [MaybeUninit<f32>],
                    pub bl: &'a mut [MaybeUninit<f32>],
                    pub br: &'a mut [MaybeUninit<f32>],
                }
                impl<'a> ValueGradients2D<'a> {
                    #[inline(always)]
                    pub fn new(arena: &'a mut Arena, size: usize) -> Self {
                        Self {
                            tl: arena.allocate(size),
                            tr: arena.allocate(size),
                            bl: arena.allocate(size),
                            br: arena.allocate(size),
                        }
                    }
                    #[inline(always)]
                    pub fn swap_top_bottom(&mut self) {
                        std::mem::swap(&mut self.tl, &mut self.bl);
                        std::mem::swap(&mut self.tr, &mut self.br);
                    }
                }
                impl<'a> fmt::Debug for ValueGradients2D<'a> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        unsafe {
                            f.debug_struct("GridData").field("tl",
                                                &assume_init_slice(self.tl)).field("tr",
                                            &assume_init_slice(self.tr)).field("bl",
                                        &assume_init_slice(self.bl)).field("br",
                                    &assume_init_slice(self.br)).finish()
                        }
                    }
                }
                const LERP: u8 = Lerp::Cubic as u8;
                impl GridGenerator<2> for Value {
                    #[inline(always)]
                    fn sample_grid<A: Arch, C: Combiner, const INIT : bool,
                        const FINAL :
                        bool>(params: GridNoiseParams<2>, fractal_config: C::Config,
                        state: &mut [f32], dst: &mut [f32]) {
                        validate_grid_size(params.grid_size, dst.len());
                        validate_state_size::<C, _>(params.grid_size, state.len());
                        let padded_size = pad_grid_size(params.grid_size);
                        let required_cache =
                            padded_size[1] * 3 + padded_size[0] * 8;
                        let mut cache = ArenaBuffer::with_capacity(required_cache);
                        let mut arena = Arena::with_cache(&mut cache);
                        let num_blocks = A::NUM_SIMD_REG / 4;
                        let bilerp_config =
                            InterpolationConfig::new(num_blocks, params.grid_size[0]);
                        let mut sub_arena =
                            arena.allocate_arena(padded_size[0] * 3 +
                                    padded_size[1] * 3);
                        let mut grid_data =
                            GridData::new::<LERP>(&params, &mut sub_arena,
                                &padded_size);
                        let grad_scratch = arena.allocate(padded_size[0]);
                        let mut gradients =
                            ValueGradients2D::new(&mut arena, padded_size[0]);
                        grid_gradients_2d(&params, &mut grid_data, grad_scratch,
                            gradients.tl, gradients.tr, 0);
                        let mut y_cur_index = 0;
                        for y_it in 0..grid_data.num_loops[1] {
                            let y_next_index =
                                unsafe {
                                    grid_data.grid_indices[1].get_unchecked(y_it).assume_init()
                                        as usize
                                };
                            grid_gradients_2d(&params, &mut grid_data, grad_scratch,
                                gradients.bl, gradients.br, y_it + 1);
                            let y_range = y_cur_index..y_next_index;
                            grid_bilerp::<C, INIT,
                                    FINAL>(&bilerp_config, &fractal_config, &grid_data,
                                &gradients, y_range, (state, dst));
                            gradients.swap_top_bottom();
                            y_cur_index = y_next_index;
                        }
                    }
                }
                #[inline(always)]
                pub(super) fn grid_gradients_2d<'a,
                    A: Arch>(params: &GridNoiseParams<2>,
                    grid_data: &mut GridData<2>,
                    grad_buffer: &mut [MaybeUninit<f32>],
                    left: &'a mut [MaybeUninit<f32>],
                    right: &'a mut [MaybeUninit<f32>], y_it: usize) {
                    let lanes = Simd::<f32, A>::LANES;
                    let y_start = grid_data.grid_start[1] + y_it as i32;
                    let y_rem =
                        grid_data.octave_tiling[1].map_or(y_start,
                            |t| y_start.rem_euclid(t as i32));
                    let y_vec =
                        Simd::splat((y_rem as u32).wrapping_mul(params.seed));
                    let prime = Simd::splat(0x85ebca6b_u32);
                    const BYTE_SHUFFLE: [u8; 64] =
                        [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                    let shuffle_indices =
                        unsafe {
                            Simd::<u8>::from_slice_unchecked(&BYTE_SHUFFLE[..])
                        };
                    let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;
                    let y_shuf = y_shuf * y_shuf;
                    let hash_mask: Simd<u32> = Simd::splat(0x007FFFFF);
                    let exp_bits: Simd<u32> = Simd::splat(0x40000000);
                    let three: Simd<f32> = Simd::splat(3.0);
                    if let Some(x_tiling) = grid_data.octave_tiling[0] {
                        let x_tiling = Simd::splat(x_tiling as f32);
                        let mut x_vec =
                            Simd::splat(grid_data.grid_start[0]) + Simd::iota(0);
                        let x_vec_stride = Simd::splat(lanes as i32);
                        let seed_vec = Simd::splat(params.seed);
                        let end_index = grid_data.num_loops[0] + 1;
                        for i in (0..end_index).step_by(lanes) {
                            let x_floats = x_vec.cast_float();
                            let x_rem =
                                x_floats - (x_floats / x_tiling).floor() * x_tiling;
                            let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;
                            let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
                            let hash = y_shuf * x_shuf;
                            let grad =
                                ((hash & hash_mask) | exp_bits).raw_cast() - three;
                            unsafe { grad_buffer.write_simd_aligned(i, grad) };
                            x_vec += x_vec_stride;
                        }
                    } else {
                        let iota_vec = Simd::iota(0) * Simd::splat(params.seed);
                        let mut x_vec =
                            Simd::splat((grid_data.grid_start[0] as
                                                u32).wrapping_mul(params.seed)) + iota_vec;
                        let x_vec_stride =
                            Simd::splat((lanes as u32).wrapping_mul(params.seed));
                        let end_index = grid_data.num_loops[0] + 1;
                        for i in (0..end_index).step_by(lanes) {
                            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
                            let hash = y_shuf * x_shuf;
                            let grad =
                                ((hash & hash_mask) | exp_bits).raw_cast() - three;
                            unsafe { grad_buffer.write_simd_aligned(i, grad) };
                            x_vec += x_vec_stride;
                        }
                    }
                    let mut x_cur_index = 0;
                    for x_it in 0..grid_data.num_loops[0] {
                        let x_next_index =
                            unsafe {
                                grid_data.grid_indices[0].get_unchecked(x_it).assume_init()
                            };
                        let mut amount = (x_next_index - x_cur_index) as isize;
                        unsafe {
                            let l = grad_buffer.get_unchecked(x_it).assume_init();
                            let r = grad_buffer.get_unchecked(x_it + 1).assume_init();
                            let mut index = x_cur_index as usize;
                            while amount > 0 {
                                left.write_simd(index, Simd::splat(l));
                                right.write_simd(index, Simd::splat(r));
                                amount -= lanes as isize;
                                index += lanes;
                            }
                        }
                        x_cur_index = x_next_index;
                    }
                }
                /// Handles interpolation execution state and fills
                /// the dst slice with interpolated values from gradient dot produtcts.
                pub(crate) struct BilerpExecuter<'a, A: Arch, C: Combiner,
                    const INIT : bool, const FINAL : bool> {
                    config: &'a InterpolationConfig<A>,
                    fractal_config: &'a C::Config,
                    grid_data: &'a GridData<'a, 2>,
                    gradients: &'a ValueGradients2D<'a>,
                    y_range: Range<usize>,
                    top: A::Block2<f32>,
                    dif: A::Block2<f32>,
                    weight: Simd<f32, A>,
                }
                /// Fills the dst slice with interpolated dot products from gradients.
                #[inline(always)]
                pub(super) fn grid_bilerp<A: Arch, C: Combiner, const INIT :
                    bool, const FINAL :
                    bool>(config: &InterpolationConfig<A>,
                    fractal_config: &C::Config, grid_data: &GridData<2>,
                    gradients: &ValueGradients2D, y_range: Range<usize>,
                    output: (&mut [f32], &mut [f32])) {
                    let mut executer =
                        BilerpExecuter::<A, C, INIT,
                            FINAL> {
                            config,
                            fractal_config,
                            grid_data,
                            gradients,
                            y_range,
                            top: Default::default(),
                            dif: Default::default(),
                            weight: Simd::splat(grid_data.weight),
                        };
                    let (state, dst) = output;
                    if config.has_block_head {
                        executer.interpolate::<false>(state, dst);
                    }
                    if config.has_block_tail {
                        executer.interpolate::<true>(state, dst);
                        std::hint::cold_path();
                    }
                }
                impl<'a, A: Arch, C: Combiner, const INIT : bool, const FINAL
                    : bool> BilerpExecuter<'a, A, C, INIT, FINAL> {
                    #[inline(always)]
                    pub fn interpolate<const IS_TAIL :
                        bool>(&mut self, state: &mut [f32], dst: &mut [f32]) {
                        let range =
                            if IS_TAIL {
                                self.config.block_tail_start..self.grid_data.grid_size[0]
                            } else { 0..self.config.block_tail_start };
                        for x in range.step_by(self.config.block_lanes) {
                            self.initialize_factors::<IS_TAIL>(x);
                            let mut y = self.y_range.start;
                            while y < self.y_range.end {
                                if y + 4 > self.y_range.end {
                                    self.process_factors::<IS_TAIL>(x, y, state, dst);
                                    y += 1;
                                } else {
                                    self.process_factors::<IS_TAIL>(x, y, state, dst);
                                    self.process_factors::<IS_TAIL>(x, y + 1, state, dst);
                                    self.process_factors::<IS_TAIL>(x, y + 2, state, dst);
                                    self.process_factors::<IS_TAIL>(x, y + 3, state, dst);
                                    y += 4;
                                }
                            }
                        }
                    }
                    #[inline(always)]
                    fn initialize_factors<const IS_TAIL :
                        bool>(&mut self, x: usize) {
                        let num_blocks =
                            if IS_TAIL {
                                self.config.block_tail_size
                            } else { self.config.num_blocks };
                        for block in 0..num_blocks {
                            let index = x + Simd::<f32, A>::LANES * block;
                            let x_lerp =
                                unsafe {
                                    self.grid_data.fade_factors[0].load_simd_aligned(index)
                                };
                            let tl =
                                unsafe { self.gradients.tl.load_simd_aligned(index) };
                            let tr =
                                unsafe { self.gradients.tr.load_simd_aligned(index) };
                            let bl =
                                unsafe { self.gradients.bl.load_simd_aligned(index) };
                            let br =
                                unsafe { self.gradients.br.load_simd_aligned(index) };
                            self.top[block] = x_lerp.mul_add(tr - tl, tl) * self.weight;
                            let bottom = x_lerp.mul_add(br - bl, bl) * self.weight;
                            self.dif[block] = bottom - self.top[block];
                        }
                    }
                    #[inline(always)]
                    fn process_factors<const IS_TAIL :
                        bool>(&mut self, x: usize, y: usize, state: &mut [f32],
                        dst: &mut [f32]) {
                        let y_lerp =
                            Simd::splat(unsafe {
                                    self.grid_data.fade_factors[1].get_unchecked(y).assume_init()
                                });
                        let range =
                            if IS_TAIL {
                                0..self.config.block_tail_size
                            } else { 0..self.config.num_blocks };
                        let index = y * self.grid_data.grid_size[0] + x;
                        let tail_end = index + self.config.tail_size;
                        for block in range {
                            let index = index + block * Simd::<f32, A>::LANES;
                            let output =
                                y_lerp.mul_add(self.dif[block], self.top[block]);
                            let (cur_state, mut result) =
                                if INIT {
                                    C::initialize_sample(self.fractal_config, output)
                                } else {
                                    let mut cur_state = C::State::default();
                                    for i in 0..C::State::STATE_SIZE {
                                        let offset = i * self.grid_data.total_size;
                                        let index = index + offset;
                                        let tail_end = tail_end + offset;
                                        cur_state[i] =
                                            unsafe {
                                                maybe_tail_load::<IS_TAIL>(index..tail_end, state)
                                            };
                                    }
                                    let cur_result =
                                        unsafe { maybe_tail_load::<IS_TAIL>(index..tail_end, dst) };
                                    C::apply_sample(self.fractal_config, cur_state, cur_result,
                                        output)
                                };
                            if !FINAL {
                                for i in 0..C::State::STATE_SIZE {
                                    let offset = i * self.grid_data.total_size;
                                    let index = index + offset;
                                    let tail_end = tail_end + offset;
                                    unsafe {
                                        maybe_tail_store::<IS_TAIL>(index..tail_end, cur_state[i],
                                            state)
                                    };
                                }
                            }
                            if FINAL {
                                result =
                                    C::finalize_sample(self.fractal_config, cur_state, result);
                            }
                            unsafe {
                                maybe_tail_store::<IS_TAIL>(index..tail_end, result, dst)
                            };
                        }
                    }
                }
            }
            pub mod grid_3d {
                use std::array::from_fn;
                use std::mem::MaybeUninit;
                use std::ops::Range;
                use crate::GridGenerator;
                use crate::api::grid::interface::GridNoiseParams;
                use crate::generators::Value;
                use crate::noise::combiners::{Combiner, CombinerState};
                use crate::noise::util::grid_data::{GridData, Lerp};
                use crate::noise::util::grid_helpers::{
                    Arena, ArenaBuffer, InterpolationConfig,
                    MaybeUninitSliceSimdExt, maybe_tail_load, maybe_tail_store,
                    pad_grid_size, validate_grid_size, validate_state_size,
                };
                use crate::simd::{Arch, Simd};
                pub struct ValueGradients3D<'a> {
                    pub tlf: &'a mut [MaybeUninit<f32>],
                    pub trf: &'a mut [MaybeUninit<f32>],
                    pub blf: &'a mut [MaybeUninit<f32>],
                    pub brf: &'a mut [MaybeUninit<f32>],
                    pub tlb: &'a mut [MaybeUninit<f32>],
                    pub trb: &'a mut [MaybeUninit<f32>],
                    pub blb: &'a mut [MaybeUninit<f32>],
                    pub brb: &'a mut [MaybeUninit<f32>],
                    pub grad_buffers: [&'a mut [MaybeUninit<f32>]; 2],
                }
                impl<'a> ValueGradients3D<'a> {
                    #[inline(always)]
                    pub fn new(arena: &'a mut Arena, size: usize) -> Self {
                        Self {
                            tlf: arena.allocate(size),
                            trf: arena.allocate(size),
                            blf: arena.allocate(size),
                            brf: arena.allocate(size),
                            tlb: arena.allocate(size),
                            trb: arena.allocate(size),
                            blb: arena.allocate(size),
                            brb: arena.allocate(size),
                            grad_buffers: from_fn(|_| arena.allocate(size)),
                        }
                    }
                    #[inline(always)]
                    pub fn swap_top_bottom(&mut self) {
                        std::mem::swap(&mut self.tlf, &mut self.blf);
                        std::mem::swap(&mut self.trf, &mut self.brf);
                        std::mem::swap(&mut self.tlb, &mut self.blb);
                        std::mem::swap(&mut self.trb, &mut self.brb);
                    }
                }
                const LERP: u8 = Lerp::Quintic as u8;
                impl GridGenerator<3> for Value {
                    fn sample_grid<A: Arch, C: Combiner, const INIT : bool,
                        const FINAL :
                        bool>(params: GridNoiseParams<3>, fractal_config: C::Config,
                        state: &mut [f32], dst: &mut [f32]) {
                        validate_grid_size(params.grid_size, dst.len());
                        validate_state_size::<C, _>(params.grid_size, state.len());
                        let padded_size = pad_grid_size(params.grid_size);
                        let required_cache =
                            padded_size[0] * 17 + padded_size[1] * 3 +
                                padded_size[2] * 3;
                        let mut cache = ArenaBuffer::with_capacity(required_cache);
                        let mut arena = Arena::with_cache(&mut cache);
                        let mut data_arena =
                            arena.allocate_arena(padded_size.iter().fold(0,
                                    |n, x| n + 3 * x));
                        let mut trilerp_arena =
                            arena.allocate_arena(padded_size[0] * 4);
                        let num_blocks = A::NUM_SIMD_REG / 4;
                        let bilerp_config =
                            InterpolationConfig::new(num_blocks, params.grid_size[0]);
                        let grid_data =
                            GridData::new::<LERP>(&params, &mut data_arena,
                                &padded_size);
                        let mut trilerp_buffers =
                            TrilerpBuffers::new(&mut trilerp_arena, padded_size[0]);
                        let mut gradients =
                            ValueGradients3D::new(&mut arena, padded_size[0]);
                        let mut z_cur_index = 0;
                        for z_it in 0..grid_data.num_loops[2] {
                            let z_next_index =
                                unsafe {
                                    grid_data.grid_indices[2].get_unchecked(z_it).assume_init()
                                        as usize
                                };
                            let z_range = z_cur_index..z_next_index;
                            grid_gradients_3d(&params, &grid_data, &mut gradients, 0,
                                z_it);
                            gradients.swap_top_bottom();
                            let mut y_cur_index = 0;
                            for y_it in 0..grid_data.num_loops[1] {
                                let y_next_index =
                                    unsafe {
                                        grid_data.grid_indices[1].get_unchecked(y_it).assume_init()
                                            as usize
                                    };
                                let y_range = y_cur_index..y_next_index;
                                grid_gradients_3d(&params, &grid_data, &mut gradients,
                                    y_it + 1, z_it);
                                grid_trilerp::<C, INIT,
                                        FINAL>(&mut trilerp_buffers, &bilerp_config,
                                    &fractal_config, &grid_data, &gradients,
                                    (y_range, z_range.clone()), (state, dst));
                                gradients.swap_top_bottom();
                                y_cur_index = y_next_index;
                            }
                            z_cur_index = z_next_index;
                        }
                    }
                }
                #[inline(always)]
                pub(super) fn grid_gradients_3d<'a,
                    A: Arch>(params: &GridNoiseParams<3>,
                    grid_data: &GridData<3>,
                    gradients: &mut ValueGradients3D<'a>, y_it: usize,
                    z_it: usize) {
                    let lanes = Simd::<f32, A>::LANES;
                    let y_start = y_it as i32 + grid_data.grid_start[1];
                    let z_start = z_it as i32 + grid_data.grid_start[2];
                    let (z1, z2) =
                        match grid_data.octave_tiling[2] {
                            None =>
                                ((z_start as u32).wrapping_mul(params.seed),
                                    (z_start as
                                                    u32).wrapping_mul(params.seed).wrapping_add(params.seed)),
                            Some(t) =>
                                ((z_start.rem_euclid(t as i32)) as u32,
                                    ((z_start + 1).rem_euclid(t as i32)) as u32),
                        };
                    let z_vec = [Simd::splat(z1), Simd::splat(z2)];
                    let y_rem =
                        grid_data.octave_tiling[1].map_or(y_start,
                            |t| y_start.rem_euclid(t as i32));
                    let y_vec =
                        Simd::splat((y_rem as u32).wrapping_mul(params.seed));
                    const BYTE_SHUFFLE: [u8; 64] =
                        [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                    let shuffle_indices =
                        Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                    let prime = Simd::splat(0x85ebca6b_u32);
                    let z_shuf: [_; 2] =
                        from_fn(|i| z_vec[i].permute_8(shuffle_indices) ^ prime);
                    let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;
                    let zy_mix: [_; 2] = from_fn(|i| z_shuf[i] * y_shuf);
                    let end_index = grid_data.num_loops[0] + 1;
                    let hash_mask: Simd<u32> = Simd::splat(0x007FFFFF);
                    let exp_bits: Simd<u32> = Simd::splat(0x40000000);
                    let three: Simd<f32> = Simd::splat(3.0);
                    if let Some(x_tiling) = grid_data.octave_tiling[0] {
                        let x_tiling = Simd::splat(x_tiling as f32);
                        let mut x_vec =
                            Simd::splat(grid_data.grid_start[0]) + Simd::iota(0);
                        let x_vec_stride = Simd::splat(lanes as i32);
                        let seed_vec = Simd::splat(params.seed);
                        for i in (0..end_index).step_by(lanes) {
                            let x_floats = x_vec.cast_float();
                            let x_rem =
                                x_floats - (x_floats / x_tiling).floor() * x_tiling;
                            let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;
                            let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
                            let hashes: [_; 2] =
                                from_fn(|i| zy_mix[i] + x_shuf * y_shuf);
                            let grads: [_; 2] =
                                from_fn(|i|
                                        ((hashes[i] & hash_mask) | exp_bits).raw_cast() - three);
                            unsafe {
                                gradients.grad_buffers[0].write_simd_aligned(i, grads[0]);
                                gradients.grad_buffers[1].write_simd_aligned(i, grads[1]);
                            };
                            x_vec += x_vec_stride;
                        }
                    } else {
                        let iota_vec = Simd::iota(0) * Simd::splat(params.seed);
                        let x_start_seeded =
                            (grid_data.grid_start[0] as u32).wrapping_mul(params.seed);
                        let mut x_vec = Simd::splat(x_start_seeded) + iota_vec;
                        let x_vec_stride =
                            Simd::splat((lanes as u32).wrapping_mul(params.seed));
                        for i in (0..end_index).step_by(lanes) {
                            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
                            let hashes: [_; 2] =
                                from_fn(|i| zy_mix[i] + x_shuf * y_shuf);
                            let grads: [_; 2] =
                                from_fn(|i|
                                        ((hashes[i] & hash_mask) | exp_bits).raw_cast() - three);
                            unsafe {
                                gradients.grad_buffers[0].write_simd_aligned(i, grads[0]);
                                gradients.grad_buffers[1].write_simd_aligned(i, grads[1]);
                            };
                            x_vec += x_vec_stride;
                        }
                    }
                    grid_gradients_3d_set_loop::<true>(grid_data, gradients);
                    grid_gradients_3d_set_loop::<false>(grid_data, gradients);
                }
                #[inline(always)]
                pub(super) fn grid_gradients_3d_set_loop<'a, A: Arch, const
                    IS_FRONT :
                    bool>(grid_data: &GridData<3>,
                    gradients: &mut ValueGradients3D<'a>) {
                    let (grad_buffer, left, right) =
                        if IS_FRONT {
                            (&mut gradients.grad_buffers[0], &mut gradients.blf,
                                &mut gradients.brf)
                        } else {
                            (&mut gradients.grad_buffers[1], &mut gradients.blb,
                                &mut gradients.brb)
                        };
                    let mut x_cur_index = 0;
                    for x_it in 0..grid_data.num_loops[0] {
                        let x_next_index =
                            unsafe {
                                grid_data.grid_indices[0].get_unchecked(x_it).assume_init()
                            };
                        let mut amount = (x_next_index - x_cur_index) as isize;
                        unsafe {
                            let l = grad_buffer.get_unchecked(x_it).assume_init();
                            let r = grad_buffer.get_unchecked(x_it + 1).assume_init();
                            let mut index = x_cur_index as usize;
                            while amount > 0 {
                                left.write_simd(index, Simd::splat(l));
                                right.write_simd(index, Simd::splat(r));
                                amount -= Simd::<f32, A>::LANES as isize;
                                index += Simd::<f32, A>::LANES;
                            }
                        }
                        x_cur_index = x_next_index;
                    }
                }
                pub(crate) struct TrilerpBuffers<'a> {
                    tf_base: &'a mut [MaybeUninit<f32>],
                    bf_base: &'a mut [MaybeUninit<f32>],
                    top_base_dif: &'a mut [MaybeUninit<f32>],
                    bottom_base_dif: &'a mut [MaybeUninit<f32>],
                }
                impl<'a> TrilerpBuffers<'a> {
                    #[inline(always)]
                    pub fn new(arena: &'a mut Arena, x_size: usize) -> Self {
                        Self {
                            tf_base: arena.allocate(x_size),
                            bf_base: arena.allocate(x_size),
                            top_base_dif: arena.allocate(x_size),
                            bottom_base_dif: arena.allocate(x_size),
                        }
                    }
                }
                /// Handles interpolation execution state and fills
                /// the dst slice with interpolated values from gradient dot produtcts.
                pub(crate) struct DottedTrilerpExecuter<'a, A: Arch,
                    C: Combiner, const INIT : bool, const FINAL : bool> {
                    config: &'a InterpolationConfig<A>,
                    fractal_config: &'a C::Config,
                    grid_data: &'a GridData<'a, 3>,
                    gradients: &'a ValueGradients3D<'a>,
                    y_range: Range<usize>,
                    z_range: Range<usize>,
                    top: A::Block2<f32>,
                    dif: A::Block2<f32>,
                    weight: Simd<f32, A>,
                }
                /// Fills the dst slice with interpolated dot products from gradients.
                #[inline(always)]
                pub(super) fn grid_trilerp<A: Arch, C: Combiner, const INIT :
                    bool, const FINAL :
                    bool>(buffers: &mut TrilerpBuffers,
                    config: &InterpolationConfig<A>, fractal_config: &C::Config,
                    grid_data: &GridData<3>, gradients: &ValueGradients3D,
                    ranges: (Range<usize>, Range<usize>),
                    output: (&mut [f32], &mut [f32])) {
                    let mut executer =
                        DottedTrilerpExecuter::<A, C, INIT,
                            FINAL> {
                            config,
                            fractal_config,
                            grid_data,
                            gradients,
                            y_range: ranges.0,
                            z_range: ranges.1,
                            top: Default::default(),
                            dif: Default::default(),
                            weight: Simd::splat(grid_data.weight),
                        };
                    executer.initialize_trilerp_buffers(buffers);
                    let (state, dst) = output;
                    if config.has_block_head {
                        executer.interpolate::<false>(buffers, state, dst);
                    }
                    if config.has_block_tail {
                        executer.interpolate::<true>(buffers, state, dst);
                        std::hint::cold_path();
                    }
                }
                impl<'a, A: Arch, C: Combiner, const INIT : bool, const FINAL
                    : bool> DottedTrilerpExecuter<'a, A, C, INIT, FINAL> {
                    #[inline(always)]
                    pub fn interpolate<const IS_TAIL :
                        bool>(&mut self, buffers: &TrilerpBuffers,
                        state: &mut [f32], dst: &mut [f32]) {
                        let range =
                            if IS_TAIL {
                                self.config.block_tail_start..self.grid_data.grid_size[0]
                            } else { 0..self.config.block_tail_start };
                        let z_hop =
                            self.grid_data.grid_size[0] * self.grid_data.grid_size[1];
                        let y_hop = self.grid_data.grid_size[0];
                        for z in self.z_range.start..self.z_range.end {
                            let z_lerp =
                                unsafe { self.grid_data.fade_factors[2].get_unchecked(z) };
                            let z_lerp = unsafe { z_lerp.assume_init() };
                            let z_lerp = Simd::splat(z_lerp);
                            for x in range.clone().step_by(self.config.block_lanes) {
                                self.intialize_factors::<IS_TAIL>(buffers, x, z_lerp);
                                let index = z * z_hop + x;
                                let mut y = self.y_range.start;
                                while y < self.y_range.end {
                                    let index = index + y * y_hop;
                                    if y + 4 > self.y_range.end {
                                        self.process_factors::<IS_TAIL>(index, y, state, dst);
                                        y += 1;
                                    } else {
                                        self.process_factors::<IS_TAIL>(index, y, state, dst);
                                        self.process_factors::<IS_TAIL>(index + y_hop, y + 1, state,
                                            dst);
                                        self.process_factors::<IS_TAIL>(index + 2 * y_hop, y + 2,
                                            state, dst);
                                        self.process_factors::<IS_TAIL>(index + 3 * y_hop, y + 3,
                                            state, dst);
                                        y += 4;
                                    }
                                }
                            }
                        }
                    }
                    #[inline(always)]
                    fn initialize_trilerp_buffers(&mut self,
                        buffers: &mut TrilerpBuffers) {
                        for x in
                            (0..self.grid_data.grid_size[0]).step_by(Simd::<f32,
                                    A>::LANES) {
                            unsafe {
                                let x_lerp =
                                    self.grid_data.fade_factors[0].load_simd_aligned(x);
                                let tlf = self.gradients.tlf.load_simd_aligned(x);
                                let trf = self.gradients.trf.load_simd_aligned(x);
                                let blf = self.gradients.blf.load_simd_aligned(x);
                                let brf = self.gradients.brf.load_simd_aligned(x);
                                let tlb = self.gradients.tlb.load_simd_aligned(x);
                                let trb = self.gradients.trb.load_simd_aligned(x);
                                let blb = self.gradients.blb.load_simd_aligned(x);
                                let brb = self.gradients.brb.load_simd_aligned(x);
                                let tf_base = x_lerp.mul_add(trf - tlf, tlf) * self.weight;
                                let bf_base = x_lerp.mul_add(brf - blf, blf) * self.weight;
                                let hi_base_dif =
                                    x_lerp.mul_add(trb - tlb,
                                            tlb).mul_sub(self.weight, tf_base);
                                let lo_base_dif =
                                    x_lerp.mul_add(brb - blb,
                                            blb).mul_sub(self.weight, bf_base);
                                buffers.tf_base.write_simd_aligned(x, tf_base);
                                buffers.bf_base.write_simd_aligned(x, bf_base);
                                buffers.top_base_dif.write_simd_aligned(x, hi_base_dif);
                                buffers.bottom_base_dif.write_simd_aligned(x, lo_base_dif);
                            }
                        }
                    }
                    #[inline(always)]
                    fn intialize_factors<const IS_TAIL :
                        bool>(&mut self, buffers: &TrilerpBuffers, x: usize,
                        z_lerp: Simd<f32, A>) {
                        let num_blocks =
                            if IS_TAIL {
                                self.config.block_tail_size
                            } else { self.config.num_blocks };
                        for block in 0..num_blocks {
                            unsafe {
                                let index = x + Simd::<f32, A>::LANES * block;
                                let tf = buffers.tf_base.load_simd_aligned(index);
                                let bf = buffers.bf_base.load_simd_aligned(index);
                                let top_dif = buffers.top_base_dif.load_simd_aligned(index);
                                let bottom_dif =
                                    buffers.bottom_base_dif.load_simd_aligned(index);
                                self.top[block] = z_lerp.mul_add(top_dif, tf);
                                let bottom = z_lerp.mul_add(bottom_dif, bf);
                                self.dif[block] = bottom - self.top[block];
                            }
                        }
                    }
                    #[inline(always)]
                    fn process_factors<const IS_TAIL :
                        bool>(&mut self, index: usize, y: usize, state: &mut [f32],
                        dst: &mut [f32]) {
                        let y_lerp =
                            Simd::splat(unsafe {
                                    self.grid_data.fade_factors[1].get_unchecked(y).assume_init()
                                });
                        let range =
                            if IS_TAIL {
                                0..self.config.block_tail_size
                            } else { 0..self.config.num_blocks };
                        let tail_end = index + self.config.tail_size;
                        for block in range {
                            let index = index + block * Simd::<f32, A>::LANES;
                            let output =
                                y_lerp.mul_add(self.dif[block], self.top[block]);
                            let (cur_state, mut result) =
                                if INIT {
                                    C::initialize_sample(self.fractal_config, output)
                                } else {
                                    let mut cur_state = C::State::default();
                                    for i in 0..C::State::STATE_SIZE {
                                        let offset = i * self.grid_data.total_size;
                                        let index = index + offset;
                                        let tail_end = tail_end + offset;
                                        cur_state[i] =
                                            unsafe {
                                                maybe_tail_load::<IS_TAIL>(index..tail_end, state)
                                            };
                                    }
                                    let cur_result =
                                        unsafe { maybe_tail_load::<IS_TAIL>(index..tail_end, dst) };
                                    C::apply_sample(self.fractal_config, cur_state, cur_result,
                                        output)
                                };
                            if !FINAL {
                                for i in 0..C::State::STATE_SIZE {
                                    let offset = i * self.grid_data.total_size;
                                    let index = index + offset;
                                    let tail_end = tail_end + offset;
                                    unsafe {
                                        maybe_tail_store::<IS_TAIL>(index..tail_end, cur_state[i],
                                            state)
                                    };
                                }
                            }
                            if FINAL {
                                result =
                                    C::finalize_sample(self.fractal_config, cur_state, result);
                            }
                            unsafe {
                                maybe_tail_store::<IS_TAIL>(index..tail_end, result, dst)
                            };
                        }
                    }
                }
            }
        }
        pub struct Simplex {}
        #[automatically_derived]
        impl ::core::default::Default for Simplex {
            #[inline]
            fn default() -> Simplex { Simplex {} }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Simplex { }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for Simplex { }
        #[automatically_derived]
        impl ::core::clone::Clone for Simplex {
            #[inline]
            fn clone(&self) -> Simplex { *self }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Simplex { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Simplex {
            #[inline]
            fn eq(&self, other: &Simplex) -> bool { true }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Simplex {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "Simplex")
            }
        }
        pub mod simplex {
            pub mod batch_2d {
                use std::f32::consts::SQRT_2;
                use crate::api::batch::interface::BatchGenerator;
                use crate::noise::generators::Simplex;
                use crate::simd::{Arch, Simd};
                const SQRT_3: f32 = 1.732_050_8;
                const SKEW_2D: f32 = (SQRT_3 - 1.0) / 2.0;
                const UNSKEW_2D: f32 = (3.0 - SQRT_3) / 6.0;
                const SCALE: f32 = 80.0;
                const SCALED_SQRT: f32 = (SQRT_2 / 2.0) * SCALE;
                const A: f32 = SCALE;
                const B: f32 = SCALED_SQRT;
                const C: f32 = 0.0;
                pub const X_GRADIENTS_2D: [f32; 8] =
                    [A, B, C, -B, -A, -B, C, B];
                pub const Y_GRADIENTS_2D: [f32; 8] =
                    [C, B, A, B, C, -B, -A, -B];
                impl BatchGenerator<2> for Simplex {
                    fn sample_batch<A: Arch>(seed: u32,
                        input: [Simd<f32, A>; 2], freq: [Simd<f32, A>; 2])
                        -> Simd<f32, A> {
                        let skew: Simd<f32, A> = Simd::splat(SKEW_2D);
                        let unskew: Simd<f32, A> = Simd::splat(UNSKEW_2D);
                        let subbed_unskew: Simd<f32, A> =
                            Simd::splat(UNSKEW_2D - 1.0);
                        let hi_skew_offset: Simd<f32, A> =
                            Simd::splat(2.0 * UNSKEW_2D - 1.0);
                        let half: Simd<f32, A> = Simd::splat(0.5);
                        let zero: Simd<f32, A> = Simd::splat(0.0);
                        let t_hi_coef = Simd::splat(2.0 * SQRT_3 / 3.0);
                        let neg_two_thirds = Simd::splat(-2.0 / 3.0);
                        const BYTE_SHUFFLE: [u8; 64] =
                            [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                    2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                    7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                    6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                        let shuffle_indices =
                            Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                        let channel_seed = Simd::splat(seed);
                        let prime = Simd::splat(0x85ebca6b_u32);
                        let x_scaled = input[0] * freq[0];
                        let y_scaled = input[1] * freq[1];
                        let s = (x_scaled + y_scaled) * skew;
                        let x_grid = (x_scaled + s).floor();
                        let y_grid = (y_scaled + s).floor();
                        let unskew_sub = (x_grid + y_grid) * unskew;
                        let x_dist_lo = x_scaled - x_grid + unskew_sub;
                        let y_dist_lo = y_scaled - y_grid + unskew_sub;
                        let triangle_mask = x_dist_lo.simd_gt(y_dist_lo);
                        let x_dist_mi_offset =
                            triangle_mask.select(subbed_unskew, unskew);
                        let y_dist_mi_offset =
                            triangle_mask.select(unskew, subbed_unskew);
                        let x_dist_mi = x_dist_lo + x_dist_mi_offset;
                        let y_dist_mi = y_dist_lo + y_dist_mi_offset;
                        let x_dist_hi = x_dist_lo + hi_skew_offset;
                        let y_dist_hi = y_dist_lo + hi_skew_offset;
                        let x1: Simd<u32, A> =
                            x_grid.cast_int_trunc().raw_cast() * channel_seed;
                        let y1: Simd<u32, A> =
                            y_grid.cast_int_trunc().raw_cast() * channel_seed;
                        let x2 = x1 + channel_seed;
                        let y2 = y1 + channel_seed;
                        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
                        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
                        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
                        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
                        let mix_lo = (x1_shuf * y1_shuf) ^ x1_shuf;
                        let mix_hi = (x2_shuf * y2_shuf) ^ x2_shuf;
                        let x_shuf_mi =
                            triangle_mask.raw_cast().select(x2_shuf, x1_shuf);
                        let y_shuf_mi =
                            triangle_mask.raw_cast().select(y1_shuf, y2_shuf);
                        let mix_mi = (x_shuf_mi * y_shuf_mi) ^ x_shuf_mi;
                        let indices_lo = mix_lo >> 29;
                        let indices_mi = mix_mi >> 29;
                        let indices_hi = mix_hi >> 29;
                        let x_grads_lo = indices_lo.gather(&X_GRADIENTS_2D);
                        let y_grads_lo = indices_lo.gather(&Y_GRADIENTS_2D);
                        let x_grads_mi = indices_mi.gather(&X_GRADIENTS_2D);
                        let y_grads_mi = indices_mi.gather(&Y_GRADIENTS_2D);
                        let x_grads_hi = indices_hi.gather(&X_GRADIENTS_2D);
                        let y_grads_hi = indices_hi.gather(&Y_GRADIENTS_2D);
                        let t_lo_pre =
                            half - x_dist_lo.mul_add(x_dist_lo, y_dist_lo * y_dist_lo);
                        let t_mi_pre =
                            half - x_dist_mi.mul_add(x_dist_mi, y_dist_mi * y_dist_mi);
                        let t_hi_pre =
                            t_lo_pre +
                                t_hi_coef.mul_add(x_dist_lo + y_dist_lo, neg_two_thirds);
                        let t_lo = t_lo_pre.max(zero);
                        let t_mi = t_mi_pre.max(zero);
                        let t_hi = t_hi_pre.max(zero);
                        let t2_lo = t_lo * t_lo;
                        let t2_mi = t_mi * t_mi;
                        let t2_hi = t_hi * t_hi;
                        let t4_lo = t2_lo * t2_lo;
                        let t4_mi = t2_mi * t2_mi;
                        let t4_hi = t2_hi * t2_hi;
                        let dot_lo =
                            x_grads_lo.mul_add(x_dist_lo, y_grads_lo * y_dist_lo);
                        let dot_mi =
                            x_grads_mi.mul_add(x_dist_mi, y_grads_mi * y_dist_mi);
                        let dot_hi =
                            x_grads_hi.mul_add(x_dist_hi, y_grads_hi * y_dist_hi);
                        t4_lo.mul_add(dot_lo, t4_mi.mul_add(dot_mi, t4_hi * dot_hi))
                    }
                }
            }
            pub mod batch_3d {
                use crate::api::batch::interface::BatchGenerator;
                use crate::noise::generators::Simplex;
                use crate::simd::{Arch, Simd};
                const SKEW_3D: f32 = 1.0 / 3.0;
                const UNSKEW_3D: f32 = 1.0 / 6.0;
                impl BatchGenerator<3> for Simplex {
                    fn sample_batch<A: Arch>(seed: u32,
                        input: [Simd<f32, A>; 3], freq: [Simd<f32, A>; 3])
                        -> Simd<f32, A> {
                        let skew: Simd<f32, A> = Simd::splat(SKEW_3D);
                        let unskew: Simd<f32, A> = Simd::splat(UNSKEW_3D);
                        let subbed_unskew: Simd<f32, A> =
                            Simd::splat(UNSKEW_3D - 1.0);
                        let hi_skew_offset: Simd<f32, A> =
                            Simd::splat(3.0 * UNSKEW_3D - 1.0);
                        let two_unskew: Simd<f32, A> = Simd::splat(2.0 * UNSKEW_3D);
                        let mi2_skew_offset: Simd<f32, A> =
                            Simd::splat(2.0 * UNSKEW_3D - 1.0);
                        let half: Simd<f32, A> = Simd::splat(0.5);
                        let zero: Simd<f32, A> = Simd::splat(0.0);
                        let three_int: Simd<u32, A> = Simd::splat(3);
                        let c1: Simd<u32, A> = Simd::splat(0x09009999);
                        let c2: Simd<u32, A> = Simd::splat(0xA59900A5);
                        let c3: Simd<u32, A> = Simd::splat(0x90A5A500);
                        const BYTE_SHUFFLE: [u8; 64] =
                            [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                    2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                    7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                    6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                        const S: f32 = 100.0;
                        const GRAD_TABLE: [f32; 4] = [0.0, S, -S, 0.0];
                        let shuffle_indices =
                            Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                        let channel_seed = Simd::splat(seed);
                        let prime = Simd::splat(0x85ebca6b_u32);
                        let x_scaled = input[0] * freq[0];
                        let y_scaled = input[1] * freq[1];
                        let z_scaled = input[2] * freq[2];
                        let s = (x_scaled + y_scaled + z_scaled) * skew;
                        let x_grid = (x_scaled + s).floor();
                        let y_grid = (y_scaled + s).floor();
                        let z_grid = (z_scaled + s).floor();
                        let unskew_sub = (x_grid + y_grid + z_grid) * unskew;
                        let x_dist_lo = x_scaled - x_grid + unskew_sub;
                        let y_dist_lo = y_scaled - y_grid + unskew_sub;
                        let z_dist_lo = z_scaled - z_grid + unskew_sub;
                        let x_gt_y = x_dist_lo.simd_gt(y_dist_lo);
                        let x_gt_z = x_dist_lo.simd_gt(z_dist_lo);
                        let ny_gt_z = y_dist_lo.simd_le(z_dist_lo);
                        let nx_gt_y = x_dist_lo.simd_le(y_dist_lo);
                        let nx_gt_z = x_dist_lo.simd_le(z_dist_lo);
                        let y_gt_z = y_dist_lo.simd_gt(z_dist_lo);
                        let i1 = x_gt_y & x_gt_z;
                        let j1 = nx_gt_y & y_gt_z;
                        let k1 = nx_gt_z & ny_gt_z;
                        let i2 = x_gt_y | x_gt_z;
                        let j2 = nx_gt_y | y_gt_z;
                        let k2 = nx_gt_z | ny_gt_z;
                        let x_dist_mi1 =
                            x_dist_lo + i1.select(subbed_unskew, unskew);
                        let y_dist_mi1 =
                            y_dist_lo + j1.select(subbed_unskew, unskew);
                        let z_dist_mi1 =
                            z_dist_lo + k1.select(subbed_unskew, unskew);
                        let x_dist_mi2 =
                            x_dist_lo + i2.select(mi2_skew_offset, two_unskew);
                        let y_dist_mi2 =
                            y_dist_lo + j2.select(mi2_skew_offset, two_unskew);
                        let z_dist_mi2 =
                            z_dist_lo + k2.select(mi2_skew_offset, two_unskew);
                        let x_dist_hi = x_dist_lo + hi_skew_offset;
                        let y_dist_hi = y_dist_lo + hi_skew_offset;
                        let z_dist_hi = z_dist_lo + hi_skew_offset;
                        let x1: Simd<u32, A> =
                            x_grid.cast_int_trunc().raw_cast() * channel_seed;
                        let y1: Simd<u32, A> =
                            y_grid.cast_int_trunc().raw_cast() * channel_seed;
                        let z1: Simd<u32, A> =
                            z_grid.cast_int_trunc().raw_cast() * channel_seed;
                        let x2 = x1 + channel_seed;
                        let y2 = y1 + channel_seed;
                        let z2 = z1 + channel_seed;
                        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
                        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
                        let z1_shuf = z1.permute_8(shuffle_indices) ^ prime;
                        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
                        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
                        let z2_shuf = z2.permute_8(shuffle_indices) ^ prime;
                        let x_mi1_shuf = i1.raw_cast().select(x2_shuf, x1_shuf);
                        let y_mi1_shuf = j1.raw_cast().select(y2_shuf, y1_shuf);
                        let z_mi1_shuf = k1.raw_cast().select(z2_shuf, z1_shuf);
                        let x_mi2_shuf = i2.raw_cast().select(x2_shuf, x1_shuf);
                        let y_mi2_shuf = j2.raw_cast().select(y2_shuf, y1_shuf);
                        let z_mi2_shuf = k2.raw_cast().select(z2_shuf, z1_shuf);
                        let mix_lo = x1_shuf * y1_shuf * z1_shuf;
                        let mix_hi = x2_shuf * y2_shuf * z2_shuf;
                        let mix_mi1 = x_mi1_shuf * y_mi1_shuf * z_mi1_shuf;
                        let mix_mi2 = x_mi2_shuf * y_mi2_shuf * z_mi2_shuf;
                        let indices_lo = (mix_lo >> 28) << 1;
                        let indices_mi1 = (mix_mi1 >> 28) << 1;
                        let indices_mi2 = (mix_mi2 >> 28) << 1;
                        let indices_hi = (mix_hi >> 28) << 1;
                        let x_grads_lo =
                            ((c1 >> indices_lo) & three_int).gather(&GRAD_TABLE);
                        let y_grads_lo =
                            ((c2 >> indices_lo) & three_int).gather(&GRAD_TABLE);
                        let z_grads_lo =
                            ((c3 >> indices_lo) & three_int).gather(&GRAD_TABLE);
                        let x_grads_mi1 =
                            ((c1 >> indices_mi1) & three_int).gather(&GRAD_TABLE);
                        let y_grads_mi1 =
                            ((c2 >> indices_mi1) & three_int).gather(&GRAD_TABLE);
                        let z_grads_mi1 =
                            ((c3 >> indices_mi1) & three_int).gather(&GRAD_TABLE);
                        let x_grads_mi2 =
                            ((c1 >> indices_mi2) & three_int).gather(&GRAD_TABLE);
                        let y_grads_mi2 =
                            ((c2 >> indices_mi2) & three_int).gather(&GRAD_TABLE);
                        let z_grads_mi2 =
                            ((c3 >> indices_mi2) & three_int).gather(&GRAD_TABLE);
                        let x_grads_hi =
                            ((c1 >> indices_hi) & three_int).gather(&GRAD_TABLE);
                        let y_grads_hi =
                            ((c2 >> indices_hi) & three_int).gather(&GRAD_TABLE);
                        let z_grads_hi =
                            ((c3 >> indices_hi) & three_int).gather(&GRAD_TABLE);
                        let t_lo =
                            (half -
                                        x_dist_lo.mul_add(x_dist_lo,
                                            y_dist_lo.mul_add(y_dist_lo,
                                                z_dist_lo * z_dist_lo))).max(zero);
                        let t_mi1 =
                            (half -
                                        x_dist_mi1.mul_add(x_dist_mi1,
                                            y_dist_mi1.mul_add(y_dist_mi1,
                                                z_dist_mi1 * z_dist_mi1))).max(zero);
                        let t_mi2 =
                            (half -
                                        x_dist_mi2.mul_add(x_dist_mi2,
                                            y_dist_mi2.mul_add(y_dist_mi2,
                                                z_dist_mi2 * z_dist_mi2))).max(zero);
                        let t_hi =
                            (half -
                                        x_dist_hi.mul_add(x_dist_hi,
                                            y_dist_hi.mul_add(y_dist_hi,
                                                z_dist_hi * z_dist_hi))).max(zero);
                        let t2_lo = t_lo * t_lo;
                        let t2_mi1 = t_mi1 * t_mi1;
                        let t2_mi2 = t_mi2 * t_mi2;
                        let t2_hi = t_hi * t_hi;
                        let t4_lo = t2_lo * t2_lo;
                        let t4_mi1 = t2_mi1 * t2_mi1;
                        let t4_mi2 = t2_mi2 * t2_mi2;
                        let t4_hi = t2_hi * t2_hi;
                        let dot_lo =
                            x_grads_lo.mul_add(x_dist_lo,
                                y_grads_lo.mul_add(y_dist_lo, z_dist_lo * z_grads_lo));
                        let dot_mi1 =
                            x_grads_mi1.mul_add(x_dist_mi1,
                                y_grads_mi1.mul_add(y_dist_mi1, z_dist_mi1 * z_grads_mi1));
                        let dot_mi2 =
                            x_grads_mi2.mul_add(x_dist_mi2,
                                y_grads_mi2.mul_add(y_dist_mi2, z_dist_mi2 * z_grads_mi2));
                        let dot_hi =
                            x_grads_hi.mul_add(x_dist_hi,
                                y_grads_hi.mul_add(y_dist_hi, z_dist_hi * z_grads_hi));
                        t4_lo.mul_add(dot_lo,
                            t4_mi1.mul_add(dot_mi1,
                                t4_mi2.mul_add(dot_mi2, t4_hi * dot_hi)))
                    }
                }
            }
        }
        pub struct Cellular {}
        #[automatically_derived]
        impl ::core::default::Default for Cellular {
            #[inline]
            fn default() -> Cellular { Cellular {} }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Cellular { }
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for Cellular { }
        #[automatically_derived]
        impl ::core::clone::Clone for Cellular {
            #[inline]
            fn clone(&self) -> Cellular { *self }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Cellular { }
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Cellular {
            #[inline]
            fn eq(&self, other: &Cellular) -> bool { true }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Cellular {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "Cellular")
            }
        }
        pub mod cellular {
            pub mod batch_2d {
                use crate::api::batch::interface::BatchGenerator;
                use crate::noise::generators::Cellular;
                use crate::simd::{Arch, Simd};
                impl BatchGenerator<2> for Cellular {
                    fn sample_batch<A: Arch>(seed: u32,
                        input: [Simd<f32, A>; 2], freq: [Simd<f32, A>; 2])
                        -> Simd<f32, A> {
                        let three_halves: Simd<f32, A> = Simd::splat(1.5);
                        let one: Simd<f32, A> = Simd::splat(1.0);
                        let hash_mask: Simd<u32, A> = Simd::splat(0x007FFFFF);
                        let exp_bits: Simd<u32, A> = Simd::splat(0x3F800000);
                        const BYTE_SHUFFLE: [u8; 64] =
                            [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                    2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                    7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                    6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                        let shuffle_indices =
                            Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                        let channel_seed = Simd::splat(seed);
                        let prime = Simd::splat(0x85ebca6b_u32);
                        let x_scaled = input[0] * freq[0];
                        let y_scaled = input[1] * freq[1];
                        let x_grid_lo = x_scaled.floor();
                        let y_grid_lo = y_scaled.floor();
                        let x_dist_lo = x_scaled - x_grid_lo - three_halves;
                        let y_dist_lo = y_scaled - y_grid_lo - three_halves;
                        let x_dist_hi = one - x_dist_lo;
                        let y_dist_hi = one - y_dist_lo;
                        let close_edge_lo =
                            x_dist_lo.min(y_dist_lo) + Simd::splat(2.0);
                        let close_edge_hi = x_dist_hi.min(y_dist_hi) - one;
                        let closest_edge_dist = close_edge_lo.min(close_edge_hi);
                        let threshold = closest_edge_dist * closest_edge_dist;
                        let x1: Simd<u32, A> =
                            x_grid_lo.cast_int_trunc().raw_cast() * channel_seed;
                        let y1: Simd<u32, A> =
                            y_grid_lo.cast_int_trunc().raw_cast() * channel_seed;
                        let x2 = x1 + channel_seed;
                        let y2 = y1 + channel_seed;
                        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
                        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
                        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
                        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
                        let hash_tl = (x1_shuf * y1_shuf) ^ x1_shuf;
                        let hash_tr = (x1_shuf * y2_shuf) ^ x1_shuf;
                        let hash_bl = (x2_shuf * y1_shuf) ^ x2_shuf;
                        let hash_br = (x2_shuf * y2_shuf) ^ x2_shuf;
                        let x_dist_tl =
                            ((hash_tl & hash_mask) | exp_bits).raw_cast::<f32>() +
                                x_dist_lo;
                        let x_dist_tr =
                            ((hash_tr & hash_mask) | exp_bits).raw_cast::<f32>() +
                                x_dist_lo;
                        let x_dist_bl =
                            ((hash_bl & hash_mask) | exp_bits).raw_cast::<f32>() -
                                x_dist_hi;
                        let x_dist_br =
                            ((hash_br & hash_mask) | exp_bits).raw_cast::<f32>() -
                                x_dist_hi;
                        let y_dist_tl =
                            ((hash_tl >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                        let y_dist_tr =
                            ((hash_tr >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                        let y_dist_bl =
                            ((hash_bl >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                        let y_dist_br =
                            ((hash_br >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                        let dist_tl =
                            x_dist_tl.mul_add(x_dist_tl, y_dist_tl * y_dist_tl);
                        let dist_tr =
                            x_dist_tr.mul_add(x_dist_tr, y_dist_tr * y_dist_tr);
                        let dist_bl =
                            x_dist_bl.mul_add(x_dist_bl, y_dist_bl * y_dist_bl);
                        let dist_br =
                            x_dist_br.mul_add(x_dist_br, y_dist_br * y_dist_br);
                        let mut min_dist =
                            dist_tl.min(dist_tr).min(dist_bl).min(dist_br);
                        let is_far = min_dist.simd_gt(threshold);
                        if !is_far.all_false() {
                            let x0 = x1 - channel_seed;
                            let y0 = y1 - channel_seed;
                            let x3 = x2 + channel_seed;
                            let y3 = y2 + channel_seed;
                            let x0_shuf = x0.permute_8(shuffle_indices) ^ prime;
                            let y0_shuf = y0.permute_8(shuffle_indices) ^ prime;
                            let x3_shuf = x3.permute_8(shuffle_indices) ^ prime;
                            let y3_shuf = y3.permute_8(shuffle_indices) ^ prime;
                            let hash_ttl = (x0_shuf * y1_shuf) ^ x0_shuf;
                            let hash_tll = (x1_shuf * y0_shuf) ^ x1_shuf;
                            let hash_ttr = (x0_shuf * y2_shuf) ^ x0_shuf;
                            let hash_trr = (x1_shuf * y3_shuf) ^ x1_shuf;
                            let hash_bbl = (x3_shuf * y1_shuf) ^ x3_shuf;
                            let hash_bll = (x2_shuf * y0_shuf) ^ x2_shuf;
                            let hash_bbr = (x3_shuf * y2_shuf) ^ x3_shuf;
                            let hash_brr = (x2_shuf * y3_shuf) ^ x2_shuf;
                            let x_dist_ttl =
                                ((hash_ttl & hash_mask) | exp_bits).raw_cast::<f32>() +
                                        x_dist_lo + one;
                            let x_dist_tll =
                                ((hash_tll & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_ttr =
                                ((hash_ttr & hash_mask) | exp_bits).raw_cast::<f32>() +
                                        x_dist_lo + one;
                            let x_dist_trr =
                                ((hash_trr & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_bbl =
                                ((hash_bbl & hash_mask) | exp_bits).raw_cast::<f32>() -
                                        x_dist_hi - one;
                            let x_dist_bll =
                                ((hash_bll & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let x_dist_bbr =
                                ((hash_bbr & hash_mask) | exp_bits).raw_cast::<f32>() -
                                        x_dist_hi - one;
                            let x_dist_brr =
                                ((hash_brr & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let y_dist_ttl =
                                ((hash_ttl >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_tll =
                                ((hash_tll >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo +
                                    one;
                            let y_dist_ttr =
                                ((hash_ttr >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_trr =
                                ((hash_trr >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi -
                                    one;
                            let y_dist_bbl =
                                ((hash_bbl >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_bll =
                                ((hash_bll >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo +
                                    one;
                            let y_dist_bbr =
                                ((hash_bbr >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_brr =
                                ((hash_brr >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi -
                                    one;
                            let dist_ttl =
                                x_dist_ttl.mul_add(x_dist_ttl, y_dist_ttl * y_dist_ttl);
                            let dist_tll =
                                x_dist_tll.mul_add(x_dist_tll, y_dist_tll * y_dist_tll);
                            let dist_ttr =
                                x_dist_ttr.mul_add(x_dist_ttr, y_dist_ttr * y_dist_ttr);
                            let dist_trr =
                                x_dist_trr.mul_add(x_dist_trr, y_dist_trr * y_dist_trr);
                            let dist_bbl =
                                x_dist_bbl.mul_add(x_dist_bbl, y_dist_bbl * y_dist_bbl);
                            let dist_bll =
                                x_dist_bll.mul_add(x_dist_bll, y_dist_bll * y_dist_bll);
                            let dist_bbr =
                                x_dist_bbr.mul_add(x_dist_bbr, y_dist_bbr * y_dist_bbr);
                            let dist_brr =
                                x_dist_brr.mul_add(x_dist_brr, y_dist_brr * y_dist_brr);
                            let outer_min =
                                dist_ttl.min(dist_tll).min(dist_ttr).min(dist_trr).min(dist_bbl).min(dist_bll).min(dist_bbr).min(dist_brr);
                            min_dist = min_dist.min(outer_min);
                        }
                        min_dist.sqrt()
                    }
                }
            }
            pub mod batch_3d {
                use crate::api::batch::interface::BatchGenerator;
                use crate::noise::generators::Cellular;
                use crate::simd::{Arch, Simd};
                impl BatchGenerator<3> for Cellular {
                    fn sample_batch<A: Arch>(seed: u32,
                        input: [Simd<f32, A>; 3], freq: [Simd<f32, A>; 3])
                        -> Simd<f32, A> {
                        let three_halves: Simd<f32, A> = Simd::splat(1.5);
                        let one: Simd<f32, A> = Simd::splat(1.0);
                        let hash_mask: Simd<u32, A> = Simd::splat(0x007FFFFF);
                        let exp_bits: Simd<u32, A> = Simd::splat(0x3F800000);
                        const BYTE_SHUFFLE: [u8; 64] =
                            [3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0,
                                    2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1,
                                    7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4,
                                    6, 5, 11, 8, 10, 9, 15, 12, 14, 13];
                        let shuffle_indices =
                            Simd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
                        let channel_seed = Simd::splat(seed);
                        let prime = Simd::splat(0x85ebca6b_u32);
                        let x_scaled = input[0] * freq[0];
                        let y_scaled = input[1] * freq[1];
                        let z_scaled = input[2] * freq[2];
                        let x_grid_lo = x_scaled.floor();
                        let y_grid_lo = y_scaled.floor();
                        let z_grid_lo = z_scaled.floor();
                        let x_dist_lo = x_scaled - x_grid_lo - three_halves;
                        let y_dist_lo = y_scaled - y_grid_lo - three_halves;
                        let z_dist_lo = z_scaled - z_grid_lo - three_halves;
                        let x_dist_hi = one - x_dist_lo;
                        let y_dist_hi = one - y_dist_lo;
                        let z_dist_hi = one - z_dist_lo;
                        let close_edge_lo =
                            x_dist_lo.min(y_dist_lo).min(z_dist_lo) + Simd::splat(2.0);
                        let close_edge_hi =
                            x_dist_hi.min(y_dist_hi).min(z_dist_hi) - one;
                        let closest_edge_dist = close_edge_lo.min(close_edge_hi);
                        let threshold = closest_edge_dist * closest_edge_dist;
                        let x1: Simd<u32, A> =
                            x_grid_lo.cast_int_trunc().raw_cast() * channel_seed;
                        let y1: Simd<u32, A> =
                            y_grid_lo.cast_int_trunc().raw_cast() * channel_seed;
                        let z1: Simd<u32, A> =
                            z_grid_lo.cast_int_trunc().raw_cast() * channel_seed;
                        let x2 = x1 + channel_seed;
                        let y2 = y1 + channel_seed;
                        let z2 = z1 + channel_seed;
                        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
                        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
                        let z1_shuf = z1.permute_8(shuffle_indices) ^ prime;
                        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
                        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
                        let z2_shuf = z2.permute_8(shuffle_indices) ^ prime;
                        let hash_tlf = x1_shuf * y1_shuf * z1_shuf;
                        let hash_trf = x1_shuf * y1_shuf * z2_shuf;
                        let hash_blf = x1_shuf * y2_shuf * z1_shuf;
                        let hash_brf = x1_shuf * y2_shuf * z2_shuf;
                        let hash_tlb = x2_shuf * y1_shuf * z1_shuf;
                        let hash_trb = x2_shuf * y1_shuf * z2_shuf;
                        let hash_blb = x2_shuf * y2_shuf * z1_shuf;
                        let hash_brb = x2_shuf * y2_shuf * z2_shuf;
                        let x_dist_tlf =
                            ((hash_tlf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                x_dist_lo;
                        let x_dist_trf =
                            ((hash_trf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                x_dist_lo;
                        let x_dist_blf =
                            ((hash_blf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                x_dist_lo;
                        let x_dist_brf =
                            ((hash_brf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                x_dist_lo;
                        let x_dist_tlb =
                            ((hash_tlb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                x_dist_hi;
                        let x_dist_trb =
                            ((hash_trb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                x_dist_hi;
                        let x_dist_blb =
                            ((hash_blb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                x_dist_hi;
                        let x_dist_brb =
                            ((hash_brb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                x_dist_hi;
                        let y_dist_tlf =
                            ((hash_tlf >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                        let y_dist_trf =
                            ((hash_trf >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                        let y_dist_blf =
                            ((hash_blf >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                        let y_dist_brf =
                            ((hash_brf >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                        let y_dist_tlb =
                            ((hash_tlb >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                        let y_dist_trb =
                            ((hash_trb >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                        let y_dist_blb =
                            ((hash_blb >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                        let y_dist_brb =
                            ((hash_brb >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                        let z_dist_tlf =
                            (((hash_tlf << 12) & hash_mask) |
                                            exp_bits).raw_cast::<f32>() + z_dist_lo;
                        let z_dist_trf =
                            (((hash_trf << 12) & hash_mask) |
                                            exp_bits).raw_cast::<f32>() - z_dist_hi;
                        let z_dist_blf =
                            (((hash_blf << 12) & hash_mask) |
                                            exp_bits).raw_cast::<f32>() + z_dist_lo;
                        let z_dist_brf =
                            (((hash_brf << 12) & hash_mask) |
                                            exp_bits).raw_cast::<f32>() - z_dist_hi;
                        let z_dist_tlb =
                            (((hash_tlb << 12) & hash_mask) |
                                            exp_bits).raw_cast::<f32>() + z_dist_lo;
                        let z_dist_trb =
                            (((hash_trb << 12) & hash_mask) |
                                            exp_bits).raw_cast::<f32>() - z_dist_hi;
                        let z_dist_blb =
                            (((hash_blb << 12) & hash_mask) |
                                            exp_bits).raw_cast::<f32>() + z_dist_lo;
                        let z_dist_brb =
                            (((hash_brb << 12) & hash_mask) |
                                            exp_bits).raw_cast::<f32>() - z_dist_hi;
                        let dist_tlf =
                            x_dist_tlf.mul_add(x_dist_tlf,
                                y_dist_tlf.mul_add(y_dist_tlf, z_dist_tlf * z_dist_tlf));
                        let dist_trf =
                            x_dist_trf.mul_add(x_dist_trf,
                                y_dist_trf.mul_add(y_dist_trf, z_dist_trf * z_dist_trf));
                        let dist_blf =
                            x_dist_blf.mul_add(x_dist_blf,
                                y_dist_blf.mul_add(y_dist_blf, z_dist_blf * z_dist_blf));
                        let dist_brf =
                            x_dist_brf.mul_add(x_dist_brf,
                                y_dist_brf.mul_add(y_dist_brf, z_dist_brf * z_dist_brf));
                        let dist_tlb =
                            x_dist_tlb.mul_add(x_dist_tlb,
                                y_dist_tlb.mul_add(y_dist_tlb, z_dist_tlb * z_dist_tlb));
                        let dist_trb =
                            x_dist_trb.mul_add(x_dist_trb,
                                y_dist_trb.mul_add(y_dist_trb, z_dist_trb * z_dist_trb));
                        let dist_blb =
                            x_dist_blb.mul_add(x_dist_blb,
                                y_dist_blb.mul_add(y_dist_blb, z_dist_blb * z_dist_blb));
                        let dist_brb =
                            x_dist_brb.mul_add(x_dist_brb,
                                y_dist_brb.mul_add(y_dist_brb, z_dist_brb * z_dist_brb));
                        let mut min_dist =
                            dist_tlf.min(dist_trf).min(dist_blf).min(dist_brf).min(dist_tlb).min(dist_trb).min(dist_blb).min(dist_brb);
                        let is_far = min_dist.simd_gt(threshold);
                        if !is_far.all_false() {
                            let x0 = x1 - channel_seed;
                            let y0 = y1 - channel_seed;
                            let z0 = z1 - channel_seed;
                            let x3 = x2 + channel_seed;
                            let y3 = y2 + channel_seed;
                            let z3 = z2 + channel_seed;
                            let x0_shuf = x0.permute_8(shuffle_indices) ^ prime;
                            let y0_shuf = y0.permute_8(shuffle_indices) ^ prime;
                            let z0_shuf = z0.permute_8(shuffle_indices) ^ prime;
                            let x3_shuf = x3.permute_8(shuffle_indices) ^ prime;
                            let y3_shuf = y3.permute_8(shuffle_indices) ^ prime;
                            let z3_shuf = z3.permute_8(shuffle_indices) ^ prime;
                            let hash_tlff = x0_shuf * y1_shuf * z1_shuf;
                            let hash_ttlf = x1_shuf * y0_shuf * z1_shuf;
                            let hash_tllf = x1_shuf * y1_shuf * z0_shuf;
                            let hash_trff = x0_shuf * y1_shuf * z2_shuf;
                            let hash_ttrf = x1_shuf * y0_shuf * z2_shuf;
                            let hash_trrf = x1_shuf * y1_shuf * z3_shuf;
                            let hash_blff = x0_shuf * y2_shuf * z1_shuf;
                            let hash_bblf = x1_shuf * y3_shuf * z1_shuf;
                            let hash_bllf = x1_shuf * y2_shuf * z0_shuf;
                            let hash_brff = x0_shuf * y2_shuf * z2_shuf;
                            let hash_bbrf = x1_shuf * y3_shuf * z2_shuf;
                            let hash_brrf = x1_shuf * y2_shuf * z3_shuf;
                            let hash_tlbb = x3_shuf * y1_shuf * z1_shuf;
                            let hash_ttlb = x2_shuf * y0_shuf * z1_shuf;
                            let hash_tllb = x2_shuf * y1_shuf * z0_shuf;
                            let hash_trbb = x3_shuf * y1_shuf * z2_shuf;
                            let hash_ttrb = x2_shuf * y0_shuf * z2_shuf;
                            let hash_trrb = x2_shuf * y1_shuf * z3_shuf;
                            let hash_blbb = x3_shuf * y2_shuf * z1_shuf;
                            let hash_bblb = x2_shuf * y3_shuf * z1_shuf;
                            let hash_bllb = x2_shuf * y2_shuf * z0_shuf;
                            let hash_brbb = x3_shuf * y2_shuf * z2_shuf;
                            let hash_bbrb = x2_shuf * y3_shuf * z2_shuf;
                            let hash_brrb = x2_shuf * y2_shuf * z3_shuf;
                            let x_dist_tlff =
                                ((hash_tlff & hash_mask) | exp_bits).raw_cast::<f32>() +
                                        x_dist_lo + one;
                            let x_dist_ttlf =
                                ((hash_ttlf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_tllf =
                                ((hash_tllf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_trff =
                                ((hash_trff & hash_mask) | exp_bits).raw_cast::<f32>() +
                                        x_dist_lo + one;
                            let x_dist_ttrf =
                                ((hash_ttrf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_trrf =
                                ((hash_trrf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_blff =
                                ((hash_blff & hash_mask) | exp_bits).raw_cast::<f32>() +
                                        x_dist_lo + one;
                            let x_dist_bblf =
                                ((hash_bblf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_bllf =
                                ((hash_bllf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_brff =
                                ((hash_brff & hash_mask) | exp_bits).raw_cast::<f32>() +
                                        x_dist_lo + one;
                            let x_dist_bbrf =
                                ((hash_bbrf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_brrf =
                                ((hash_brrf & hash_mask) | exp_bits).raw_cast::<f32>() +
                                    x_dist_lo;
                            let x_dist_tlbb =
                                ((hash_tlbb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                        x_dist_hi - one;
                            let x_dist_ttlb =
                                ((hash_ttlb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let x_dist_tllb =
                                ((hash_tllb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let x_dist_trbb =
                                ((hash_trbb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                        x_dist_hi - one;
                            let x_dist_ttrb =
                                ((hash_ttrb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let x_dist_trrb =
                                ((hash_trrb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let x_dist_blbb =
                                ((hash_blbb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                        x_dist_hi - one;
                            let x_dist_bblb =
                                ((hash_bblb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let x_dist_bllb =
                                ((hash_bllb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let x_dist_brbb =
                                ((hash_brbb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                        x_dist_hi - one;
                            let x_dist_bbrb =
                                ((hash_bbrb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let x_dist_brrb =
                                ((hash_brrb & hash_mask) | exp_bits).raw_cast::<f32>() -
                                    x_dist_hi;
                            let y_dist_tlff =
                                ((hash_tlff >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_ttlf =
                                ((hash_ttlf >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo
                                    + one;
                            let y_dist_tllf =
                                ((hash_tllf >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_trff =
                                ((hash_trff >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_ttrf =
                                ((hash_ttrf >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo
                                    + one;
                            let y_dist_trrf =
                                ((hash_trrf >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_blff =
                                ((hash_blff >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_bblf =
                                ((hash_bblf >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi
                                    - one;
                            let y_dist_bllf =
                                ((hash_bllf >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_brff =
                                ((hash_brff >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_bbrf =
                                ((hash_bbrf >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi
                                    - one;
                            let y_dist_brrf =
                                ((hash_brrf >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_tlbb =
                                ((hash_tlbb >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_ttlb =
                                ((hash_ttlb >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo
                                    + one;
                            let y_dist_tllb =
                                ((hash_tllb >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_trbb =
                                ((hash_trbb >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_ttrb =
                                ((hash_ttrb >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo
                                    + one;
                            let y_dist_trrb =
                                ((hash_trrb >> 9) | exp_bits).raw_cast::<f32>() + y_dist_lo;
                            let y_dist_blbb =
                                ((hash_blbb >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_bblb =
                                ((hash_bblb >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi
                                    - one;
                            let y_dist_bllb =
                                ((hash_bllb >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_brbb =
                                ((hash_brbb >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let y_dist_bbrb =
                                ((hash_bbrb >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi
                                    - one;
                            let y_dist_brrb =
                                ((hash_brrb >> 9) | exp_bits).raw_cast::<f32>() - y_dist_hi;
                            let z_dist_tlff =
                                (((hash_tlff << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() + z_dist_lo;
                            let z_dist_ttlf =
                                (((hash_ttlf << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() + z_dist_lo;
                            let z_dist_tllf =
                                (((hash_tllf << 12) & hash_mask) |
                                                    exp_bits).raw_cast::<f32>() + z_dist_lo + one;
                            let z_dist_trff =
                                (((hash_trff << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() - z_dist_hi;
                            let z_dist_ttrf =
                                (((hash_ttrf << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() - z_dist_hi;
                            let z_dist_trrf =
                                (((hash_trrf << 12) & hash_mask) |
                                                    exp_bits).raw_cast::<f32>() - z_dist_hi - one;
                            let z_dist_blff =
                                (((hash_blff << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() + z_dist_lo;
                            let z_dist_bblf =
                                (((hash_bblf << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() + z_dist_lo;
                            let z_dist_bllf =
                                (((hash_bllf << 12) & hash_mask) |
                                                    exp_bits).raw_cast::<f32>() + z_dist_lo + one;
                            let z_dist_brff =
                                (((hash_brff << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() - z_dist_hi;
                            let z_dist_bbrf =
                                (((hash_bbrf << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() - z_dist_hi;
                            let z_dist_brrf =
                                (((hash_brrf << 12) & hash_mask) |
                                                    exp_bits).raw_cast::<f32>() - z_dist_hi - one;
                            let z_dist_tlbb =
                                (((hash_tlbb << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() + z_dist_lo;
                            let z_dist_ttlb =
                                (((hash_ttlb << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() + z_dist_lo;
                            let z_dist_tllb =
                                (((hash_tllb << 12) & hash_mask) |
                                                    exp_bits).raw_cast::<f32>() + z_dist_lo + one;
                            let z_dist_trbb =
                                (((hash_trbb << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() - z_dist_hi;
                            let z_dist_ttrb =
                                (((hash_ttrb << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() - z_dist_hi;
                            let z_dist_trrb =
                                (((hash_trrb << 12) & hash_mask) |
                                                    exp_bits).raw_cast::<f32>() - z_dist_hi - one;
                            let z_dist_blbb =
                                (((hash_blbb << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() + z_dist_lo;
                            let z_dist_bblb =
                                (((hash_bblb << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() + z_dist_lo;
                            let z_dist_bllb =
                                (((hash_bllb << 12) & hash_mask) |
                                                    exp_bits).raw_cast::<f32>() + z_dist_lo + one;
                            let z_dist_brbb =
                                (((hash_brbb << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() - z_dist_hi;
                            let z_dist_bbrb =
                                (((hash_bbrb << 12) & hash_mask) |
                                                exp_bits).raw_cast::<f32>() - z_dist_hi;
                            let z_dist_brrb =
                                (((hash_brrb << 12) & hash_mask) |
                                                    exp_bits).raw_cast::<f32>() - z_dist_hi - one;
                            let dist_tlff =
                                x_dist_tlff.mul_add(x_dist_tlff,
                                    y_dist_tlff.mul_add(y_dist_tlff,
                                        z_dist_tlff * z_dist_tlff));
                            let dist_ttlf =
                                x_dist_ttlf.mul_add(x_dist_ttlf,
                                    y_dist_ttlf.mul_add(y_dist_ttlf,
                                        z_dist_ttlf * z_dist_ttlf));
                            let dist_tllf =
                                x_dist_tllf.mul_add(x_dist_tllf,
                                    y_dist_tllf.mul_add(y_dist_tllf,
                                        z_dist_tllf * z_dist_tllf));
                            let dist_trff =
                                x_dist_trff.mul_add(x_dist_trff,
                                    y_dist_trff.mul_add(y_dist_trff,
                                        z_dist_trff * z_dist_trff));
                            let dist_ttrf =
                                x_dist_ttrf.mul_add(x_dist_ttrf,
                                    y_dist_ttrf.mul_add(y_dist_ttrf,
                                        z_dist_ttrf * z_dist_ttrf));
                            let dist_trrf =
                                x_dist_trrf.mul_add(x_dist_trrf,
                                    y_dist_trrf.mul_add(y_dist_trrf,
                                        z_dist_trrf * z_dist_trrf));
                            let dist_blff =
                                x_dist_blff.mul_add(x_dist_blff,
                                    y_dist_blff.mul_add(y_dist_blff,
                                        z_dist_blff * z_dist_blff));
                            let dist_bblf =
                                x_dist_bblf.mul_add(x_dist_bblf,
                                    y_dist_bblf.mul_add(y_dist_bblf,
                                        z_dist_bblf * z_dist_bblf));
                            let dist_bllf =
                                x_dist_bllf.mul_add(x_dist_bllf,
                                    y_dist_bllf.mul_add(y_dist_bllf,
                                        z_dist_bllf * z_dist_bllf));
                            let dist_brff =
                                x_dist_brff.mul_add(x_dist_brff,
                                    y_dist_brff.mul_add(y_dist_brff,
                                        z_dist_brff * z_dist_brff));
                            let dist_bbrf =
                                x_dist_bbrf.mul_add(x_dist_bbrf,
                                    y_dist_bbrf.mul_add(y_dist_bbrf,
                                        z_dist_bbrf * z_dist_bbrf));
                            let dist_brrf =
                                x_dist_brrf.mul_add(x_dist_brrf,
                                    y_dist_brrf.mul_add(y_dist_brrf,
                                        z_dist_brrf * z_dist_brrf));
                            let dist_tlbb =
                                x_dist_tlbb.mul_add(x_dist_tlbb,
                                    y_dist_tlbb.mul_add(y_dist_tlbb,
                                        z_dist_tlbb * z_dist_tlbb));
                            let dist_ttlb =
                                x_dist_ttlb.mul_add(x_dist_ttlb,
                                    y_dist_ttlb.mul_add(y_dist_ttlb,
                                        z_dist_ttlb * z_dist_ttlb));
                            let dist_tllb =
                                x_dist_tllb.mul_add(x_dist_tllb,
                                    y_dist_tllb.mul_add(y_dist_tllb,
                                        z_dist_tllb * z_dist_tllb));
                            let dist_trbb =
                                x_dist_trbb.mul_add(x_dist_trbb,
                                    y_dist_trbb.mul_add(y_dist_trbb,
                                        z_dist_trbb * z_dist_trbb));
                            let dist_ttrb =
                                x_dist_ttrb.mul_add(x_dist_ttrb,
                                    y_dist_ttrb.mul_add(y_dist_ttrb,
                                        z_dist_ttrb * z_dist_ttrb));
                            let dist_trrb =
                                x_dist_trrb.mul_add(x_dist_trrb,
                                    y_dist_trrb.mul_add(y_dist_trrb,
                                        z_dist_trrb * z_dist_trrb));
                            let dist_blbb =
                                x_dist_blbb.mul_add(x_dist_blbb,
                                    y_dist_blbb.mul_add(y_dist_blbb,
                                        z_dist_blbb * z_dist_blbb));
                            let dist_bblb =
                                x_dist_bblb.mul_add(x_dist_bblb,
                                    y_dist_bblb.mul_add(y_dist_bblb,
                                        z_dist_bblb * z_dist_bblb));
                            let dist_bllb =
                                x_dist_bllb.mul_add(x_dist_bllb,
                                    y_dist_bllb.mul_add(y_dist_bllb,
                                        z_dist_bllb * z_dist_bllb));
                            let dist_brbb =
                                x_dist_brbb.mul_add(x_dist_brbb,
                                    y_dist_brbb.mul_add(y_dist_brbb,
                                        z_dist_brbb * z_dist_brbb));
                            let dist_bbrb =
                                x_dist_bbrb.mul_add(x_dist_bbrb,
                                    y_dist_bbrb.mul_add(y_dist_bbrb,
                                        z_dist_bbrb * z_dist_bbrb));
                            let dist_brrb =
                                x_dist_brrb.mul_add(x_dist_brrb,
                                    y_dist_brrb.mul_add(y_dist_brrb,
                                        z_dist_brrb * z_dist_brrb));
                            let outer_min =
                                dist_tlff.min(dist_ttlf).min(dist_tllf).min(dist_trff).min(dist_ttrf).min(dist_trrf).min(dist_blff).min(dist_bblf).min(dist_bllf).min(dist_brff).min(dist_bbrf).min(dist_brrf).min(dist_tlbb).min(dist_ttlb).min(dist_tllb).min(dist_trbb).min(dist_ttrb).min(dist_trrb).min(dist_blbb).min(dist_bblb).min(dist_bllb).min(dist_brbb).min(dist_bbrb).min(dist_brrb);
                            min_dist = min_dist.min(outer_min);
                        }
                        min_dist.sqrt()
                    }
                }
            }
        }
    }
    pub mod combiners {
        use std::ops::{Index, IndexMut};
        use crate::simd::Arch;
        use crate::simd::register::Simd;
        pub mod billow {
            use crate::simd::Arch;
            use crate::simd::Simd;
            use crate::{Combiner, CombinerArray};
            pub struct Billow {}
            #[automatically_derived]
            impl ::core::default::Default for Billow {
                #[inline]
                fn default() -> Billow { Billow {} }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Billow { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Billow { }
            #[automatically_derived]
            impl ::core::clone::Clone for Billow {
                #[inline]
                fn clone(&self) -> Billow { *self }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for Billow { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for Billow {
                #[inline]
                fn eq(&self, other: &Billow) -> bool { true }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Billow {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(f, "Billow")
                }
            }
            impl Combiner for Billow {
                const WEIGHT_DECAY: bool = true;
                type State<A: Arch> = CombinerArray<A, 0>;
                type Config = ();
                #[inline(always)]
                fn apply_sample<A: Arch>(_config: &(), state: Self::State<A>,
                    cur_result: Simd<f32, A>, new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    (state, cur_result + new_sample.abs())
                }
                #[inline(always)]
                fn initialize_sample<A: Arch>(_config: &(),
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    (Self::State::default(), new_sample.abs())
                }
                #[inline(always)]
                fn finalize_sample<A: Arch>(_config: &(),
                    _state: Self::State<A>, last: Simd<f32, A>)
                    -> Simd<f32, A> {
                    last
                }
            }
        }
        pub mod fbm {
            use crate::combiners::{Combiner, CombinerArray};
            use crate::simd::{Arch, Simd};
            pub struct Fbm {}
            #[automatically_derived]
            impl ::core::default::Default for Fbm {
                #[inline]
                fn default() -> Fbm { Fbm {} }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Fbm { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Fbm { }
            #[automatically_derived]
            impl ::core::clone::Clone for Fbm {
                #[inline]
                fn clone(&self) -> Fbm { *self }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for Fbm { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for Fbm {
                #[inline]
                fn eq(&self, other: &Fbm) -> bool { true }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Fbm {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(f, "Fbm")
                }
            }
            impl Combiner for Fbm {
                const WEIGHT_DECAY: bool = true;
                type State<A: Arch> = CombinerArray<A, 0>;
                type Config = ();
                #[inline(always)]
                fn apply_sample<A: Arch>(_config: &(), state: Self::State<A>,
                    cur_result: Simd<f32, A>, new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    (state, cur_result + new_sample)
                }
                #[inline(always)]
                fn initialize_sample<A: Arch>(_config: &(),
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    (Default::default(), new_sample)
                }
                #[inline(always)]
                fn finalize_sample<A: Arch>(_config: &(),
                    _state: Self::State<A>, last: Simd<f32, A>)
                    -> Simd<f32, A> {
                    last
                }
            }
        }
        pub mod hybrid_multi {
            use crate::simd::{Arch, Simd};
            use crate::{Combiner, CombinerArray};
            pub struct HybridMultiConfig {
                pub gain: f32,
                pub offset: f32,
            }
            #[automatically_derived]
            impl ::core::marker::Copy for HybridMultiConfig { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for HybridMultiConfig { }
            #[automatically_derived]
            impl ::core::clone::Clone for HybridMultiConfig {
                #[inline]
                fn clone(&self) -> HybridMultiConfig {
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for HybridMultiConfig { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for HybridMultiConfig {
                #[inline]
                fn eq(&self, other: &HybridMultiConfig) -> bool {
                    self.gain == other.gain && self.offset == other.offset
                }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for HybridMultiConfig {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field2_finish(f,
                        "HybridMultiConfig", "gain", &self.gain, "offset",
                        &&self.offset)
                }
            }
            impl Default for HybridMultiConfig {
                fn default() -> Self { Self { gain: 2.0, offset: 1.0 } }
            }
            pub struct HybridMulti {}
            #[automatically_derived]
            impl ::core::default::Default for HybridMulti {
                #[inline]
                fn default() -> HybridMulti { HybridMulti {} }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for HybridMulti { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for HybridMulti { }
            #[automatically_derived]
            impl ::core::clone::Clone for HybridMulti {
                #[inline]
                fn clone(&self) -> HybridMulti { *self }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for HybridMulti { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for HybridMulti {
                #[inline]
                fn eq(&self, other: &HybridMulti) -> bool { true }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for HybridMulti {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(f, "HybridMulti")
                }
            }
            impl Combiner for HybridMulti {
                const WEIGHT_DECAY: bool = false;
                type State<A: Arch> = CombinerArray<A, 1>;
                type Config = HybridMultiConfig;
                #[inline(always)]
                fn apply_sample<A: Arch>(config: &HybridMultiConfig,
                    state: Self::State<A>, cur_result: Simd<f32, A>,
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    let zero = Simd::splat(0.0);
                    let one = Simd::splat(1.0);
                    let signal = new_sample + Simd::splat(config.offset);
                    let weighted_signal = state[0] * signal;
                    let result = cur_result + weighted_signal;
                    let weight =
                        (weighted_signal *
                                    Simd::splat(config.gain)).clamp(zero, one);
                    ([weight], result)
                }
                #[inline(always)]
                fn initialize_sample<A: Arch>(config: &HybridMultiConfig,
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    let zero = Simd::splat(0.0);
                    let one = Simd::splat(1.0);
                    let signal = new_sample + Simd::splat(config.offset);
                    let weight =
                        (signal * Simd::splat(config.gain)).clamp(zero, one);
                    ([weight], signal)
                }
            }
        }
        pub mod multi {
            use crate::combiners::{Combiner, CombinerArray};
            use crate::simd::{Arch, Simd};
            pub struct Multi {}
            #[automatically_derived]
            impl ::core::default::Default for Multi {
                #[inline]
                fn default() -> Multi { Multi {} }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Multi { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Multi { }
            #[automatically_derived]
            impl ::core::clone::Clone for Multi {
                #[inline]
                fn clone(&self) -> Multi { *self }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for Multi { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for Multi {
                #[inline]
                fn eq(&self, other: &Multi) -> bool { true }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Multi {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(f, "Multi")
                }
            }
            impl Combiner for Multi {
                const WEIGHT_DECAY: bool = false;
                type State<A: Arch> = CombinerArray<A, 0>;
                type Config = ();
                #[inline(always)]
                fn apply_sample<A: Arch>(_config: &(), state: Self::State<A>,
                    cur_result: Simd<f32, A>, new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    (state, cur_result * (new_sample + Simd::splat(1.0)))
                }
                #[inline(always)]
                fn initialize_sample<A: Arch>(_config: &(),
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    (Default::default(), new_sample + Simd::splat(1.0))
                }
                #[inline(always)]
                fn finalize_sample<A: Arch>(_config: &(),
                    _state: Self::State<A>, last: Simd<f32, A>)
                    -> Simd<f32, A> {
                    last
                }
            }
        }
        pub mod ping_pong {
            use crate::combiners::{Combiner, CombinerArray};
            use crate::simd::{Arch, Simd};
            pub struct PingPongConfig {
                pub strength: f32,
            }
            #[automatically_derived]
            impl ::core::marker::Copy for PingPongConfig { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for PingPongConfig { }
            #[automatically_derived]
            impl ::core::clone::Clone for PingPongConfig {
                #[inline]
                fn clone(&self) -> PingPongConfig {
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for PingPongConfig { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for PingPongConfig {
                #[inline]
                fn eq(&self, other: &PingPongConfig) -> bool {
                    self.strength == other.strength
                }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for PingPongConfig {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field1_finish(f,
                        "PingPongConfig", "strength", &&self.strength)
                }
            }
            impl Default for PingPongConfig {
                fn default() -> Self { Self { strength: 2.0 } }
            }
            pub struct PingPong {}
            #[automatically_derived]
            impl ::core::default::Default for PingPong {
                #[inline]
                fn default() -> PingPong { PingPong {} }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for PingPong { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for PingPong { }
            #[automatically_derived]
            impl ::core::clone::Clone for PingPong {
                #[inline]
                fn clone(&self) -> PingPong { *self }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for PingPong { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for PingPong {
                #[inline]
                fn eq(&self, other: &PingPong) -> bool { true }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for PingPong {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(f, "PingPong")
                }
            }
            impl Combiner for PingPong {
                const WEIGHT_DECAY: bool = true;
                type State<A: Arch> = CombinerArray<A, 0>;
                type Config = PingPongConfig;
                #[inline(always)]
                fn apply_sample<A: Arch>(config: &PingPongConfig,
                    state: Self::State<A>, cur_result: Simd<f32, A>,
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    let one = Simd::splat(1.0);
                    let two = Simd::splat(2.0);
                    let t =
                        (cur_result + new_sample) * Simd::splat(config.strength);
                    let sawtooth = t - (t * Simd::splat(0.5)).floor() * two;
                    let folded = one - (sawtooth - one).abs();
                    (state, folded)
                }
            }
        }
        pub mod ridged {
            use crate::{Combiner, CombinerArray, simd::{Arch, Simd}};
            pub struct RidgedConfig {
                pub gain: f32,
            }
            #[automatically_derived]
            impl ::core::marker::Copy for RidgedConfig { }
            #[automatically_derived]
            impl ::core::fmt::Debug for RidgedConfig {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field1_finish(f,
                        "RidgedConfig", "gain", &&self.gain)
                }
            }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for RidgedConfig { }
            #[automatically_derived]
            impl ::core::clone::Clone for RidgedConfig {
                #[inline]
                fn clone(&self) -> RidgedConfig {
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for RidgedConfig { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for RidgedConfig {
                #[inline]
                fn eq(&self, other: &RidgedConfig) -> bool {
                    self.gain == other.gain
                }
            }
            impl Default for RidgedConfig {
                fn default() -> Self { Self { gain: 2.0 } }
            }
            pub struct Ridged {}
            #[automatically_derived]
            impl ::core::default::Default for Ridged {
                #[inline]
                fn default() -> Ridged { Ridged {} }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Ridged { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Ridged { }
            #[automatically_derived]
            impl ::core::clone::Clone for Ridged {
                #[inline]
                fn clone(&self) -> Ridged { *self }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for Ridged { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for Ridged {
                #[inline]
                fn eq(&self, other: &Ridged) -> bool { true }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Ridged {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(f, "Ridged")
                }
            }
            impl Combiner for Ridged {
                const WEIGHT_DECAY: bool = false;
                type State<A: Arch> = CombinerArray<A, 1>;
                type Config = RidgedConfig;
                #[inline(always)]
                fn apply_sample<A: Arch>(config: &RidgedConfig,
                    state: Self::State<A>, cur_result: Simd<f32, A>,
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    let one = Simd::splat(1.0);
                    let gain = Simd::splat(config.gain);
                    let zero = Simd::splat(0.0);
                    let weight = state[0];
                    let signal = one - new_sample.abs();
                    let signal = signal * signal * weight;
                    let next_weight = (signal * gain).clamp(zero, one);
                    let mut next_state = state;
                    next_state[0] = next_weight;
                    (next_state, cur_result + signal)
                }
                #[inline(always)]
                fn initialize_sample<A: Arch>(_config: &RidgedConfig,
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    let one = Simd::splat(1.0);
                    let signal = one - new_sample.abs();
                    let signal = signal * signal;
                    let mut state = Default::default();
                    state[0] = signal;
                    (state, signal)
                }
                #[inline(always)]
                fn finalize_sample<A: Arch>(_config: &RidgedConfig,
                    _state: Self::State<A>, last: Simd<f32, A>)
                    -> Simd<f32, A> {
                    last
                }
            }
        }
        pub mod terrace {
            use crate::{
                combiners::{Combiner, CombinerArray},
                simd::{Arch, Simd},
            };
            pub struct TerraceConfig {
                pub steps: f32,
                pub step_size: f32,
            }
            #[automatically_derived]
            impl ::core::marker::Copy for TerraceConfig { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for TerraceConfig { }
            #[automatically_derived]
            impl ::core::clone::Clone for TerraceConfig {
                #[inline]
                fn clone(&self) -> TerraceConfig {
                    let _: ::core::clone::AssertParamIsClone<f32>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for TerraceConfig { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for TerraceConfig {
                #[inline]
                fn eq(&self, other: &TerraceConfig) -> bool {
                    self.steps == other.steps &&
                        self.step_size == other.step_size
                }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for TerraceConfig {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field2_finish(f,
                        "TerraceConfig", "steps", &self.steps, "step_size",
                        &&self.step_size)
                }
            }
            impl Default for TerraceConfig {
                fn default() -> Self {
                    Self { steps: 8.0, step_size: 1.0 / 8.0 }
                }
            }
            pub struct Terrace {}
            #[automatically_derived]
            impl ::core::default::Default for Terrace {
                #[inline]
                fn default() -> Terrace { Terrace {} }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Terrace { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Terrace { }
            #[automatically_derived]
            impl ::core::clone::Clone for Terrace {
                #[inline]
                fn clone(&self) -> Terrace { *self }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for Terrace { }
            #[automatically_derived]
            impl ::core::cmp::PartialEq for Terrace {
                #[inline]
                fn eq(&self, other: &Terrace) -> bool { true }
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Terrace {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                    -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(f, "Terrace")
                }
            }
            impl Combiner for Terrace {
                const WEIGHT_DECAY: bool = true;
                type State<A: Arch> = CombinerArray<A, 0>;
                type Config = TerraceConfig;
                #[inline(always)]
                fn apply_sample<A: Arch>(_config: &TerraceConfig,
                    state: Self::State<A>, cur_result: Simd<f32, A>,
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    (state, cur_result + new_sample)
                }
                #[inline(always)]
                fn initialize_sample<A: Arch>(_config: &TerraceConfig,
                    new_sample: Simd<f32, A>)
                    -> (Self::State<A>, Simd<f32, A>) {
                    (Default::default(), new_sample)
                }
                #[inline(always)]
                fn finalize_sample<A: Arch>(config: &TerraceConfig,
                    _state: Self::State<A>, last: Simd<f32, A>)
                    -> Simd<f32, A> {
                    (last * Simd::splat(config.steps)).round() *
                        Simd::splat(config.step_size)
                }
            }
        }
        pub use billow::Billow;
        pub use fbm::Fbm;
        pub use hybrid_multi::HybridMulti;
        pub use multi::Multi;
        pub use ping_pong::PingPong;
        pub use ridged::Ridged;
        pub use terrace::Terrace;
        pub trait CombinerState<A: Arch>: Copy +
            Index<usize, Output = Simd<f32, A>> + IndexMut<usize> + Default {
            const STATE_SIZE: usize;
        }
        impl<A: Arch, const N : usize> CombinerState<A> for [Simd<f32, A>; N]
            where [Simd<f32, A>; N]: Default {
            const STATE_SIZE: usize = N;
        }
        pub type CombinerArray<A: Arch, const N : usize> = [Simd<f32, A>; N];
        pub trait Combiner: Default + Copy + Clone {
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
            type State<A: Arch>: CombinerState<A>;
            /// The config struct that is passed through noise calls to the Fractal's usage.
            /// This is used for storing new parameters specific to a custom Fractal type.
            type Config: Copy + Default;
            /// Determines how new noise samples are combined with previous samples.
            ///
            /// # Parameters
            /// - `current`: Existing noise value from previous samples
            /// - `output`: New sample output from the current noise pass
            fn apply_sample<A: Arch>(config: &Self::Config,
            state: Self::State<A>, cur_result: Simd<f32, A>,
            new_sample: Simd<f32, A>)
            -> (Self::State<A>, Simd<f32, A>);
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
            fn initialize_sample<A: Arch>(config: &Self::Config,
                new_sample: Simd<f32, A>) -> (Self::State<A>, Simd<f32, A>) {
                Self::apply_sample(config, Default::default(),
                    Default::default(), new_sample)
            }
            /// Determines how the final noise sample is processed after
            /// being fully combined. This is after `sample` or `sample_first`
            /// has been called.
            ///
            /// # Parameters
            /// - `last`: The final noise sample after prior fractal processing
            #[inline(always)]
            fn finalize_sample<A: Arch>(_config: &Self::Config,
                _state: Self::State<A>, last: Simd<f32, A>) -> Simd<f32, A> {
                last
            }
        }
    }
    pub mod util {
        pub mod grid_data {
            use std::array::from_fn;
            use std::mem::MaybeUninit;
            use crate::api::grid::interface::GridNoiseParams;
            use crate::noise::util::grid_helpers::{
                Arena, configure_tiling, fill_grid_indices,
            };
            use crate::simd::Arch;
            use crate::simd::register::Simd;
            pub(crate) struct GridData<'a, const D : usize> {
                pub total_size: usize,
                pub weight: f32,
                pub grid_size: [usize; D],
                pub grid_start: [i32; D],
                pub increment: [f32; D],
                pub num_loops: [usize; D],
                pub octave_tiling: [Option<u32>; D],
                pub distances: [&'a mut [MaybeUninit<f32>]; D],
                pub fade_factors: [&'a mut [MaybeUninit<f32>]; D],
                pub grid_indices: [&'a mut [MaybeUninit<u32>]; D],
            }
            #[repr(u8)]
            pub(crate) enum Lerp { Cubic = 0, Quintic = 1, }
            impl Lerp {
                #[inline(always)]
                pub const fn from_u8(val: u8) -> Self {
                    match val {
                        0 => Self::Cubic,
                        1 => Self::Quintic,
                        _ =>
                            ::core::panicking::panic("internal error: entered unreachable code"),
                    }
                }
            }
            impl<'a, const D : usize> GridData<'a, D> {
                #[inline(always)]
                pub fn new<F: Arch, const LERP :
                    u8>(params: &GridNoiseParams<D>, arena: &mut Arena<'a>,
                    padded_size: &[usize; D]) -> Self {
                    let lerp_type = Lerp::from_u8(LERP);
                    let lanes = Simd::<f32, F>::LANES;
                    let total_size = params.grid_size.iter().product();
                    let increment =
                        from_fn(|i| params.frequency[i] * params.magnification);
                    let grid_start: [i32; D] =
                        from_fn(|i|
                                (params.position[i] as f32 * increment[i]).floor() as i32);
                    let frac_start: [f32; D] =
                        from_fn(|i|
                                (params.position[i] as f32 * increment[i] -
                                            grid_start[i] as f32).max(0.0));
                    let distances = from_fn(|i| arena.allocate(padded_size[i]));
                    let fade_factors =
                        from_fn(|i| arena.allocate(padded_size[i]));
                    let mut cur_dist: [_; D] =
                        from_fn(|i|
                                {
                                    Simd::<f32, F>::iota(0.0) *
                                            Simd::<f32, F>::splat(increment[i]) +
                                        Simd::<f32, F>::splat(frac_start[i])
                                });
                    let chunk_increment: [_; D] =
                        from_fn(|i|
                                Simd::<f32, F>::splat(increment[i] * lanes as f32));
                    for axis in 0..D {
                        for i in (0..params.grid_size[axis]).step_by(lanes) {
                            let fract_dist = cur_dist[axis].fract();
                            let cur_lerp =
                                match lerp_type {
                                    Lerp::Cubic => fract_dist.cubic_lerp(),
                                    Lerp::Quintic => fract_dist.quintic_lerp(),
                                };
                            unsafe {
                                fract_dist.copy_to_aligned_slice_unchecked(distances[axis].get_unchecked_mut(i..).assume_init_mut());
                                cur_lerp.copy_to_aligned_slice_unchecked(fade_factors[axis].get_unchecked_mut(i..).assume_init_mut());
                            }
                            cur_dist[axis] += chunk_increment[axis];
                        }
                    }
                    let mut grid_indices =
                        from_fn(|i| arena.allocate(padded_size[i]));
                    let num_loops =
                        fill_grid_indices(&mut grid_indices, &distances,
                            params.grid_size);
                    let octave_tiling = configure_tiling(params);
                    Self {
                        total_size,
                        weight: params.weight,
                        grid_size: params.grid_size,
                        grid_start,
                        increment,
                        num_loops,
                        octave_tiling,
                        distances,
                        fade_factors,
                        grid_indices,
                    }
                }
            }
        }
        pub mod grid_helpers {
            use std::array::from_fn;
            use std::marker::PhantomData;
            use std::mem::MaybeUninit;
            use std::ops::Range;
            use crate::api::grid::interface::GridNoiseParams;
            use crate::noise::combiners::{Combiner, CombinerState};
            use crate::simd::Arch;
            use crate::simd::static_simd::{StaticSimd, SIMD_WIDTH};
            use crate::simd::register::Simd;
            use crate::simd::traits::SimdElement;
            const STACK_SIZE: usize = 8192;
            pub struct ArenaBuffer<F: Arch> {
                heap: Vec<f32>,
                stack: [MaybeUninit<f32>; STACK_SIZE],
                _family: PhantomData<F>,
            }
            impl<F: Arch> ArenaBuffer<F> {
                #[inline(always)]
                pub fn with_capacity(capacity: usize) -> Self {
                    let capacity = capacity + Simd::<f32, F>::LANES;
                    let heap =
                        if capacity > STACK_SIZE {
                            Vec::with_capacity(capacity)
                        } else { Vec::new() };
                    let stack: [MaybeUninit<f32>; STACK_SIZE] =
                        std::array::from_fn(|_| MaybeUninit::uninit());
                    Self { heap, stack, _family: PhantomData::<F> }
                }
                #[inline(always)]
                pub fn as_mut_slice(&mut self) -> &mut [MaybeUninit<f32>] {
                    let slice =
                        if self.heap.capacity() > 0 {
                            self.heap.spare_capacity_mut()
                        } else { self.stack.as_mut_slice() };
                    let offset = slice.as_ptr().align_offset(F::SIMD_WIDTH);
                    unsafe { slice.get_unchecked_mut(offset..) }
                }
            }
            pub struct Arena<'a> {
                slice: &'a mut [MaybeUninit<f32>],
            }
            impl<'a> Arena<'a> {
                #[inline(always)]
                pub fn with_cache<F: Arch>(cache: &'a mut ArenaBuffer<F>)
                    -> Self {
                    let slice = cache.as_mut_slice();
                    Self { slice }
                }
                #[inline(always)]
                pub fn allocate<T>(&mut self, capacity: usize)
                    -> &'a mut [MaybeUninit<T>] {
                    const {
                            if !(size_of::<T>() == size_of::<f32>()) {
                                ::core::panicking::panic("assertion failed: size_of::<T>() == size_of::<f32>()")
                            };
                        }
                    let whole = std::mem::take(&mut self.slice);
                    let (buf, rem) = whole.split_at_mut(capacity);
                    self.slice = rem;
                    unsafe { std::mem::transmute(buf) }
                }
                #[inline(always)]
                pub fn allocate_arena(&mut self, capacity: usize) -> Self {
                    let whole = std::mem::take(&mut self.slice);
                    let (slice, rem) = whole.split_at_mut(capacity);
                    self.slice = rem;
                    Self { slice }
                }
            }
            pub struct InterpolationConfig<A: Arch> {
                pub num_blocks: usize,
                pub block_lanes: usize,
                pub has_block_head: bool,
                pub has_block_tail: bool,
                pub tail_size: usize,
                pub block_tail_size: usize,
                pub block_tail_start: usize,
                pub _family: PhantomData<A>,
            }
            impl<F: Arch> InterpolationConfig<F> {
                pub fn new(num_blocks: usize, x_dim: usize) -> Self {
                    let lanes: usize = Simd::<f32, F>::LANES;
                    let block_lanes: usize = num_blocks * lanes;
                    Self {
                        num_blocks,
                        block_lanes,
                        has_block_head: x_dim >= block_lanes,
                        has_block_tail: !x_dim.is_multiple_of(block_lanes),
                        tail_size: x_dim % block_lanes,
                        block_tail_size: (x_dim % block_lanes).div_ceil(lanes),
                        block_tail_start: (x_dim / block_lanes) * block_lanes,
                        _family: PhantomData::<F>,
                    }
                }
            }
            #[inline(always)]
            pub(crate) unsafe fn maybe_tail_load<A: Arch, const IS_TAIL :
                bool>(range: Range<usize>, slice: &[f32]) -> Simd<f32, A> {
                unsafe {
                    if IS_TAIL {
                        Simd::from_slice(slice.get_unchecked(range))
                    } else {
                        Simd::from_slice_unchecked(slice.get_unchecked(range.start..))
                    }
                }
            }
            #[inline(always)]
            pub(crate) unsafe fn maybe_tail_store<A: Arch, const IS_TAIL :
                bool>(range: Range<usize>, simd: Simd<f32, A>,
                slice: &mut [f32]) {
                unsafe {
                    if IS_TAIL {
                        simd.copy_to_slice(slice.get_unchecked_mut(range));
                    } else {
                        simd.copy_to_slice_unchecked(slice.get_unchecked_mut(range.start..));
                    }
                }
            }
            pub trait MaybeUninitSliceSimdExt<T: SimdElement, F: Arch> {
                /// # Safety
                /// - The range `index..index + ArchSimd::<T>::LANES` must be in bounds.
                /// - Data in range `index..index + ArchSimd::<T>::LANES` must be initialized.
                unsafe fn load_simd(&self, index: usize)
                -> Simd<T, F>;
                /// # Safety
                /// - The range `index..index + ArchSimd::<T>::LANES` must be in bounds.
                /// - Data in range `index..index + ArchSimd::<T>::LANES` must be initialized.
                /// - `index` must be aligned according to `SIMD_WIDTH`.
                unsafe fn load_simd_aligned(&self, index: usize)
                -> Simd<T, F>;
                /// # Safety
                /// - The range `index..index + ArchSimd::<T>::LANES` must be in bounds.
                unsafe fn write_simd(&mut self, index: usize,
                simd: Simd<T, F>);
                /// # Safety
                /// - The range `index..index + ArchSimd::<T>::LANES` must be in bounds.
                /// - `index` must be aligned according to `SIMD_WIDTH`.
                unsafe fn write_simd_aligned(&mut self, index: usize,
                simd: Simd<T, F>);
            }
            impl<T: SimdElement, F: Arch> MaybeUninitSliceSimdExt<T, F> for
                [MaybeUninit<T>] {
                unsafe fn load_simd(&self, index: usize) -> Simd<T, F> {
                    unsafe {
                        Simd::from_slice_unchecked(self.get_unchecked(index..).assume_init_ref())
                    }
                }
                unsafe fn load_simd_aligned(&self, index: usize)
                    -> Simd<T, F> {
                    unsafe {
                        Simd::from_aligned_slice_unchecked(self.get_unchecked(index..).assume_init_ref())
                    }
                }
                unsafe fn write_simd(&mut self, index: usize,
                    simd: Simd<T, F>) {
                    unsafe {
                        simd.copy_to_slice_unchecked(self.get_unchecked_mut(index..).assume_init_mut())
                    }
                }
                unsafe fn write_simd_aligned(&mut self, index: usize,
                    simd: Simd<T, F>) {
                    unsafe {
                        simd.copy_to_aligned_slice_unchecked(self.get_unchecked_mut(index..).assume_init_mut())
                    }
                }
            }
            #[inline(always)]
            pub fn validate_grid_size<const D :
                usize>(grid_size: [usize; D], slice_len: usize) {
                let num_samples = grid_size.iter().product();
                if !(slice_len >= num_samples) {
                    {
                        ::core::panicking::panic_fmt(format_args!("Uniform grid with dimensions {0:?} has a size of {1}, which is more than the given slice length of {2}",
                                grid_size, num_samples, slice_len));
                    }
                };
            }
            #[inline(always)]
            pub fn validate_state_size<C: Combiner, const D :
                usize>(grid_size: [usize; D], slice_len: usize) {
                if C::State::STATE_SIZE > 0 {
                    let total_size: usize = grid_size.iter().product();
                    let required_size = total_size * C::State::STATE_SIZE;
                    if !(slice_len >= required_size) {
                        {
                            ::core::panicking::panic_fmt(format_args!("Uniform grid with dimensions {0:?} with {1} state variables requires a state size of{2}, which is more than the given slice length of {3}",
                                    required_size, C::State::STATE_SIZE, required_size,
                                    slice_len));
                        }
                    };
                }
            }
            #[inline(always)]
            pub fn pad_grid_size<F: Arch, const D :
                usize>(grid_size: [usize; D]) -> [usize; D] {
                let lanes: usize = Simd::<f32, F>::LANES;
                from_fn(|i|
                        lanes - grid_size[i] % lanes + grid_size[i] + lanes)
            }
            pub(crate) unsafe fn assume_init_slice<T>(s: &[MaybeUninit<T>])
                -> &[T] {
                unsafe {
                    std::slice::from_raw_parts(s.as_ptr().cast(), s.len())
                }
            }
            #[inline(always)]
            pub fn fill_grid_indices<const D :
                usize>(grid_indices: &mut [&mut [MaybeUninit<u32>]; D],
                distances: &[&mut [MaybeUninit<f32>]; D],
                distances_len: [usize; D]) -> [usize; D] {
                const LANES: usize = StaticSimd::<f32>::LANES;
                std::array::from_fn(|i|
                        {
                            let mut write_idx = 0usize;
                            let indices_ptr = grid_indices[i].as_mut_ptr();
                            let last_valid = distances_len[i] - 1;
                            let full_block_end = last_valid - last_valid % 64;
                            for base_index in (1..=full_block_end).step_by(64) {
                                let mut bits = 0u64;
                                for bit_index in (0..64).step_by(LANES) {
                                    let cur_index = base_index + bit_index;
                                    let cur = unsafe { distances[i].load_simd(cur_index) };
                                    let prev =
                                        unsafe { distances[i].load_simd_aligned(cur_index - 1) };
                                    let mask_bits = prev.simd_gt(cur).to_bits();
                                    bits |= mask_bits << bit_index;
                                }
                                while bits != 0 {
                                    let cur_index = base_index as u32 + bits.trailing_zeros();
                                    unsafe {
                                        indices_ptr.add(write_idx).write(MaybeUninit::new(cur_index))
                                    };
                                    write_idx += 1;
                                    bits &= bits - 1;
                                }
                            }
                            let tail_len = last_valid - full_block_end;
                            let mut bits = 0u64;
                            for bit_index in (0..tail_len).step_by(LANES) {
                                let cur_index = bit_index + full_block_end + 1;
                                let cur = unsafe { distances[i].load_simd(cur_index) };
                                let prev =
                                    unsafe { distances[i].load_simd_aligned(cur_index - 1) };
                                let mask_bits = prev.simd_gt(cur).to_bits();
                                bits |= mask_bits << bit_index;
                            }
                            bits &= (1u64 << tail_len) - 1;
                            while bits != 0 {
                                let cur_index =
                                    full_block_end as u32 + bits.trailing_zeros() + 1;
                                unsafe {
                                    indices_ptr.add(write_idx).write(MaybeUninit::new(cur_index))
                                };
                                write_idx += 1;
                                bits &= bits - 1;
                            }
                            unsafe {
                                indices_ptr.add(write_idx).write(MaybeUninit::new(distances_len[i]
                                            as u32))
                            };
                            write_idx + 1
                        })
            }
            #[inline(always)]
            pub(crate) fn configure_tiling<const D :
                usize>(params: &GridNoiseParams<D>) -> [Option<u32>; D] {
                std::array::from_fn(|i|
                        {
                            if let Some(val) = params.tiling[i] {
                                let float = val as f32 * params.frequency[i];
                                let nearness = (float - float.round()).abs();
                                if !(nearness < 0.001) {
                                    {
                                        ::core::panicking::panic_fmt(format_args!("Frequency does not align with the tiling!"));
                                    }
                                };
                                Some(float.round() as u32)
                            } else { None }
                        })
            }
        }
    }
}
pub mod simd {
    pub mod static_simd {
        use crate::simd::architectures::interface::Static;
        use crate::simd::mask::Mask;
        use crate::simd::register::Simd;
        use crate::simd::architectures::arch::Scalar128;
        pub type StaticSimd<T> = Simd<T, Scalar128>;
        pub type StaticMask<T> = Mask<T, Scalar128>;
        pub type StaticArch = Scalar128;
        pub type ScalarArch = <StaticArch as Arch>::ScalarFamily;
        pub type ScalarSimd<T> = Simd<T, ScalarArch>;
        pub type ScalarMask<T> = Mask<T, ScalarArch>;
    }
    pub mod architectures {
        #![allow(clippy::missing_transmute_annotations, unused_unsafe,
        clippy::useless_transmute, clippy::macro_metavars_in_unsafe)]
        pub mod intrinsics {
            pub mod avx2 {
                use std::arch::x86_64::*;
                use std::mem::{transmute, transmute_copy};
                use crate::simd::architectures::interface::*;
                use crate::simd::architectures::macros::*;
                #[repr(transparent)]
                pub struct Avx2Reg(pub __m256i);
                #[automatically_derived]
                impl ::core::marker::Copy for Avx2Reg { }
                #[automatically_derived]
                #[doc(hidden)]
                unsafe impl ::core::clone::TrivialClone for Avx2Reg { }
                #[automatically_derived]
                impl ::core::clone::Clone for Avx2Reg {
                    #[inline]
                    fn clone(&self) -> Avx2Reg {
                        let _: ::core::clone::AssertParamIsClone<__m256i>;
                        *self
                    }
                }
                impl SimdArch for Avx2Reg {}
                impl MaskArch for Avx2Reg {}
                impl SimdAddImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn f64_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_add_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_add_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i64_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_add_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_add_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_add_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i8_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_add_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdSubImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn f64_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sub_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sub_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i64_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sub_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sub_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sub_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i8_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sub_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdMulImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn f64_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_mul_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_mul_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_mullo_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_mullo_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdDivImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn f64_div(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_div_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_div(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_div_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdBitwiseImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn and(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_and_si256(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn or(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_or_si256(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn xor(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_xor_si256(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn not(self) -> Self {
                        unsafe { Self(self.xor(Self::splat_32(!0)).0) }
                    }
                    #[inline(always)]
                    unsafe fn and_not(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_andnot_si256(transmute_copy(&rhs),
                                            transmute_copy(&self))))
                        }
                    }
                }
                impl SimdShiftImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn sllv_64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sllv_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_srlv_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_srav_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn sllv_32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sllv_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_srlv_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_srav_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn sllv_16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sllv_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_srlv_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_srav_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdLoadImpl for Avx2Reg {
                    type MaskType = Self;
                    #[inline(always)]
                    unsafe fn load_aligned<T>(ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_load_si256(transmute_copy(&ptr))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn load_unaligned<T>(ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_loadu_si256(transmute_copy(&ptr))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_load_64<T>(ptr: *const T,
                        mask: Self::MaskType) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_maskload_epi64(transmute_copy(&ptr),
                                            transmute_copy(&mask))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_load_32<T>(ptr: *const T,
                        mask: Self::MaskType) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_maskload_epi32(transmute_copy(&ptr),
                                            transmute_copy(&mask))))
                        }
                    }
                }
                impl SimdStoreImpl for Avx2Reg {
                    type MaskType = Self;
                    #[inline(always)]
                    unsafe fn store_aligned<T>(self, ptr: *mut T) {
                        unsafe {
                            _mm256_store_si256(transmute_copy(&ptr),
                                transmute_copy(&self))
                        };
                    }
                    #[inline(always)]
                    unsafe fn store_unaligned<T>(self, ptr: *mut T) {
                        unsafe {
                            _mm256_storeu_si256(transmute_copy(&ptr),
                                transmute_copy(&self))
                        };
                    }
                    #[inline(always)]
                    unsafe fn masked_store_64<T>(self, ptr: *mut T,
                        mask: Self::MaskType) {
                        unsafe {
                            _mm256_maskstore_epi64(transmute_copy(&ptr),
                                transmute_copy(&mask), transmute_copy(&self))
                        };
                    }
                    #[inline(always)]
                    unsafe fn masked_store_32<T>(self, ptr: *mut T,
                        mask: Self::MaskType) {
                        unsafe {
                            _mm256_maskstore_epi32(transmute_copy(&ptr),
                                transmute_copy(&mask), transmute_copy(&self))
                        };
                    }
                }
                impl SimdZeroImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn zero() -> Self {
                        unsafe { Self(transmute_copy(&_mm256_setzero_si256())) }
                    }
                }
                impl SimdFloatCastsImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn float_to_int_trunc(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cvttps_epi32(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn float_to_int_round(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cvtps_epi32(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdIntCastsImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn int_to_float(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cvtepi32_ps(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdPermuteImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn permute_32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_permutevar8x32_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn permute_8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_shuffle_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdVariableBlendImpl for Avx2Reg {
                    type VecType = Self;
                    #[inline(always)]
                    unsafe fn vblend_64(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self::VecType {
                        unsafe {
                            Self(transmute_copy(&_mm256_blendv_pd(transmute_copy(&false_values),
                                            transmute_copy(&true_values), transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn vblend_32(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self::VecType {
                        unsafe {
                            Self(transmute_copy(&_mm256_blendv_ps(transmute_copy(&false_values),
                                            transmute_copy(&true_values), transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn vblend_8(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self::VecType {
                        unsafe {
                            Self(transmute_copy(&_mm256_blendv_epi8(transmute_copy(&false_values),
                                            transmute_copy(&true_values), transmute_copy(&self))))
                        }
                    }
                }
                impl SimdImmediateBlendImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn blend_64<const N : i32>(self, false_values: Self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm256_blend_pd::<{
                                                N
                                            }>(transmute(false_values), transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn blend_32<const N : i32>(self, false_values: Self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm256_blend_ps::<{
                                                N
                                            }>(transmute(false_values), transmute(self))))
                        }
                    }
                }
                impl SimdMulAddImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn mul_add_f64(self, mult: Self, add: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_fmadd_pd(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_sub_f64(self, mult: Self, sub: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_fmsub_pd(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_add_f64(self, mult: Self, add: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_fnmadd_pd(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_sub_f64(self, mult: Self, sub: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_fnmsub_pd(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_add_f32(self, mult: Self, add: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_fmadd_ps(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_sub_f32(self, mult: Self, sub: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_fmsub_ps(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_add_f32(self, mult: Self, add: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_fnmadd_ps(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_sub_f32(self, mult: Self, sub: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_fnmsub_ps(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&sub))))
                        }
                    }
                }
                impl SimdRoundImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn round_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_round_pd::<{
                                                _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn round_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_round_ps::<{
                                                _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn floor_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_round_pd::<{
                                                _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn floor_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_round_ps::<{
                                                _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn ceil_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_round_pd::<{
                                                _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn ceil_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_round_ps::<{
                                                _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                }
                impl SimdPartialOrdImpl for Avx2Reg {
                    type MaskType = Self;
                    #[inline(always)]
                    unsafe fn cmp_f64_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_pd::<{
                                                _CMP_EQ_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_lt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_pd::<{
                                                _CMP_LT_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_le(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_pd::<{
                                                _CMP_LE_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_pd::<{
                                                _CMP_GT_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_ge(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_pd::<{
                                                _CMP_GE_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_neq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_pd::<{
                                                _CMP_NEQ_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_ps::<{
                                                _CMP_EQ_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_lt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_ps::<{
                                                _CMP_LT_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_le(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_ps::<{
                                                _CMP_LE_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_ps::<{
                                                _CMP_GT_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_ge(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_ps::<{
                                                _CMP_GE_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_neq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute(_mm256_cmp_ps::<{
                                                _CMP_NEQ_OQ
                                            }>(transmute(self), transmute(rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i64_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpeq_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i64_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpgt_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i32_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpeq_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i32_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpgt_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i16_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpeq_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i16_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpgt_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i8_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpeq_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i8_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpgt_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_f64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_max_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_f64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_min_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_f32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_max_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_f32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_min_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_max_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_min_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_max_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_min_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_max_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_min_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_max_epu32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_min_epu32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_max_epu16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_min_epu16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_max_epu8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_min_epu8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdSplatImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn splat_64<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_set1_epi64x(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_32<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_set1_epi32(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_16<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_set1_epi16(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_8<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_set1_epi8(transmute_copy(&val))))
                        }
                    }
                }
                impl SimdGatherImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn gather_32_from_32<T, const B :
                        i32>(self, ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute(_mm256_i32gather_epi32::<{
                                                B
                                            }>(transmute(ptr), transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn gather_64_from_64<T, const B :
                        i32>(self, ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute(_mm256_i64gather_epi64::<{
                                                B
                                            }>(transmute(ptr), transmute(self))))
                        }
                    }
                }
                impl SimdSqrtImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn sqrt_f64(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sqrt_pd(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn sqrt_f32(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_sqrt_ps(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn rsqrt_f32(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm256_rsqrt_ps(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdAllBitsImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn all_zero(self) -> bool {
                        (unsafe {
                                _mm256_testz_si256(transmute_copy(&self),
                                    transmute_copy(&self))
                            }) != 0
                    }
                }
                impl SimdNegateImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn negate_f64(self) -> Self {
                        unsafe { Self::splat_64(-0.0f64).xor(self) }
                    }
                    #[inline(always)]
                    unsafe fn negate_f32(self) -> Self {
                        unsafe { Self::splat_32(-0.0f64).xor(self) }
                    }
                }
                impl SimdBlockShiftImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn block_left_byte_shift<const N : i32>(self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm256_bslli_epi128::<{
                                                N
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn block_right_byte_shift<const N : i32>(self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm256_bsrli_epi128::<{
                                                N
                                            }>(transmute(self))))
                        }
                    }
                }
                impl SimdMaskBitConversion for Avx2Reg {
                    #[inline(always)]
                    unsafe fn to_bits_64(self) -> u64 {
                        (unsafe { _mm256_movemask_pd(transmute_copy(&self)) }) as
                            u64
                    }
                    #[inline(always)]
                    unsafe fn to_bits_32(self) -> u64 {
                        (unsafe { _mm256_movemask_ps(transmute_copy(&self)) }) as
                            u64
                    }
                    #[inline(always)]
                    unsafe fn to_bits_8(self) -> u64 {
                        (unsafe { _mm256_movemask_epi8(transmute_copy(&self)) }) as
                            u64
                    }
                    #[inline(always)]
                    unsafe fn from_bits_64(bitmask: u64) -> Self {
                        let mask =
                            unsafe {
                                Self(transmute_copy(&_mm256_set_epi64x(transmute_copy(&1),
                                                transmute_copy(&2), transmute_copy(&4),
                                                transmute_copy(&8))))
                            };
                        let bits =
                            unsafe {
                                Self(transmute_copy(&_mm256_set1_epi64x(transmute_copy(&(bitmask
                                                            as i64)))))
                            };
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpgt_epi64(transmute_copy(&bits.and(mask)),
                                            transmute_copy(&Self::zero()))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_32(bitmask: u64) -> Self {
                        let mask =
                            unsafe {
                                Self(transmute_copy(&_mm256_set_epi32(transmute_copy(&1),
                                                transmute_copy(&2), transmute_copy(&4), transmute_copy(&8),
                                                transmute_copy(&16), transmute_copy(&32),
                                                transmute_copy(&64), transmute_copy(&128))))
                            };
                        let bits =
                            unsafe {
                                Self(transmute_copy(&_mm256_set1_epi32(transmute_copy(&(bitmask
                                                            as u32)))))
                            };
                        unsafe {
                            Self(transmute_copy(&_mm256_cmpgt_epi32(transmute_copy(&bits.and(mask)),
                                            transmute_copy(&Self::zero()))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_16(bitmask: u64) -> Self {
                        #[rustfmt::skip]
                        let mask =
                            unsafe {
                                Self(transmute_copy(&_mm256_set_epi16(transmute_copy(&1),
                                                transmute_copy(&2), transmute_copy(&4), transmute_copy(&8),
                                                transmute_copy(&16), transmute_copy(&32),
                                                transmute_copy(&64), transmute_copy(&128),
                                                transmute_copy(&256), transmute_copy(&512),
                                                transmute_copy(&1024), transmute_copy(&2048),
                                                transmute_copy(&4096), transmute_copy(&8192),
                                                transmute_copy(&16384), transmute_copy(&-32768))))
                            };
                        let bits =
                            unsafe {
                                Self(transmute_copy(&_mm256_set1_epi16(transmute_copy(&(bitmask
                                                            as i16)))))
                            };
                        unsafe {
                            unsafe {
                                    Self(transmute_copy(&_mm256_cmpeq_epi16(transmute_copy(&bits.and(mask)),
                                                    transmute_copy(&Self::zero()))))
                                }.not()
                        }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_8(bitmask: u64) -> Self {
                        let b1 = bitmask as i8;
                        let b2 = (bitmask >> 8) as i8;
                        let b3 = (bitmask >> 16) as i8;
                        let b4 = (bitmask >> 24) as i8;
                        #[rustfmt::skip]
                        let mask =
                            unsafe {
                                Self(transmute_copy(&_mm256_set_epi8(transmute_copy(&1),
                                                transmute_copy(&2), transmute_copy(&4), transmute_copy(&8),
                                                transmute_copy(&16), transmute_copy(&32),
                                                transmute_copy(&64), transmute_copy(&-128),
                                                transmute_copy(&1), transmute_copy(&2), transmute_copy(&4),
                                                transmute_copy(&8), transmute_copy(&16),
                                                transmute_copy(&32), transmute_copy(&64),
                                                transmute_copy(&-128), transmute_copy(&1),
                                                transmute_copy(&2), transmute_copy(&4), transmute_copy(&8),
                                                transmute_copy(&16), transmute_copy(&32),
                                                transmute_copy(&64), transmute_copy(&-128),
                                                transmute_copy(&1), transmute_copy(&2), transmute_copy(&4),
                                                transmute_copy(&8), transmute_copy(&16),
                                                transmute_copy(&32), transmute_copy(&64),
                                                transmute_copy(&-128))))
                            };
                        #[rustfmt::skip]
                        let bits =
                            unsafe {
                                Self(transmute_copy(&_mm256_set_epi8(transmute_copy(&b1),
                                                transmute_copy(&b1), transmute_copy(&b1),
                                                transmute_copy(&b1), transmute_copy(&b1),
                                                transmute_copy(&b1), transmute_copy(&b1),
                                                transmute_copy(&b1), transmute_copy(&b2),
                                                transmute_copy(&b2), transmute_copy(&b2),
                                                transmute_copy(&b2), transmute_copy(&b2),
                                                transmute_copy(&b2), transmute_copy(&b2),
                                                transmute_copy(&b2), transmute_copy(&b3),
                                                transmute_copy(&b3), transmute_copy(&b3),
                                                transmute_copy(&b3), transmute_copy(&b3),
                                                transmute_copy(&b3), transmute_copy(&b3),
                                                transmute_copy(&b3), transmute_copy(&b4),
                                                transmute_copy(&b4), transmute_copy(&b4),
                                                transmute_copy(&b4), transmute_copy(&b4),
                                                transmute_copy(&b4), transmute_copy(&b4),
                                                transmute_copy(&b4))))
                            };
                        unsafe {
                            unsafe {
                                    Self(transmute_copy(&_mm256_cmpeq_epi8(transmute_copy(&bits.and(mask)),
                                                    transmute_copy(&Self::zero()))))
                                }.not()
                        }
                    }
                }
                impl SimdLaneShiftImpl for Avx2Reg {
                    #[inline(always)]
                    unsafe fn left_lane_shift_32(self, n: u32) -> Self {
                        unsafe {
                            match n {
                                0 =>
                                    self,
                                    #[rustfmt::skip]
                                    1 =>
                                    Self::zero().blend_32::<1>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&1),
                                                            transmute_copy(&2), transmute_copy(&3), transmute_copy(&4),
                                                            transmute_copy(&5), transmute_copy(&6), transmute_copy(&7),
                                                            transmute_copy(&0))))
                                        }),
                                    #[rustfmt::skip]
                                    2 =>
                                    Self::zero().blend_32::<1>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&2),
                                                            transmute_copy(&3), transmute_copy(&4), transmute_copy(&5),
                                                            transmute_copy(&6), transmute_copy(&7), transmute_copy(&0),
                                                            transmute_copy(&0))))
                                        }),
                                    #[rustfmt::skip]
                                    3 =>
                                    Self::zero().blend_32::<1>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&3),
                                                            transmute_copy(&4), transmute_copy(&5), transmute_copy(&6),
                                                            transmute_copy(&7), transmute_copy(&0), transmute_copy(&0),
                                                            transmute_copy(&0))))
                                        }),
                                    #[rustfmt::skip]
                                    4 =>
                                    Self::zero().blend_32::<1>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&4),
                                                            transmute_copy(&5), transmute_copy(&6), transmute_copy(&7),
                                                            transmute_copy(&0), transmute_copy(&0), transmute_copy(&0),
                                                            transmute_copy(&0))))
                                        }),
                                    #[rustfmt::skip]
                                    5 =>
                                    Self::zero().blend_32::<1>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&5),
                                                            transmute_copy(&6), transmute_copy(&7), transmute_copy(&0),
                                                            transmute_copy(&0), transmute_copy(&0), transmute_copy(&0),
                                                            transmute_copy(&0))))
                                        }),
                                    #[rustfmt::skip]
                                    6 =>
                                    Self::zero().blend_32::<1>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&6),
                                                            transmute_copy(&7), transmute_copy(&0), transmute_copy(&0),
                                                            transmute_copy(&0), transmute_copy(&0), transmute_copy(&0),
                                                            transmute_copy(&0))))
                                        }),
                                    #[rustfmt::skip]
                                    7 =>
                                    Self::zero().blend_32::<1>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&7),
                                                            transmute_copy(&0), transmute_copy(&0), transmute_copy(&0),
                                                            transmute_copy(&0), transmute_copy(&0), transmute_copy(&0),
                                                            transmute_copy(&0))))
                                        }),
                                _ => Self::zero(),
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn right_lane_shift_32(self, n: u32) -> Self {
                        unsafe {
                            match n {
                                0 => self,
                                1 =>
                                    Self::zero().blend_32::<0x80>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&7),
                                                            transmute_copy(&0), transmute_copy(&1), transmute_copy(&2),
                                                            transmute_copy(&3), transmute_copy(&4), transmute_copy(&5),
                                                            transmute_copy(&6))))
                                        }),
                                2 =>
                                    Self::zero().blend_32::<0x80>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&0), transmute_copy(&1),
                                                            transmute_copy(&2), transmute_copy(&3), transmute_copy(&4),
                                                            transmute_copy(&5))))
                                        }),
                                3 =>
                                    Self::zero().blend_32::<0x80>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&7), transmute_copy(&0),
                                                            transmute_copy(&1), transmute_copy(&2), transmute_copy(&3),
                                                            transmute_copy(&4))))
                                        }),
                                4 =>
                                    Self::zero().blend_32::<0x80>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&7), transmute_copy(&7),
                                                            transmute_copy(&0), transmute_copy(&1), transmute_copy(&2),
                                                            transmute_copy(&3))))
                                        }),
                                5 =>
                                    Self::zero().blend_32::<0x80>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&7), transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&0), transmute_copy(&1),
                                                            transmute_copy(&2))))
                                        }),
                                6 =>
                                    Self::zero().blend_32::<0x80>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&7), transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&7), transmute_copy(&0),
                                                            transmute_copy(&1))))
                                        }),
                                7 =>
                                    Self::zero().blend_32::<0x80>(self).permute_32(unsafe {
                                            Self(transmute_copy(&_mm256_setr_epi32(transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&7), transmute_copy(&7),
                                                            transmute_copy(&7), transmute_copy(&7), transmute_copy(&7),
                                                            transmute_copy(&0))))
                                        }),
                                _ => Self::zero(),
                            }
                        }
                    }
                }
            }
            pub mod avx512 {
                use std::arch::x86_64::*;
                use std::mem::{transmute, transmute_copy};
                use crate::simd::architectures::interface::*;
                use crate::simd::architectures::macros::*;
                #[repr(transparent)]
                pub struct Avx512Reg(pub __m512i);
                #[automatically_derived]
                impl ::core::marker::Copy for Avx512Reg { }
                #[automatically_derived]
                #[doc(hidden)]
                unsafe impl ::core::clone::TrivialClone for Avx512Reg { }
                #[automatically_derived]
                impl ::core::clone::Clone for Avx512Reg {
                    #[inline]
                    fn clone(&self) -> Avx512Reg {
                        let _: ::core::clone::AssertParamIsClone<__m512i>;
                        *self
                    }
                }
                impl SimdArch for Avx512Reg {}
                pub struct Avx512Mask(pub __mmask64);
                #[automatically_derived]
                impl ::core::marker::Copy for Avx512Mask { }
                #[automatically_derived]
                #[doc(hidden)]
                unsafe impl ::core::clone::TrivialClone for Avx512Mask { }
                #[automatically_derived]
                impl ::core::clone::Clone for Avx512Mask {
                    #[inline]
                    fn clone(&self) -> Avx512Mask {
                        let _: ::core::clone::AssertParamIsClone<__mmask64>;
                        *self
                    }
                }
                impl MaskArch for Avx512Mask {}
                impl SimdAddImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn f64_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_add_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_add_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i64_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_add_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_add_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_add_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i8_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_add_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdSubImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn f64_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sub_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sub_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i64_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sub_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sub_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sub_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i8_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sub_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdMulImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn f64_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_mul_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_mul_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_mullo_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_mullo_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdDivImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn f64_div(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_div_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_div(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_div_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdBitwiseImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn and(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_and_si512(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn or(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_or_si512(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn xor(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_xor_si512(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn not(self) -> Self {
                        unsafe { Self(self.xor(Self::splat_32(!0)).0) }
                    }
                    #[inline(always)]
                    unsafe fn and_not(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_andnot_si512(transmute_copy(&rhs),
                                            transmute_copy(&self))))
                        }
                    }
                }
                impl SimdShiftImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn sllv_64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sllv_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_srlv_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_srav_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn sllv_32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sllv_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_srlv_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_srav_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn sllv_16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sllv_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_srlv_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_srav_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdLoadImpl for Avx512Reg {
                    type MaskType = Avx512Mask;
                    #[inline(always)]
                    unsafe fn load_aligned<T>(ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_load_si512(transmute_copy(&ptr))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn load_unaligned<T>(ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_loadu_si512(transmute_copy(&ptr))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_load_64<T>(ptr: *const T,
                        mask: Self::MaskType) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_mask_loadu_epi64(transmute_copy(&Self::zero()),
                                            transmute_copy(&mask), transmute_copy(&ptr))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_load_32<T>(ptr: *const T,
                        mask: Self::MaskType) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_mask_loadu_epi32(transmute_copy(&Self::zero()),
                                            transmute_copy(&mask), transmute_copy(&ptr))))
                        }
                    }
                }
                impl SimdStoreImpl for Avx512Reg {
                    type MaskType = Avx512Mask;
                    #[inline(always)]
                    unsafe fn store_aligned<T>(self, ptr: *mut T) {
                        unsafe {
                            _mm512_store_si512(transmute_copy(&ptr),
                                transmute_copy(&self))
                        };
                    }
                    #[inline(always)]
                    unsafe fn store_unaligned<T>(self, ptr: *mut T) {
                        unsafe {
                            _mm512_storeu_si512(transmute_copy(&ptr),
                                transmute_copy(&self))
                        };
                    }
                    #[inline(always)]
                    unsafe fn masked_store_64<T>(self, ptr: *mut T,
                        mask: Self::MaskType) {
                        unsafe {
                            _mm512_mask_storeu_epi64(transmute_copy(&ptr),
                                transmute_copy(&mask), transmute_copy(&self))
                        };
                    }
                    #[inline(always)]
                    unsafe fn masked_store_32<T>(self, ptr: *mut T,
                        mask: Self::MaskType) {
                        unsafe {
                            _mm512_mask_storeu_epi32(transmute_copy(&ptr),
                                transmute_copy(&mask), transmute_copy(&self))
                        };
                    }
                }
                impl SimdZeroImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn zero() -> Self {
                        unsafe { Self(transmute_copy(&_mm512_setzero_si512())) }
                    }
                }
                impl SimdFloatCastsImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn float_to_int_trunc(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_cvttps_epi32(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn float_to_int_round(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_cvtps_epi32(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdIntCastsImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn int_to_float(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_cvtepi32_ps(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdPermuteImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn permute_32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_permutexvar_epi32(transmute_copy(&rhs),
                                            transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn permute_8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_shuffle_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdVariableBlendImpl for Avx512Mask {
                    type VecType = Avx512Reg;
                    #[inline(always)]
                    unsafe fn vblend_64(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self::VecType {
                        unsafe {
                            Avx512Reg(transmute(unsafe {
                                        _mm512_mask_blend_pd(transmute_copy(&self),
                                            transmute_copy(&false_values), transmute_copy(&true_values))
                                    }))
                        }
                    }
                    #[inline(always)]
                    unsafe fn vblend_32(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self::VecType {
                        unsafe {
                            Avx512Reg(transmute(unsafe {
                                        _mm512_mask_blend_ps(transmute_copy(&self),
                                            transmute_copy(&false_values), transmute_copy(&true_values))
                                    }))
                        }
                    }
                    #[inline(always)]
                    unsafe fn vblend_8(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self::VecType {
                        unsafe {
                            Avx512Reg(transmute(unsafe {
                                        _mm512_mask_blend_epi8(transmute_copy(&self),
                                            transmute_copy(&false_values), transmute_copy(&true_values))
                                    }))
                        }
                    }
                }
                impl SimdImmediateBlendImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn blend_64<const N : i32>(self, false_values: Self)
                        -> Self {
                        let mask = Avx512Mask(N as u64);
                        unsafe { mask.vblend_64(self, false_values) }
                    }
                    #[inline(always)]
                    unsafe fn blend_32<const N : i32>(self, false_values: Self)
                        -> Self {
                        let mask = Avx512Mask(N as u64);
                        unsafe { mask.vblend_32(self, false_values) }
                    }
                }
                impl SimdMulAddImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn mul_add_f64(self, mult: Self, add: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_fmadd_pd(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_sub_f64(self, mult: Self, sub: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_fmsub_pd(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_add_f64(self, mult: Self, add: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_fnmadd_pd(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_sub_f64(self, mult: Self, sub: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_fnmsub_pd(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_add_f32(self, mult: Self, add: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_fmadd_ps(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_sub_f32(self, mult: Self, sub: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_fmsub_ps(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_add_f32(self, mult: Self, add: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_fnmadd_ps(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_sub_f32(self, mult: Self, sub: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_fnmsub_ps(transmute_copy(&self),
                                            transmute_copy(&mult), transmute_copy(&sub))))
                        }
                    }
                }
                impl SimdRoundImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn round_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm512_roundscale_pd::<{
                                                _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn round_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm512_roundscale_ps::<{
                                                _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn floor_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm512_roundscale_pd::<{
                                                _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn floor_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm512_roundscale_ps::<{
                                                _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn ceil_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm512_roundscale_pd::<{
                                                _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn ceil_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm512_roundscale_ps::<{
                                                _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                }
                impl SimdPartialOrdImpl for Avx512Reg {
                    type MaskType = Avx512Mask;
                    #[inline(always)]
                    unsafe fn cmp_f64_eq(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_pd_mask::<{
                                                _CMP_EQ_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_lt(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_pd_mask::<{
                                                _CMP_LT_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_le(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_pd_mask::<{
                                                _CMP_LE_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_gt(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_pd_mask::<{
                                                _CMP_GT_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_ge(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_pd_mask::<{
                                                _CMP_GE_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_neq(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_pd_mask::<{
                                                _CMP_NEQ_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_eq(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_ps_mask::<{
                                                _CMP_EQ_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_lt(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_ps_mask::<{
                                                _CMP_LT_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_le(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_ps_mask::<{
                                                _CMP_LE_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_gt(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_ps_mask::<{
                                                _CMP_GT_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_ge(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_ps_mask::<{
                                                _CMP_GE_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_neq(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmp_ps_mask::<{
                                                _CMP_NEQ_OQ
                                            }>(transmute(self), transmute(rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_i64_eq(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmpeq_epi64_mask(transmute_copy(&self),
                                        transmute_copy(&rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_i64_gt(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmpgt_epi64_mask(transmute_copy(&self),
                                        transmute_copy(&rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_i32_eq(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmpeq_epi32_mask(transmute_copy(&self),
                                        transmute_copy(&rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_i32_gt(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmpgt_epi32_mask(transmute_copy(&self),
                                        transmute_copy(&rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_i16_eq(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmpeq_epi16_mask(transmute_copy(&self),
                                        transmute_copy(&rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_i16_gt(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmpgt_epi16_mask(transmute_copy(&self),
                                        transmute_copy(&rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_i8_eq(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmpeq_epi8_mask(transmute_copy(&self),
                                        transmute_copy(&rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn cmp_i8_gt(self, rhs: Self) -> Self::MaskType {
                        Avx512Mask(unsafe {
                                    _mm512_cmpgt_epi8_mask(transmute_copy(&self),
                                        transmute_copy(&rhs))
                                } as u64)
                    }
                    #[inline(always)]
                    unsafe fn max_f64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_max_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_f64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_min_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_f32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_max_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_f32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_min_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_max_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_min_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_max_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_min_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_max_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_min_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_max_epu32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_min_epu32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_max_epu16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_min_epu16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_max_epu8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_min_epu8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdSplatImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn splat_64<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_set1_epi64(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_32<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_set1_epi32(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_16<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_set1_epi16(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_8<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_set1_epi8(transmute_copy(&val))))
                        }
                    }
                }
                impl SimdBitwiseImpl for Avx512Mask {
                    #[inline(always)]
                    unsafe fn and(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_kand_mask64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn or(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_kor_mask64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn xor(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_kxor_mask64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn not(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_knot_mask64(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn and_not(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_kandn_mask64(transmute_copy(&rhs),
                                            transmute_copy(&self))))
                        }
                    }
                }
                impl SimdGatherImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn gather_32_from_32<T, const B :
                        i32>(self, ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute(_mm512_i32gather_epi32::<{
                                                B
                                            }>(transmute(self), transmute(ptr))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn gather_64_from_64<T, const B :
                        i32>(self, ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute(_mm512_i64gather_epi64::<{
                                                B
                                            }>(transmute(self), transmute(ptr))))
                        }
                    }
                }
                impl SimdSqrtImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn sqrt_f64(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sqrt_pd(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn sqrt_f32(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_sqrt_ps(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn rsqrt_f32(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm512_rsqrt14_ps(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdAllBitsImpl for Avx512Mask {
                    #[inline(always)]
                    unsafe fn all_zero(self) -> bool { self.0 == 0 }
                }
                impl SimdNegateImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn negate_f64(self) -> Self {
                        unsafe { Self::splat_64(-0.0f64).xor(self) }
                    }
                    #[inline(always)]
                    unsafe fn negate_f32(self) -> Self {
                        unsafe { Self::splat_32(-0.0f64).xor(self) }
                    }
                }
                impl SimdBlockShiftImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn block_left_byte_shift<const N : i32>(self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm512_bslli_epi128::<{
                                                N
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn block_right_byte_shift<const N : i32>(self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm512_bsrli_epi128::<{
                                                N
                                            }>(transmute(self))))
                        }
                    }
                }
                impl SimdMaskBitConversion for Avx512Mask {
                    #[inline(always)]
                    unsafe fn to_bits_64(self) -> u64 { self.0 }
                    #[inline(always)]
                    unsafe fn to_bits_32(self) -> u64 { self.0 }
                    #[inline(always)]
                    unsafe fn to_bits_8(self) -> u64 { self.0 }
                    #[inline(always)]
                    unsafe fn from_bits_64(bitmask: u64) -> Self {
                        unsafe { transmute(bitmask) }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_32(bitmask: u64) -> Self {
                        unsafe { transmute(bitmask) }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_16(bitmask: u64) -> Self {
                        unsafe { transmute(bitmask) }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_8(bitmask: u64) -> Self {
                        unsafe { transmute(bitmask) }
                    }
                }
                impl SimdLaneShiftImpl for Avx512Reg {
                    #[inline(always)]
                    unsafe fn right_lane_shift_32(self, n: u32) -> Self {
                        match n {
                            0 => self,
                            1 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 15)
                                    }),
                            2 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 14)
                                    }),
                            3 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 13)
                                    }),
                            4 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 12)
                                    }),
                            5 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 11)
                                    }),
                            6 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 10)
                                    }),
                            7 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 9)
                                    }),
                            8 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 8)
                                    }),
                            9 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 7)
                                    }),
                            10 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 6)
                                    }),
                            11 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 5)
                                    }),
                            12 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 4)
                                    }),
                            13 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 3)
                                    }),
                            14 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 2)
                                    }),
                            15 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 1)
                                    }),
                            _ => unsafe { Self::zero() },
                        }
                    }
                    #[inline(always)]
                    unsafe fn left_lane_shift_32(self, n: u32) -> Self {
                        match n {
                            0 => self,
                            1 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 1)
                                    }),
                            2 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 2)
                                    }),
                            3 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 3)
                                    }),
                            4 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 4)
                                    }),
                            5 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 5)
                                    }),
                            6 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 6)
                                    }),
                            7 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 7)
                                    }),
                            8 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 8)
                                    }),
                            9 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 9)
                                    }),
                            10 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 10)
                                    }),
                            11 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 11)
                                    }),
                            12 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 12)
                                    }),
                            13 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 13)
                                    }),
                            14 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 14)
                                    }),
                            15 =>
                                Self(unsafe {
                                        _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 15)
                                    }),
                            _ => unsafe { Self::zero() },
                        }
                    }
                }
            }
            pub mod sse {
                use std::arch::x86_64::*;
                use std::mem::{transmute, transmute_copy};
                use crate::simd::architectures::interface::*;
                use crate::simd::architectures::macros::*;
                #[repr(transparent)]
                pub struct SseReg(pub __m128i);
                #[automatically_derived]
                impl ::core::marker::Copy for SseReg { }
                #[automatically_derived]
                #[doc(hidden)]
                unsafe impl ::core::clone::TrivialClone for SseReg { }
                #[automatically_derived]
                impl ::core::clone::Clone for SseReg {
                    #[inline]
                    fn clone(&self) -> SseReg {
                        let _: ::core::clone::AssertParamIsClone<__m128i>;
                        *self
                    }
                }
                impl SimdArch for SseReg {}
                impl MaskArch for SseReg {}
                impl SimdAddImpl for SseReg {
                    #[inline(always)]
                    unsafe fn f64_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_add_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_add_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i64_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_add_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_add_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_add_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i8_add(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_add_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdSubImpl for SseReg {
                    #[inline(always)]
                    unsafe fn f64_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i64_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i8_sub(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdMulImpl for SseReg {
                    #[inline(always)]
                    unsafe fn f64_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_mul_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_mul_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_mullo_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_mul(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_mullo_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdDivImpl for SseReg {
                    #[inline(always)]
                    unsafe fn f64_div(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_div_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_div(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_div_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdBitwiseImpl for SseReg {
                    #[inline(always)]
                    unsafe fn and(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_and_si128(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn or(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_or_si128(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn xor(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_xor_si128(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn not(self) -> Self {
                        unsafe { Self(self.xor(Self::splat_32(!0)).0) }
                    }
                    #[inline(always)]
                    unsafe fn and_not(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_andnot_si128(transmute_copy(&rhs),
                                            transmute_copy(&self))))
                        }
                    }
                }
                macro_rules! scalar_shift {
                    ($self:expr, $rhs:expr, $lanes:expr, $elem:ty, $uelem:ty,
                    $op:tt) =>
                    {{
                            let mut a = [<$elem>::default(); $lanes]; let mut b =
                            [<$elem>::default(); $lanes];
                            _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, $self.0);
                            _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, $rhs.0);
                            let width = (std::mem::size_of::<$elem>() * 8) as u32; let
                            mut out = [<$elem>::default(); $lanes]; for i in 0..$lanes
                            {
                                let shift = b[i] as $uelem as u32; out[i] =
                                scalar_shift!(@apply $op, a[i], shift, width, $elem,
                                $uelem);
                            } Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                        }};
                    (@apply sll, $val:expr, $shift:expr, $width:expr, $elem:ty,
                    $uelem:ty) =>
                    {
                        if $shift >= $width { 0 } else
                        { (($val as $uelem) << $shift) as $elem }
                    };
                    (@apply srl, $val:expr, $shift:expr, $width:expr, $elem:ty,
                    $uelem:ty) =>
                    {
                        if $shift >= $width { 0 } else
                        { (($val as $uelem) >> $shift) as $elem }
                    };
                    (@apply sra, $val:expr, $shift:expr, $width:expr, $elem:ty,
                    $uelem:ty) => { $val >> $shift.min($width - 1) };
                }
                impl SimdShiftImpl for SseReg {
                    #[inline(always)]
                    unsafe fn sllv_64(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i64>::default(); 2];
                                let mut b = [<i64>::default(); 2];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i64>() * 8) as u32;
                                let mut out = [<i64>::default(); 2];
                                for i in 0..2 {
                                    let shift = b[i] as u64 as u32;
                                    out[i] =
                                        if shift >= width {
                                            0
                                        } else { ((a[i] as u64) << shift) as i64 };
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_64(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i64>::default(); 2];
                                let mut b = [<i64>::default(); 2];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i64>() * 8) as u32;
                                let mut out = [<i64>::default(); 2];
                                for i in 0..2 {
                                    let shift = b[i] as u64 as u32;
                                    out[i] =
                                        if shift >= width {
                                            0
                                        } else { ((a[i] as u64) >> shift) as i64 };
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_64(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i64>::default(); 2];
                                let mut b = [<i64>::default(); 2];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i64>() * 8) as u32;
                                let mut out = [<i64>::default(); 2];
                                for i in 0..2 {
                                    let shift = b[i] as u64 as u32;
                                    out[i] = a[i] >> shift.min(width - 1);
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn sllv_32(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i32>::default(); 4];
                                let mut b = [<i32>::default(); 4];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i32>() * 8) as u32;
                                let mut out = [<i32>::default(); 4];
                                for i in 0..4 {
                                    let shift = b[i] as u32 as u32;
                                    out[i] =
                                        if shift >= width {
                                            0
                                        } else { ((a[i] as u32) << shift) as i32 };
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_32(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i32>::default(); 4];
                                let mut b = [<i32>::default(); 4];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i32>() * 8) as u32;
                                let mut out = [<i32>::default(); 4];
                                for i in 0..4 {
                                    let shift = b[i] as u32 as u32;
                                    out[i] =
                                        if shift >= width {
                                            0
                                        } else { ((a[i] as u32) >> shift) as i32 };
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_32(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i32>::default(); 4];
                                let mut b = [<i32>::default(); 4];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i32>() * 8) as u32;
                                let mut out = [<i32>::default(); 4];
                                for i in 0..4 {
                                    let shift = b[i] as u32 as u32;
                                    out[i] = a[i] >> shift.min(width - 1);
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn sllv_16(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i16>::default(); 8];
                                let mut b = [<i16>::default(); 8];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i16>() * 8) as u32;
                                let mut out = [<i16>::default(); 8];
                                for i in 0..8 {
                                    let shift = b[i] as u16 as u32;
                                    out[i] =
                                        if shift >= width {
                                            0
                                        } else { ((a[i] as u16) << shift) as i16 };
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_16(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i16>::default(); 8];
                                let mut b = [<i16>::default(); 8];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i16>() * 8) as u32;
                                let mut out = [<i16>::default(); 8];
                                for i in 0..8 {
                                    let shift = b[i] as u16 as u32;
                                    out[i] =
                                        if shift >= width {
                                            0
                                        } else { ((a[i] as u16) >> shift) as i16 };
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_16(self, rhs: Self) -> Self {
                        unsafe {
                            {
                                let mut a = [<i16>::default(); 8];
                                let mut b = [<i16>::default(); 8];
                                _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0);
                                _mm_storeu_si128(b.as_mut_ptr() as *mut __m128i, rhs.0);
                                let width = (std::mem::size_of::<i16>() * 8) as u32;
                                let mut out = [<i16>::default(); 8];
                                for i in 0..8 {
                                    let shift = b[i] as u16 as u32;
                                    out[i] = a[i] >> shift.min(width - 1);
                                }
                                Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                            }
                        }
                    }
                }
                impl SimdLoadImpl for SseReg {
                    type MaskType = Self;
                    #[inline(always)]
                    unsafe fn load_aligned<T>(ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_load_si128(transmute_copy(&ptr))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn load_unaligned<T>(ptr: *const T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_loadu_si128(transmute_copy(&ptr))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_load_64<T>(ptr: *const T,
                        mask: Self::MaskType) -> Self {
                        unsafe {
                            let mut m = [0i64; 2];
                            _mm_storeu_si128(m.as_mut_ptr() as *mut __m128i, mask.0);
                            let src = ptr as *const i64;
                            let mut out = [0i64; 2];
                            for i in 0..2 {
                                if (m[i] as u64) & (1 << 63) != 0 { out[i] = *src.add(i); }
                            }
                            Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_load_32<T>(ptr: *const T,
                        mask: Self::MaskType) -> Self {
                        unsafe {
                            let mut m = [0i32; 4];
                            _mm_storeu_si128(m.as_mut_ptr() as *mut __m128i, mask.0);
                            let src = ptr as *const i32;
                            let mut out = [0i32; 4];
                            for i in 0..4 {
                                if (m[i] as u32) & (1 << 31) != 0 { out[i] = *src.add(i); }
                            }
                            Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                        }
                    }
                }
                impl SimdStoreImpl for SseReg {
                    type MaskType = Self;
                    #[inline(always)]
                    unsafe fn store_aligned<T>(self, ptr: *mut T) {
                        unsafe {
                            _mm_store_si128(transmute_copy(&ptr), transmute_copy(&self))
                        };
                    }
                    #[inline(always)]
                    unsafe fn store_unaligned<T>(self, ptr: *mut T) {
                        unsafe {
                            _mm_storeu_si128(transmute_copy(&ptr),
                                transmute_copy(&self))
                        };
                    }
                    #[inline(always)]
                    unsafe fn masked_store_64<T>(self, ptr: *mut T,
                        mask: Self::MaskType) {
                        unsafe {
                            let mut m = [0i64; 2];
                            let mut v = [0i64; 2];
                            _mm_storeu_si128(m.as_mut_ptr() as *mut __m128i, mask.0);
                            _mm_storeu_si128(v.as_mut_ptr() as *mut __m128i, self.0);
                            let dst = ptr as *mut i64;
                            for i in 0..2 {
                                if (m[i] as u64) & (1 << 63) != 0 { *dst.add(i) = v[i]; }
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_store_32<T>(self, ptr: *mut T,
                        mask: Self::MaskType) {
                        unsafe {
                            let mut m = [0i32; 4];
                            let mut v = [0i32; 4];
                            _mm_storeu_si128(m.as_mut_ptr() as *mut __m128i, mask.0);
                            _mm_storeu_si128(v.as_mut_ptr() as *mut __m128i, self.0);
                            let dst = ptr as *mut i32;
                            for i in 0..4 {
                                if (m[i] as u32) & (1 << 31) != 0 { *dst.add(i) = v[i]; }
                            }
                        }
                    }
                }
                impl SimdZeroImpl for SseReg {
                    #[inline(always)]
                    unsafe fn zero() -> Self {
                        unsafe { Self(transmute_copy(&_mm_setzero_si128())) }
                    }
                }
                impl SimdFloatCastsImpl for SseReg {
                    #[inline(always)]
                    unsafe fn float_to_int_trunc(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cvttps_epi32(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn float_to_int_round(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cvtps_epi32(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdIntCastsImpl for SseReg {
                    #[inline(always)]
                    unsafe fn int_to_float(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cvtepi32_ps(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdPermuteImpl for SseReg {
                    #[inline(always)]
                    unsafe fn permute_32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut a = [0.0f32; 4];
                            let mut idx = [0i32; 4];
                            _mm_storeu_ps(a.as_mut_ptr(), std::mem::transmute(self.0));
                            _mm_storeu_si128(idx.as_mut_ptr() as *mut __m128i, rhs.0);
                            let mut out = [0.0f32; 4];
                            for i in 0..4 { out[i] = a[(idx[i] & 0b11) as usize]; }
                            Self(std::mem::transmute(_mm_loadu_ps(out.as_ptr())))
                        }
                    }
                    #[inline(always)]
                    unsafe fn permute_8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_shuffle_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdVariableBlendImpl for SseReg {
                    type VecType = Self;
                    #[inline(always)]
                    unsafe fn vblend_64(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_blendv_pd(transmute_copy(&false_values),
                                            transmute_copy(&true_values), transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn vblend_32(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_blendv_ps(transmute_copy(&false_values),
                                            transmute_copy(&true_values), transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn vblend_8(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_blendv_epi8(transmute_copy(&false_values),
                                            transmute_copy(&true_values), transmute_copy(&self))))
                        }
                    }
                }
                impl SimdImmediateBlendImpl for SseReg {
                    #[inline(always)]
                    unsafe fn blend_64<const N : i32>(self, false_values: Self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm_blend_pd::<{
                                                N
                                            }>(transmute(false_values), transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn blend_32<const N : i32>(self, false_values: Self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm_blend_ps::<{
                                                N
                                            }>(transmute(false_values), transmute(self))))
                        }
                    }
                }
                impl SimdMulAddImpl for SseReg {
                    #[inline(always)]
                    unsafe fn mul_add_f64(self, mult: Self, add: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_add_pd(transmute_copy(&unsafe {
                                                        Self(transmute_copy(&_mm_mul_pd(transmute_copy(&self),
                                                                        transmute_copy(&mult))))
                                                    }), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_sub_f64(self, mult: Self, sub: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_pd(transmute_copy(&unsafe {
                                                        Self(transmute_copy(&_mm_mul_pd(transmute_copy(&self),
                                                                        transmute_copy(&mult))))
                                                    }), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_add_f64(self, mult: Self, add: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_pd(transmute_copy(&add),
                                            transmute_copy(&unsafe {
                                                        Self(transmute_copy(&_mm_mul_pd(transmute_copy(&self),
                                                                        transmute_copy(&mult))))
                                                    }))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_sub_f64(self, mult: Self, sub: Self)
                        -> Self {
                        let neg =
                            unsafe {
                                Self(transmute_copy(&_mm_mul_pd(transmute_copy(&self),
                                                transmute_copy(&mult))))
                            };
                        unsafe {
                            Self(transmute_copy(&_mm_sub_pd(transmute_copy(&unsafe {
                                                        Self(transmute_copy(&_mm_xor_pd(transmute_copy(&neg),
                                                                        transmute_copy(&Self::splat_64(-0.0f64).0))))
                                                    }), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_add_f32(self, mult: Self, add: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_add_ps(transmute_copy(&unsafe {
                                                        Self(transmute_copy(&_mm_mul_ps(transmute_copy(&self),
                                                                        transmute_copy(&mult))))
                                                    }), transmute_copy(&add))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_sub_f32(self, mult: Self, sub: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_ps(transmute_copy(&unsafe {
                                                        Self(transmute_copy(&_mm_mul_ps(transmute_copy(&self),
                                                                        transmute_copy(&mult))))
                                                    }), transmute_copy(&sub))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_add_f32(self, mult: Self, add: Self)
                        -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sub_ps(transmute_copy(&add),
                                            transmute_copy(&unsafe {
                                                        Self(transmute_copy(&_mm_mul_ps(transmute_copy(&self),
                                                                        transmute_copy(&mult))))
                                                    }))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_sub_f32(self, mult: Self, sub: Self)
                        -> Self {
                        let neg =
                            unsafe {
                                Self(transmute_copy(&_mm_mul_ps(transmute_copy(&self),
                                                transmute_copy(&mult))))
                            };
                        unsafe {
                            Self(transmute_copy(&_mm_sub_ps(transmute_copy(&unsafe {
                                                        Self(transmute_copy(&_mm_xor_ps(transmute_copy(&neg),
                                                                        transmute_copy(&Self::splat_32(-0.0f32).0))))
                                                    }), transmute_copy(&sub))))
                        }
                    }
                }
                impl SimdRoundImpl for SseReg {
                    #[inline(always)]
                    unsafe fn round_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm_round_pd::<{
                                                _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn round_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm_round_ps::<{
                                                _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn floor_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm_round_pd::<{
                                                _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn floor_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm_round_ps::<{
                                                _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn ceil_f64(self) -> Self {
                        unsafe {
                            Self(transmute(_mm_round_pd::<{
                                                _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn ceil_f32(self) -> Self {
                        unsafe {
                            Self(transmute(_mm_round_ps::<{
                                                _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC
                                            }>(transmute(self))))
                        }
                    }
                }
                impl SimdPartialOrdImpl for SseReg {
                    type MaskType = Self;
                    #[inline(always)]
                    unsafe fn cmp_f64_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpeq_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_lt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmplt_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_le(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmple_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpgt_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_ge(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpge_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_neq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpneq_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpeq_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_lt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmplt_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_le(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmple_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpgt_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_ge(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpge_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_neq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpneq_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i64_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpeq_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i64_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpgt_epi64(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i32_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpeq_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i32_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpgt_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i16_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpeq_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i16_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpgt_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i8_eq(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpeq_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i8_gt(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_cmpgt_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_f64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_max_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_f64(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_min_pd(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_f32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_max_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_f32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_min_ps(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_max_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_min_epi32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_max_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_min_epi16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_max_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_min_epi8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_max_epu32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u32(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_min_epu32(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_max_epu16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u16(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_min_epu16(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_max_epu8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u8(self, rhs: Self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_min_epu8(transmute_copy(&self),
                                            transmute_copy(&rhs))))
                        }
                    }
                }
                impl SimdSplatImpl for SseReg {
                    #[inline(always)]
                    unsafe fn splat_64<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_set1_epi64x(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_32<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_set1_epi32(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_16<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_set1_epi16(transmute_copy(&val))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_8<T>(val: T) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_set1_epi8(transmute_copy(&val))))
                        }
                    }
                }
                impl SimdGatherImpl for SseReg {
                    #[inline(always)]
                    unsafe fn gather_32_from_32<T, const B :
                        i32>(self, ptr: *const T) -> Self {
                        unsafe {
                            let mut idx = [0i32; 4];
                            _mm_storeu_si128(idx.as_mut_ptr() as *mut __m128i, self.0);
                            let base = ptr as *const u8;
                            let mut out = [0i32; 4];
                            for i in 0..4 {
                                let byte_offset = (idx[i] as isize) * (B as isize);
                                out[i] = *(base.offset(byte_offset) as *const i32);
                            }
                            Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                        }
                    }
                    #[inline(always)]
                    unsafe fn gather_64_from_64<T, const B :
                        i32>(self, ptr: *const T) -> Self {
                        unsafe {
                            let mut idx = [0i64; 2];
                            _mm_storeu_si128(idx.as_mut_ptr() as *mut __m128i, self.0);
                            let base = ptr as *const u8;
                            let mut out = [0i64; 2];
                            for i in 0..2 {
                                let byte_offset = (idx[i] as isize) * (B as isize);
                                out[i] = *(base.offset(byte_offset) as *const i64);
                            }
                            Self(_mm_loadu_si128(out.as_ptr() as *const __m128i))
                        }
                    }
                }
                impl SimdSqrtImpl for SseReg {
                    #[inline(always)]
                    unsafe fn sqrt_f64(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sqrt_pd(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn sqrt_f32(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_sqrt_ps(transmute_copy(&self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn rsqrt_f32(self) -> Self {
                        unsafe {
                            Self(transmute_copy(&_mm_rsqrt_ps(transmute_copy(&self))))
                        }
                    }
                }
                impl SimdAllBitsImpl for SseReg {
                    #[inline(always)]
                    unsafe fn all_zero(self) -> bool {
                        (unsafe {
                                _mm_testz_si128(transmute_copy(&self),
                                    transmute_copy(&self))
                            }) == 0
                    }
                }
                impl SimdNegateImpl for SseReg {
                    #[inline(always)]
                    unsafe fn negate_f64(self) -> Self {
                        unsafe { Self::splat_64(-0.0f64).xor(self) }
                    }
                    #[inline(always)]
                    unsafe fn negate_f32(self) -> Self {
                        unsafe { Self::splat_32(-0.0f64).xor(self) }
                    }
                }
                impl SimdBlockShiftImpl for SseReg {
                    #[inline(always)]
                    unsafe fn block_left_byte_shift<const N : i32>(self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm_bslli_si128::<{ N }>(transmute(self))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn block_right_byte_shift<const N : i32>(self)
                        -> Self {
                        unsafe {
                            Self(transmute(_mm_bsrli_si128::<{ N }>(transmute(self))))
                        }
                    }
                }
                impl SimdMaskBitConversion for SseReg {
                    #[inline(always)]
                    unsafe fn to_bits_64(self) -> u64 {
                        (unsafe { _mm_movemask_pd(transmute_copy(&self)) }) as u64
                    }
                    #[inline(always)]
                    unsafe fn to_bits_32(self) -> u64 {
                        (unsafe { _mm_movemask_ps(transmute_copy(&self)) }) as u64
                    }
                    #[inline(always)]
                    unsafe fn to_bits_8(self) -> u64 {
                        (unsafe { _mm_movemask_epi8(transmute_copy(&self)) }) as u64
                    }
                    #[inline(always)]
                    unsafe fn from_bits_64(bitmask: u64) -> Self {
                        let mask =
                            unsafe {
                                Self(transmute_copy(&_mm_set_epi64x(transmute_copy(&1),
                                                transmute_copy(&2))))
                            };
                        let bits =
                            unsafe {
                                Self(transmute_copy(&_mm_set1_epi64x(transmute_copy(&(bitmask
                                                            as i64)))))
                            };
                        unsafe {
                            Self(transmute_copy(&_mm_cmpgt_epi64(transmute_copy(&bits.and(mask)),
                                            transmute_copy(&Self::zero()))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_32(bitmask: u64) -> Self {
                        let mask =
                            unsafe {
                                Self(transmute_copy(&_mm_set_epi32(transmute_copy(&1),
                                                transmute_copy(&2), transmute_copy(&4),
                                                transmute_copy(&8))))
                            };
                        let bits =
                            unsafe {
                                Self(transmute_copy(&_mm_set1_epi32(transmute_copy(&(bitmask
                                                            as u32)))))
                            };
                        unsafe {
                            Self(transmute_copy(&_mm_cmpgt_epi32(transmute_copy(&bits.and(mask)),
                                            transmute_copy(&Self::zero()))))
                        }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_16(bitmask: u64) -> Self {
                        #[rustfmt::skip]
                        let mask =
                            unsafe {
                                Self(transmute_copy(&_mm_set_epi16(transmute_copy(&1),
                                                transmute_copy(&2), transmute_copy(&4), transmute_copy(&8),
                                                transmute_copy(&16), transmute_copy(&32),
                                                transmute_copy(&64), transmute_copy(&128))))
                            };
                        let bits =
                            unsafe {
                                Self(transmute_copy(&_mm_set1_epi16(transmute_copy(&(bitmask
                                                            as i16)))))
                            };
                        unsafe {
                            unsafe {
                                    Self(transmute_copy(&_mm_cmpeq_epi16(transmute_copy(&bits.and(mask)),
                                                    transmute_copy(&Self::zero()))))
                                }.not()
                        }
                    }
                    #[inline(always)]
                    unsafe fn from_bits_8(bitmask: u64) -> Self {
                        let b1 = bitmask as i8;
                        let b2 = (bitmask >> 8) as i8;
                        #[rustfmt::skip]
                        let mask =
                            unsafe {
                                Self(transmute_copy(&_mm_set_epi8(transmute_copy(&1),
                                                transmute_copy(&2), transmute_copy(&4), transmute_copy(&8),
                                                transmute_copy(&16), transmute_copy(&32),
                                                transmute_copy(&64), transmute_copy(&-128),
                                                transmute_copy(&1), transmute_copy(&2), transmute_copy(&4),
                                                transmute_copy(&8), transmute_copy(&16),
                                                transmute_copy(&32), transmute_copy(&64),
                                                transmute_copy(&-128))))
                            };
                        #[rustfmt::skip]
                        let bits =
                            unsafe {
                                Self(transmute_copy(&_mm_set_epi8(transmute_copy(&b1),
                                                transmute_copy(&b1), transmute_copy(&b1),
                                                transmute_copy(&b1), transmute_copy(&b1),
                                                transmute_copy(&b1), transmute_copy(&b1),
                                                transmute_copy(&b1), transmute_copy(&b2),
                                                transmute_copy(&b2), transmute_copy(&b2),
                                                transmute_copy(&b2), transmute_copy(&b2),
                                                transmute_copy(&b2), transmute_copy(&b2),
                                                transmute_copy(&b2))))
                            };
                        unsafe {
                            unsafe {
                                    Self(transmute_copy(&_mm_cmpeq_epi8(transmute_copy(&bits.and(mask)),
                                                    transmute_copy(&Self::zero()))))
                                }.not()
                        }
                    }
                }
                impl SimdLaneShiftImpl for SseReg {
                    #[inline(always)]
                    unsafe fn left_lane_shift_32(self, n: u32) -> Self {
                        match n {
                            0 => self,
                            1 => unsafe {
                                Self(transmute(_mm_bsrli_si128::<{ 4 }>(transmute(self))))
                            },
                            2 => unsafe {
                                Self(transmute(_mm_bsrli_si128::<{ 8 }>(transmute(self))))
                            },
                            3 => unsafe {
                                Self(transmute(_mm_bsrli_si128::<{ 12 }>(transmute(self))))
                            },
                            _ => unsafe { Self::zero() },
                        }
                    }
                    #[inline(always)]
                    unsafe fn right_lane_shift_32(self, n: u32) -> Self {
                        match n {
                            0 => self,
                            1 => unsafe {
                                Self(transmute(_mm_bslli_si128::<{ 4 }>(transmute(self))))
                            },
                            2 => unsafe {
                                Self(transmute(_mm_bslli_si128::<{ 8 }>(transmute(self))))
                            },
                            3 => unsafe {
                                Self(transmute(_mm_bslli_si128::<{ 12 }>(transmute(self))))
                            },
                            _ => unsafe { Self::zero() },
                        }
                    }
                }
            }
            pub mod scalar {
                use std::mem::transmute_copy;
                use crate::simd::architectures::interface::*;
                #[repr(align(8))]
                pub struct ScalarReg<const N : usize>(pub [u8; N]);
                #[automatically_derived]
                impl<const N : usize> ::core::marker::Copy for ScalarReg<N> {
                }
                #[automatically_derived]
                #[doc(hidden)]
                unsafe impl<const N : usize> ::core::clone::TrivialClone for
                    ScalarReg<N> {
                }
                #[automatically_derived]
                impl<const N : usize> ::core::clone::Clone for ScalarReg<N> {
                    #[inline]
                    fn clone(&self) -> ScalarReg<N> {
                        let _: ::core::clone::AssertParamIsClone<[u8; N]>;
                        *self
                    }
                }
                pub struct ScalarMask<const N : usize>(pub [bool; N]);
                #[automatically_derived]
                impl<const N : usize> ::core::marker::Copy for ScalarMask<N> {
                }
                #[automatically_derived]
                #[doc(hidden)]
                unsafe impl<const N : usize> ::core::clone::TrivialClone for
                    ScalarMask<N> {
                }
                #[automatically_derived]
                impl<const N : usize> ::core::clone::Clone for ScalarMask<N> {
                    #[inline]
                    fn clone(&self) -> ScalarMask<N> {
                        let _: ::core::clone::AssertParamIsClone<[bool; N]>;
                        *self
                    }
                }
                impl<const N : usize> SimdArch for ScalarReg<N> {}
                impl<const N : usize> MaskArch for ScalarMask<N> {}
                macro_rules! scalar_token_op {
                    ($type:ty, $op:tt, $self:ident, $rhs:ident, $size:expr) =>
                    {
                        unsafe
                        {
                            let mut new = ScalarReg::<$size>([0; $size]); let self_ptr:
                            *const $type = $self.0.as_ptr() as *const $type; let
                            rhs_ptr: *const $type = $rhs.0.as_ptr() as *const $type; let
                            new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type; for
                            i in 0..($size / size_of::<$type>())
                            { *new_ptr.add(i) = *self_ptr.add(i) $op *rhs_ptr.add(i); }
                            new
                        }
                    }
                }
                macro_rules! scalar_token_op_usize_rhs {
                    ($type:ty, $op:tt, $self:ident, $rhs:ident, $size:expr) =>
                    {
                        unsafe
                        {
                            let mut new = ScalarReg::<$size>([0; $size]); let self_ptr:
                            *const $type = $self.0.as_ptr() as *const $type; let
                            rhs_ptr: *const $type = $rhs.0.as_ptr() as *const $type; let
                            new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type; for
                            i in 0..($size / size_of::<$type>())
                            {
                                *new_ptr.add(i) = *self_ptr.add(i)
                                $op(*rhs_ptr.add(i) as usize);
                            } new
                        }
                    }
                }
                macro_rules! scalar_func_op {
                    ($type:ty, $op:ident, $self:ident, $rhs:ident, $size:expr)
                    =>
                    {
                        unsafe
                        {
                            let mut new = ScalarReg::<$size>([0; $size]); let self_ptr:
                            *const $type = $self.0.as_ptr() as *const $type; let
                            rhs_ptr: *const $type = $rhs.0.as_ptr() as *const $type; let
                            new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type; for
                            i in 0..($size / size_of::<$type>())
                            {
                                *new_ptr.add(i) = (*self_ptr.add(i)).$op(*rhs_ptr.add(i));
                            } new
                        }
                    };
                }
                macro_rules! scalar_self_op {
                    ($type:ty, $op:ident, $self:ident, $size:expr) =>
                    {
                        unsafe
                        {
                            let mut new = ScalarReg::<$size>([0; $size]); let self_ptr:
                            *const $type = $self.0.as_ptr() as *const $type; let
                            new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type; for
                            i in 0..($size / size_of::<$type>())
                            { *new_ptr.add(i) = (*self_ptr.add(i)).$op(); } new
                        }
                    };
                }
                macro_rules! scalar_fma_expr_op {
                    ($type:ty, $self:ident, $mult:ident, $add:ident, $size:expr,
                    |$a:ident, $b:ident, $c:ident| $op:expr) =>
                    {
                        unsafe
                        {
                            let mut new = ScalarReg::<$size>([0; $size]); let a_ptr:
                            *const $type = $self.0.as_ptr() as *const $type; let b_ptr:
                            *const $type = $mult.0.as_ptr() as *const $type; let c_ptr:
                            *const $type = $add.0.as_ptr() as *const $type; let new_ptr:
                            *mut $type = new.0.as_mut_ptr() as *mut $type; for i in
                            0..($size / size_of::<$type>())
                            {
                                let $a = *a_ptr.add(i); let $b = *b_ptr.add(i); let $c =
                                *c_ptr.add(i); *new_ptr.add(i) = $op;
                            } new
                        }
                    };
                }
                macro_rules! scalar_cmp {
                    {$type:ty, $op:tt, $self:ident, $rhs:ident, $size:expr} =>
                    {
                        unsafe
                        {
                            let mut result = ScalarMask::<$size>([false; $size]); let
                            self_ptr = $self.0.as_ptr() as *const $type; let rhs_ptr =
                            $rhs.0.as_ptr() as *const $type; for i in
                            0..($size / size_of::<$type>())
                            { result.0[i] = *self_ptr.add(i) $op *rhs_ptr.add(i); }
                            result
                        }
                    }
                }
                macro_rules! scalar_splat {
                    {$type:ty, $self:ident, $val:expr, $size:expr} =>
                    {
                        unsafe
                        {
                            let mut new = ScalarReg::<$size>([0; $size]); let new_ptr =
                            new.0.as_mut_ptr() as *mut $type; for i in
                            0..(N / size_of::<$type>()) { *new_ptr.add(i) = $val; } new
                        }
                    }
                }
                impl<const N : usize> SimdAddImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn f64_add(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let rhs_ptr: *const f64 = rhs.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) + *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_add(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let rhs_ptr: *const f32 = rhs.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) + *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i64_add(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i64 = self.0.as_ptr() as *const i64;
                            let rhs_ptr: *const i64 = rhs.0.as_ptr() as *const i64;
                            let new_ptr: *mut i64 = new.0.as_mut_ptr() as *mut i64;
                            for i in 0..(N / size_of::<i64>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_add(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_add(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i32 = self.0.as_ptr() as *const i32;
                            let rhs_ptr: *const i32 = rhs.0.as_ptr() as *const i32;
                            let new_ptr: *mut i32 = new.0.as_mut_ptr() as *mut i32;
                            for i in 0..(N / size_of::<i32>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_add(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_add(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i16 = self.0.as_ptr() as *const i16;
                            let rhs_ptr: *const i16 = rhs.0.as_ptr() as *const i16;
                            let new_ptr: *mut i16 = new.0.as_mut_ptr() as *mut i16;
                            for i in 0..(N / size_of::<i16>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_add(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i8_add(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i8 = self.0.as_ptr() as *const i8;
                            let rhs_ptr: *const i8 = rhs.0.as_ptr() as *const i8;
                            let new_ptr: *mut i8 = new.0.as_mut_ptr() as *mut i8;
                            for i in 0..(N / size_of::<i8>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_add(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdSubImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn f64_sub(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let rhs_ptr: *const f64 = rhs.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) - *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_sub(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let rhs_ptr: *const f32 = rhs.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) - *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i64_sub(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i64 = self.0.as_ptr() as *const i64;
                            let rhs_ptr: *const i64 = rhs.0.as_ptr() as *const i64;
                            let new_ptr: *mut i64 = new.0.as_mut_ptr() as *mut i64;
                            for i in 0..(N / size_of::<i64>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_sub(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_sub(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i32 = self.0.as_ptr() as *const i32;
                            let rhs_ptr: *const i32 = rhs.0.as_ptr() as *const i32;
                            let new_ptr: *mut i32 = new.0.as_mut_ptr() as *mut i32;
                            for i in 0..(N / size_of::<i32>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_sub(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_sub(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i16 = self.0.as_ptr() as *const i16;
                            let rhs_ptr: *const i16 = rhs.0.as_ptr() as *const i16;
                            let new_ptr: *mut i16 = new.0.as_mut_ptr() as *mut i16;
                            for i in 0..(N / size_of::<i16>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_sub(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i8_sub(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i8 = self.0.as_ptr() as *const i8;
                            let rhs_ptr: *const i8 = rhs.0.as_ptr() as *const i8;
                            let new_ptr: *mut i8 = new.0.as_mut_ptr() as *mut i8;
                            for i in 0..(N / size_of::<i8>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_sub(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdMulImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn f64_mul(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let rhs_ptr: *const f64 = rhs.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) * *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_mul(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let rhs_ptr: *const f32 = rhs.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) * *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i32_mul(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i32 = self.0.as_ptr() as *const i32;
                            let rhs_ptr: *const i32 = rhs.0.as_ptr() as *const i32;
                            let new_ptr: *mut i32 = new.0.as_mut_ptr() as *mut i32;
                            for i in 0..(N / size_of::<i32>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_mul(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn i16_mul(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i16 = self.0.as_ptr() as *const i16;
                            let rhs_ptr: *const i16 = rhs.0.as_ptr() as *const i16;
                            let new_ptr: *mut i16 = new.0.as_mut_ptr() as *mut i16;
                            for i in 0..(N / size_of::<i16>()) {
                                *new_ptr.add(i) =
                                    (*self_ptr.add(i)).wrapping_mul(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdDivImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn f64_div(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let rhs_ptr: *const f64 = rhs.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) / *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn f32_div(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let rhs_ptr: *const f32 = rhs.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) / *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdBitwiseImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn and(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u64 = self.0.as_ptr() as *const u64;
                            let rhs_ptr: *const u64 = rhs.0.as_ptr() as *const u64;
                            let new_ptr: *mut u64 = new.0.as_mut_ptr() as *mut u64;
                            for i in 0..(N / size_of::<u64>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) & *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn or(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u64 = self.0.as_ptr() as *const u64;
                            let rhs_ptr: *const u64 = rhs.0.as_ptr() as *const u64;
                            let new_ptr: *mut u64 = new.0.as_mut_ptr() as *mut u64;
                            for i in 0..(N / size_of::<u64>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) | *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn xor(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u64 = self.0.as_ptr() as *const u64;
                            let rhs_ptr: *const u64 = rhs.0.as_ptr() as *const u64;
                            let new_ptr: *mut u64 = new.0.as_mut_ptr() as *mut u64;
                            for i in 0..(N / size_of::<u64>()) {
                                *new_ptr.add(i) = *self_ptr.add(i) ^ *rhs_ptr.add(i);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn not(self) -> Self {
                        unsafe { self.xor(Self([255; N])) }
                    }
                    #[inline(always)]
                    unsafe fn and_not(self, rhs: Self) -> Self {
                        unsafe { self.and(rhs.not()) }
                    }
                }
                impl<const N : usize> SimdShiftImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn sllv_64(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u64 = self.0.as_ptr() as *const u64;
                            let rhs_ptr: *const u64 = rhs.0.as_ptr() as *const u64;
                            let new_ptr: *mut u64 = new.0.as_mut_ptr() as *mut u64;
                            for i in 0..(N / size_of::<u64>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) << (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_64(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u64 = self.0.as_ptr() as *const u64;
                            let rhs_ptr: *const u64 = rhs.0.as_ptr() as *const u64;
                            let new_ptr: *mut u64 = new.0.as_mut_ptr() as *mut u64;
                            for i in 0..(N / size_of::<u64>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) >> (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_64(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i64 = self.0.as_ptr() as *const i64;
                            let rhs_ptr: *const i64 = rhs.0.as_ptr() as *const i64;
                            let new_ptr: *mut i64 = new.0.as_mut_ptr() as *mut i64;
                            for i in 0..(N / size_of::<i64>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) >> (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn sllv_32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u32 = self.0.as_ptr() as *const u32;
                            let rhs_ptr: *const u32 = rhs.0.as_ptr() as *const u32;
                            let new_ptr: *mut u32 = new.0.as_mut_ptr() as *mut u32;
                            for i in 0..(N / size_of::<u32>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) << (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u32 = self.0.as_ptr() as *const u32;
                            let rhs_ptr: *const u32 = rhs.0.as_ptr() as *const u32;
                            let new_ptr: *mut u32 = new.0.as_mut_ptr() as *mut u32;
                            for i in 0..(N / size_of::<u32>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) >> (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i32 = self.0.as_ptr() as *const i32;
                            let rhs_ptr: *const i32 = rhs.0.as_ptr() as *const i32;
                            let new_ptr: *mut i32 = new.0.as_mut_ptr() as *mut i32;
                            for i in 0..(N / size_of::<i32>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) >> (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn sllv_16(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u16 = self.0.as_ptr() as *const u16;
                            let rhs_ptr: *const u16 = rhs.0.as_ptr() as *const u16;
                            let new_ptr: *mut u16 = new.0.as_mut_ptr() as *mut u16;
                            for i in 0..(N / size_of::<u16>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) << (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn srlv_16(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u16 = self.0.as_ptr() as *const u16;
                            let rhs_ptr: *const u16 = rhs.0.as_ptr() as *const u16;
                            let new_ptr: *mut u16 = new.0.as_mut_ptr() as *mut u16;
                            for i in 0..(N / size_of::<u16>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) >> (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn srav_16(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i16 = self.0.as_ptr() as *const i16;
                            let rhs_ptr: *const i16 = rhs.0.as_ptr() as *const i16;
                            let new_ptr: *mut i16 = new.0.as_mut_ptr() as *mut i16;
                            for i in 0..(N / size_of::<i16>()) {
                                *new_ptr.add(i) =
                                    *self_ptr.add(i) >> (*rhs_ptr.add(i) as usize);
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdLoadImpl for ScalarReg<N> {
                    type MaskType = ScalarMask<N>;
                    #[inline(always)]
                    unsafe fn load_aligned<T>(ptr: *const T) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            std::ptr::copy_nonoverlapping(ptr as *const u8,
                                new.0.as_mut_ptr(), N);
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn load_unaligned<T>(ptr: *const T) -> Self {
                        unsafe { Self::load_aligned::<T>(ptr) }
                    }
                    #[inline(always)]
                    unsafe fn masked_load_64<T>(ptr: *const T,
                        mask: Self::MaskType) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u64;
                            let data_ptr = ptr as *const u64;
                            for i in 0..(N / 8) {
                                *new_ptr.add(i) =
                                    if mask.0[i] { *data_ptr.add(i) } else { 0 };
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_load_32<T>(ptr: *const T,
                        mask: Self::MaskType) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u32;
                            let data_ptr = ptr as *const u32;
                            for i in 0..(N / 4) {
                                *new_ptr.add(i) =
                                    if mask.0[i] { *data_ptr.add(i) } else { 0 };
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdStoreImpl for ScalarReg<N> {
                    type MaskType = ScalarMask<N>;
                    #[inline(always)]
                    unsafe fn store_aligned<T>(self, ptr: *mut T) {
                        unsafe {
                            std::ptr::copy_nonoverlapping(self.0.as_ptr(),
                                ptr as *mut u8, N);
                        }
                    }
                    #[inline(always)]
                    unsafe fn store_unaligned<T>(self, ptr: *mut T) {
                        unsafe { self.store_aligned(ptr) };
                    }
                    #[inline(always)]
                    unsafe fn masked_store_64<T>(self, ptr: *mut T,
                        mask: Self::MaskType) {
                        unsafe {
                            let self_ptr = self.0.as_ptr() as *const u64;
                            let store_ptr = ptr as *mut u64;
                            for i in 0..(N / 8) {
                                if mask.0[i] { *store_ptr.add(i) = *self_ptr.add(i); }
                            }
                        }
                    }
                    #[inline(always)]
                    unsafe fn masked_store_32<T>(self, ptr: *mut T,
                        mask: Self::MaskType) {
                        unsafe {
                            let self_ptr = self.0.as_ptr() as *const u32;
                            let store_ptr = ptr as *mut u32;
                            for i in 0..(N / 4) {
                                if mask.0[i] { *store_ptr.add(i) = *self_ptr.add(i); }
                            }
                        }
                    }
                }
                impl<const N : usize> SimdZeroImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn zero() -> Self { Self([0; N]) }
                }
                impl<const N : usize> SimdFloatCastsImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn float_to_int_trunc(self) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let float_ptr = self.0.as_ptr() as *const f32;
                            let int_ptr = new.0.as_mut_ptr() as *mut i32;
                            for i in 0..(N / 4) {
                                *int_ptr.add(i) = (*float_ptr.add(i)) as i32;
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn float_to_int_round(self) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let float_ptr = self.0.as_ptr() as *const f32;
                            let int_ptr = new.0.as_mut_ptr() as *mut i32;
                            for i in 0..(N / 4) {
                                *int_ptr.add(i) =
                                    (*float_ptr.add(i)).round_ties_even() as i32;
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdIntCastsImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn int_to_float(self) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let int_ptr = self.0.as_ptr() as *const i32;
                            let float_ptr = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / 4) {
                                *float_ptr.add(i) = (*int_ptr.add(i)) as f32;
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdPermuteImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn permute_32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u32;
                            let self_ptr = self.0.as_ptr() as *const u32;
                            let indices_ptr = rhs.0.as_ptr() as *const u32;
                            for i in 0..(N / 4) {
                                let index = *indices_ptr.add(i) as usize;
                                *new_ptr.add(i) =
                                    if index < (N / 4) { *self_ptr.add(index) } else { 0 };
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn permute_8(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let new_ptr = new.0.as_mut_ptr();
                            let self_ptr = self.0.as_ptr();
                            let indices_ptr = rhs.0.as_ptr();
                            for i in 0..N {
                                let lane_base = i & !15;
                                let index = (*indices_ptr.add(i) as usize & 15) + lane_base;
                                *new_ptr.add(i) = *self_ptr.add(index);
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdVariableBlendImpl for ScalarMask<N>
                    {
                    type VecType = ScalarReg<N>;
                    #[inline(always)]
                    unsafe fn vblend_64(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> ScalarReg<N> {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u64;
                            let false_ptr = false_values.0.as_ptr() as *const u64;
                            let true_ptr = true_values.0.as_ptr() as *const u64;
                            for i in 0..(N / 8) {
                                *new_ptr.add(i) =
                                    if self.0[i] {
                                        *true_ptr.add(i)
                                    } else { *false_ptr.add(i) };
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn vblend_32(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> ScalarReg<N> {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u32;
                            let false_ptr = false_values.0.as_ptr() as *const u32;
                            let true_ptr = true_values.0.as_ptr() as *const u32;
                            for i in 0..(N / 4) {
                                *new_ptr.add(i) =
                                    if self.0[i] {
                                        *true_ptr.add(i)
                                    } else { *false_ptr.add(i) };
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn vblend_8(self, true_values: Self::VecType,
                        false_values: Self::VecType) -> ScalarReg<N> {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let new_ptr = new.0.as_mut_ptr();
                            let false_ptr = false_values.0.as_ptr();
                            let true_ptr = true_values.0.as_ptr();
                            for i in 0..N {
                                *new_ptr.add(i) =
                                    if self.0[i] {
                                        *true_ptr.add(i)
                                    } else { *false_ptr.add(i) };
                            }
                            new
                        }
                    }
                }
                impl<const M : usize> SimdImmediateBlendImpl for ScalarReg<M>
                    {
                    #[inline(always)]
                    unsafe fn blend_64<const N : i32>(self, false_values: Self)
                        -> Self {
                        unsafe {
                            let mut new = ScalarReg::<M>([0; M]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u64;
                            let false_ptr = false_values.0.as_ptr() as *const u64;
                            let true_ptr = self.0.as_ptr() as *const u64;
                            for i in 0..(M >> 3) {
                                let cond = ((N >> i) & 1) == 1;
                                *new_ptr.add(i) =
                                    if cond { *true_ptr.add(i) } else { *false_ptr.add(i) };
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn blend_32<const N : i32>(self, false_values: Self)
                        -> Self {
                        unsafe {
                            let mut new = ScalarReg::<M>([0; M]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u32;
                            let false_ptr = false_values.0.as_ptr() as *const u32;
                            let true_ptr = self.0.as_ptr() as *const u32;
                            for i in 0..(M >> 2) {
                                let cond = ((N >> i) & 1) == 1;
                                *new_ptr.add(i) =
                                    if cond { *true_ptr.add(i) } else { *false_ptr.add(i) };
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdMulAddImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn mul_add_f64(self, mult: Self, add: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let a_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let b_ptr: *const f64 = mult.0.as_ptr() as *const f64;
                            let c_ptr: *const f64 = add.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                let a = *a_ptr.add(i);
                                let b = *b_ptr.add(i);
                                let c = *c_ptr.add(i);
                                *new_ptr.add(i) = f64::mul_add(a, b, c);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_sub_f64(self, mult: Self, sub: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let a_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let b_ptr: *const f64 = mult.0.as_ptr() as *const f64;
                            let c_ptr: *const f64 = sub.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                let a = *a_ptr.add(i);
                                let b = *b_ptr.add(i);
                                let c = *c_ptr.add(i);
                                *new_ptr.add(i) = f64::mul_add(a, b, -c);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_add_f64(self, mult: Self, add: Self)
                        -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let a_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let b_ptr: *const f64 = mult.0.as_ptr() as *const f64;
                            let c_ptr: *const f64 = add.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                let a = *a_ptr.add(i);
                                let b = *b_ptr.add(i);
                                let c = *c_ptr.add(i);
                                *new_ptr.add(i) = f64::mul_add(-a, b, c);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_sub_f64(self, mult: Self, sub: Self)
                        -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let a_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let b_ptr: *const f64 = mult.0.as_ptr() as *const f64;
                            let c_ptr: *const f64 = sub.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                let a = *a_ptr.add(i);
                                let b = *b_ptr.add(i);
                                let c = *c_ptr.add(i);
                                *new_ptr.add(i) = f64::mul_add(-a, b, -c);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_add_f32(self, mult: Self, add: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let a_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let b_ptr: *const f32 = mult.0.as_ptr() as *const f32;
                            let c_ptr: *const f32 = add.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                let a = *a_ptr.add(i);
                                let b = *b_ptr.add(i);
                                let c = *c_ptr.add(i);
                                *new_ptr.add(i) = f32::mul_add(a, b, c);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn mul_sub_f32(self, mult: Self, sub: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let a_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let b_ptr: *const f32 = mult.0.as_ptr() as *const f32;
                            let c_ptr: *const f32 = sub.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                let a = *a_ptr.add(i);
                                let b = *b_ptr.add(i);
                                let c = *c_ptr.add(i);
                                *new_ptr.add(i) = f32::mul_add(a, b, -c);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_add_f32(self, mult: Self, add: Self)
                        -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let a_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let b_ptr: *const f32 = mult.0.as_ptr() as *const f32;
                            let c_ptr: *const f32 = add.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                let a = *a_ptr.add(i);
                                let b = *b_ptr.add(i);
                                let c = *c_ptr.add(i);
                                *new_ptr.add(i) = f32::mul_add(-a, b, c);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn negated_mul_sub_f32(self, mult: Self, sub: Self)
                        -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let a_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let b_ptr: *const f32 = mult.0.as_ptr() as *const f32;
                            let c_ptr: *const f32 = sub.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                let a = *a_ptr.add(i);
                                let b = *b_ptr.add(i);
                                let c = *c_ptr.add(i);
                                *new_ptr.add(i) = f32::mul_add(-a, b, -c);
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdRoundImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn round_f64(self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).round_ties_even();
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn round_f32(self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).round_ties_even();
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn floor_f64(self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).floor();
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn floor_f32(self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).floor();
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn ceil_f64(self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).ceil();
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn ceil_f32(self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).ceil();
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdPartialOrdImpl for ScalarReg<N> {
                    type MaskType = ScalarMask<N>;
                    #[inline(always)]
                    unsafe fn cmp_f64_eq(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f64;
                            let rhs_ptr = rhs.0.as_ptr() as *const f64;
                            for i in 0..(N / size_of::<f64>()) {
                                result.0[i] = *self_ptr.add(i) == *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_lt(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f64;
                            let rhs_ptr = rhs.0.as_ptr() as *const f64;
                            for i in 0..(N / size_of::<f64>()) {
                                result.0[i] = *self_ptr.add(i) < *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_le(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f64;
                            let rhs_ptr = rhs.0.as_ptr() as *const f64;
                            for i in 0..(N / size_of::<f64>()) {
                                result.0[i] = *self_ptr.add(i) <= *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_gt(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f64;
                            let rhs_ptr = rhs.0.as_ptr() as *const f64;
                            for i in 0..(N / size_of::<f64>()) {
                                result.0[i] = *self_ptr.add(i) > *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_ge(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f64;
                            let rhs_ptr = rhs.0.as_ptr() as *const f64;
                            for i in 0..(N / size_of::<f64>()) {
                                result.0[i] = *self_ptr.add(i) >= *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f64_neq(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f64;
                            let rhs_ptr = rhs.0.as_ptr() as *const f64;
                            for i in 0..(N / size_of::<f64>()) {
                                result.0[i] = *self_ptr.add(i) != *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_eq(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f32;
                            let rhs_ptr = rhs.0.as_ptr() as *const f32;
                            for i in 0..(N / size_of::<f32>()) {
                                result.0[i] = *self_ptr.add(i) == *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_lt(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f32;
                            let rhs_ptr = rhs.0.as_ptr() as *const f32;
                            for i in 0..(N / size_of::<f32>()) {
                                result.0[i] = *self_ptr.add(i) < *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_le(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f32;
                            let rhs_ptr = rhs.0.as_ptr() as *const f32;
                            for i in 0..(N / size_of::<f32>()) {
                                result.0[i] = *self_ptr.add(i) <= *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_gt(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f32;
                            let rhs_ptr = rhs.0.as_ptr() as *const f32;
                            for i in 0..(N / size_of::<f32>()) {
                                result.0[i] = *self_ptr.add(i) > *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_ge(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f32;
                            let rhs_ptr = rhs.0.as_ptr() as *const f32;
                            for i in 0..(N / size_of::<f32>()) {
                                result.0[i] = *self_ptr.add(i) >= *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_f32_neq(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const f32;
                            let rhs_ptr = rhs.0.as_ptr() as *const f32;
                            for i in 0..(N / size_of::<f32>()) {
                                result.0[i] = *self_ptr.add(i) != *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i64_eq(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const i64;
                            let rhs_ptr = rhs.0.as_ptr() as *const i64;
                            for i in 0..(N / size_of::<i64>()) {
                                result.0[i] = *self_ptr.add(i) == *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i64_gt(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const i64;
                            let rhs_ptr = rhs.0.as_ptr() as *const i64;
                            for i in 0..(N / size_of::<i64>()) {
                                result.0[i] = *self_ptr.add(i) > *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i32_eq(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const i32;
                            let rhs_ptr = rhs.0.as_ptr() as *const i32;
                            for i in 0..(N / size_of::<i32>()) {
                                result.0[i] = *self_ptr.add(i) == *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i32_gt(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const i32;
                            let rhs_ptr = rhs.0.as_ptr() as *const i32;
                            for i in 0..(N / size_of::<i32>()) {
                                result.0[i] = *self_ptr.add(i) > *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i16_eq(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const i16;
                            let rhs_ptr = rhs.0.as_ptr() as *const i16;
                            for i in 0..(N / size_of::<i16>()) {
                                result.0[i] = *self_ptr.add(i) == *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i16_gt(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const i16;
                            let rhs_ptr = rhs.0.as_ptr() as *const i16;
                            for i in 0..(N / size_of::<i16>()) {
                                result.0[i] = *self_ptr.add(i) > *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i8_eq(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const i8;
                            let rhs_ptr = rhs.0.as_ptr() as *const i8;
                            for i in 0..(N / size_of::<i8>()) {
                                result.0[i] = *self_ptr.add(i) == *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn cmp_i8_gt(self, rhs: Self) -> Self::MaskType {
                        unsafe {
                            let mut result = ScalarMask::<N>([false; N]);
                            let self_ptr = self.0.as_ptr() as *const i8;
                            let rhs_ptr = rhs.0.as_ptr() as *const i8;
                            for i in 0..(N / size_of::<i8>()) {
                                result.0[i] = *self_ptr.add(i) > *rhs_ptr.add(i);
                            }
                            result
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_f64(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let rhs_ptr: *const f64 = rhs.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).max(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_f64(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let rhs_ptr: *const f64 = rhs.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).min(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_f32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let rhs_ptr: *const f32 = rhs.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).max(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_f32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let rhs_ptr: *const f32 = rhs.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).min(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i32 = self.0.as_ptr() as *const i32;
                            let rhs_ptr: *const i32 = rhs.0.as_ptr() as *const i32;
                            let new_ptr: *mut i32 = new.0.as_mut_ptr() as *mut i32;
                            for i in 0..(N / size_of::<i32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).max(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i32 = self.0.as_ptr() as *const i32;
                            let rhs_ptr: *const i32 = rhs.0.as_ptr() as *const i32;
                            let new_ptr: *mut i32 = new.0.as_mut_ptr() as *mut i32;
                            for i in 0..(N / size_of::<i32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).min(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i16(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i16 = self.0.as_ptr() as *const i16;
                            let rhs_ptr: *const i16 = rhs.0.as_ptr() as *const i16;
                            let new_ptr: *mut i16 = new.0.as_mut_ptr() as *mut i16;
                            for i in 0..(N / size_of::<i16>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).max(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i16(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i16 = self.0.as_ptr() as *const i16;
                            let rhs_ptr: *const i16 = rhs.0.as_ptr() as *const i16;
                            let new_ptr: *mut i16 = new.0.as_mut_ptr() as *mut i16;
                            for i in 0..(N / size_of::<i16>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).min(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_i8(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i8 = self.0.as_ptr() as *const i8;
                            let rhs_ptr: *const i8 = rhs.0.as_ptr() as *const i8;
                            let new_ptr: *mut i8 = new.0.as_mut_ptr() as *mut i8;
                            for i in 0..(N / size_of::<i8>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).max(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_i8(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const i8 = self.0.as_ptr() as *const i8;
                            let rhs_ptr: *const i8 = rhs.0.as_ptr() as *const i8;
                            let new_ptr: *mut i8 = new.0.as_mut_ptr() as *mut i8;
                            for i in 0..(N / size_of::<i8>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).min(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u32 = self.0.as_ptr() as *const u32;
                            let rhs_ptr: *const u32 = rhs.0.as_ptr() as *const u32;
                            let new_ptr: *mut u32 = new.0.as_mut_ptr() as *mut u32;
                            for i in 0..(N / size_of::<u32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).max(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u32(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u32 = self.0.as_ptr() as *const u32;
                            let rhs_ptr: *const u32 = rhs.0.as_ptr() as *const u32;
                            let new_ptr: *mut u32 = new.0.as_mut_ptr() as *mut u32;
                            for i in 0..(N / size_of::<u32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).min(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u16(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u16 = self.0.as_ptr() as *const u16;
                            let rhs_ptr: *const u16 = rhs.0.as_ptr() as *const u16;
                            let new_ptr: *mut u16 = new.0.as_mut_ptr() as *mut u16;
                            for i in 0..(N / size_of::<u16>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).max(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u16(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u16 = self.0.as_ptr() as *const u16;
                            let rhs_ptr: *const u16 = rhs.0.as_ptr() as *const u16;
                            let new_ptr: *mut u16 = new.0.as_mut_ptr() as *mut u16;
                            for i in 0..(N / size_of::<u16>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).min(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn max_u8(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u8 = self.0.as_ptr() as *const u8;
                            let rhs_ptr: *const u8 = rhs.0.as_ptr() as *const u8;
                            let new_ptr: *mut u8 = new.0.as_mut_ptr() as *mut u8;
                            for i in 0..(N / size_of::<u8>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).max(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn min_u8(self, rhs: Self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const u8 = self.0.as_ptr() as *const u8;
                            let rhs_ptr: *const u8 = rhs.0.as_ptr() as *const u8;
                            let new_ptr: *mut u8 = new.0.as_mut_ptr() as *mut u8;
                            for i in 0..(N / size_of::<u8>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).min(*rhs_ptr.add(i));
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdSplatImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn splat_64<T>(val: T) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u64;
                            for i in 0..(N / size_of::<u64>()) {
                                *new_ptr.add(i) = transmute_copy(&val);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_32<T>(val: T) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u32;
                            for i in 0..(N / size_of::<u32>()) {
                                *new_ptr.add(i) = transmute_copy(&val);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_16<T>(val: T) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u16;
                            for i in 0..(N / size_of::<u16>()) {
                                *new_ptr.add(i) = transmute_copy(&val);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn splat_8<T>(val: T) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u8;
                            for i in 0..(N / size_of::<u8>()) {
                                *new_ptr.add(i) = transmute_copy(&val);
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdGatherImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn gather_32_from_32<T, const B :
                        i32>(self, ptr: *const T) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u32;
                            let indices_ptr = self.0.as_ptr() as *const u32;
                            let data_ptr = ptr as *const u32;
                            for i in 0..(N / 4) {
                                let index = *indices_ptr.add(i) as usize;
                                *new_ptr.add(i) = *data_ptr.add(index);
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn gather_64_from_64<T, const B :
                        i32>(self, ptr: *const T) -> Self {
                        unsafe {
                            let mut new = Self([0; N]);
                            let new_ptr = new.0.as_mut_ptr() as *mut u64;
                            let indices_ptr = self.0.as_ptr() as *const u64;
                            let data_ptr = ptr as *const u64;
                            for i in 0..(N / 8) {
                                let index = *indices_ptr.add(i) as usize;
                                *new_ptr.add(i) = *data_ptr.add(index);
                            }
                            new
                        }
                    }
                }
                impl<const N : usize> SimdSqrtImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn sqrt_f64(self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f64 = self.0.as_ptr() as *const f64;
                            let new_ptr: *mut f64 = new.0.as_mut_ptr() as *mut f64;
                            for i in 0..(N / size_of::<f64>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).sqrt();
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn sqrt_f32(self) -> Self {
                        unsafe {
                            let mut new = ScalarReg::<N>([0; N]);
                            let self_ptr: *const f32 = self.0.as_ptr() as *const f32;
                            let new_ptr: *mut f32 = new.0.as_mut_ptr() as *mut f32;
                            for i in 0..(N / size_of::<f32>()) {
                                *new_ptr.add(i) = (*self_ptr.add(i)).sqrt();
                            }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn rsqrt_f32(self) -> Self {
                        unsafe { Self::splat_32(1.0f32).f32_div(self.sqrt_f32()) }
                    }
                }
                impl<const N : usize> SimdAllBitsImpl for ScalarMask<N> {
                    #[inline(always)]
                    unsafe fn all_zero(self) -> bool {
                        self.0.iter().any(|&x| !x)
                    }
                }
                impl<const N : usize> SimdBitwiseImpl for ScalarMask<N> {
                    #[inline(always)]
                    unsafe fn and(self, rhs: Self) -> Self {
                        Self(std::array::from_fn(|i| self.0[i] & rhs.0[i]))
                    }
                    #[inline(always)]
                    unsafe fn or(self, rhs: Self) -> Self {
                        Self(std::array::from_fn(|i| self.0[i] | rhs.0[i]))
                    }
                    #[inline(always)]
                    unsafe fn xor(self, rhs: Self) -> Self {
                        Self(std::array::from_fn(|i| self.0[i] ^ rhs.0[i]))
                    }
                    #[inline(always)]
                    unsafe fn not(self) -> Self {
                        Self(std::array::from_fn(|i| !self.0[i]))
                    }
                    #[inline(always)]
                    unsafe fn and_not(self, rhs: Self) -> Self {
                        unsafe { self.and(rhs.not()) }
                    }
                }
                impl<const N : usize> SimdNegateImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn negate_f64(self) -> Self {
                        unsafe { Self::splat_64(-0.0f64).xor(self) }
                    }
                    #[inline(always)]
                    unsafe fn negate_f32(self) -> Self {
                        unsafe { Self::splat_32(-0.0f64).xor(self) }
                    }
                }
                impl<const N : usize> SimdBlockShiftImpl for ScalarReg<N> {
                    #[inline(always)]
                    unsafe fn block_left_byte_shift<const M : i32>(self)
                        -> Self {
                        let mut new = unsafe { Self::splat_8(0) };
                        for block_start in (0..N).step_by(16) {
                            let block_end = block_start + 16;
                            for i in (block_start + M as usize)..block_end {
                                new.0[i] = self.0[i - M as usize];
                            }
                        }
                        new
                    }
                    #[inline(always)]
                    unsafe fn block_right_byte_shift<const M : i32>(self)
                        -> Self {
                        let mut new = unsafe { Self::splat_8(0) };
                        for block_start in (0..N).step_by(16) {
                            let block_end = block_start + 16;
                            for i in block_start..(block_end - M as usize) {
                                new.0[i] = self.0[i + M as usize];
                            }
                        }
                        new
                    }
                }
                impl<const N : usize> SimdMaskBitConversion for ScalarMask<N>
                    {
                    #[inline(always)]
                    unsafe fn to_bits_64(self) -> u64 {
                        let mut bits = 0u64;
                        for i in 0..(N >> 3) { bits ^= (self.0[i] as u64) << i }
                        bits
                    }
                    #[inline(always)]
                    unsafe fn to_bits_32(self) -> u64 {
                        let mut bits = 0u64;
                        for i in 0..(N >> 2) { bits ^= (self.0[i] as u64) << i }
                        bits
                    }
                    #[inline(always)]
                    unsafe fn to_bits_8(self) -> u64 {
                        let mut bits = 0u64;
                        for i in 0..N { bits ^= (self.0[i] as u64) << i }
                        bits
                    }
                    #[inline(always)]
                    unsafe fn from_bits_64(bitmask: u64) -> Self {
                        let mut new_mask = Self([false; N]);
                        for i in 0..N { new_mask.0[i] = ((bitmask >> i) & 1) == 1; }
                        new_mask
                    }
                    unsafe fn from_bits_32(bitmask: u64) -> Self {
                        let mut new_mask = Self([false; N]);
                        for i in 0..N { new_mask.0[i] = ((bitmask >> i) & 1) == 1; }
                        new_mask
                    }
                    unsafe fn from_bits_16(bitmask: u64) -> Self {
                        let mut new_mask = Self([false; N]);
                        for i in 0..N { new_mask.0[i] = ((bitmask >> i) & 1) == 1; }
                        new_mask
                    }
                    unsafe fn from_bits_8(bitmask: u64) -> Self {
                        let mut new_mask = Self([false; N]);
                        for i in 0..N { new_mask.0[i] = ((bitmask >> i) & 1) == 1; }
                        new_mask
                    }
                }
                impl<const M : usize> SimdLaneShiftImpl for ScalarReg<M> {
                    #[inline(always)]
                    unsafe fn left_lane_shift_32(self, n: u32) -> Self {
                        let mut new = unsafe { Self::zero() };
                        if n as usize * 4 >= M {
                            new
                        } else {
                            let bytes = (n * 4) as usize;
                            for i in 0..(M - bytes) { new.0[i] = self.0[i + bytes]; }
                            new
                        }
                    }
                    #[inline(always)]
                    unsafe fn right_lane_shift_32(self, n: u32) -> Self {
                        let mut new = unsafe { Self::zero() };
                        if n as usize * 4 >= M {
                            new
                        } else {
                            let bytes = (n * 4) as usize;
                            for i in bytes..M { new.0[i] = self.0[i - bytes]; }
                            new
                        }
                    }
                }
            }
        }
        #[macro_use]
        pub mod macros {
            #[macro_export]
            macro_rules! execute_intrinsic {
                ($intrinsic:ident, $($arg:expr),*) =>
                { unsafe { $intrinsic($(transmute_copy(&$arg)),*) } };
            }
            #[macro_export]
            macro_rules! execute_const_intrinsic {
                ($intrinsic:ident, $const:expr, $($arg:expr),*) =>
                { unsafe { $intrinsic::<{$const}>($(transmute($arg)),*) } };
            }
            #[macro_export]
            macro_rules! self_from_op {
                ($intrinsic:ident, $($arg:expr),*) =>
                {
                    unsafe
                    {
                        Self(transmute_copy(&$intrinsic($(transmute_copy(&$arg)),*)))
                    }
                }
            }
            #[macro_export]
            macro_rules! self_from_const_op {
                ($intrinsic:ident, $const:expr, $($arg:expr),*) =>
                {
                    unsafe
                    {
                        Self(transmute($intrinsic::<{$const}>($(transmute($arg)),*)))
                    }
                }
            }
            pub use crate::execute_intrinsic;
            pub use crate::execute_const_intrinsic;
            pub use crate::self_from_op;
            pub use crate::self_from_const_op;
        }
        pub mod arch {
            use crate::simd::SimdElement;
            use crate::simd::architectures::interface::Arch;
            use crate::simd::architectures::intrinsics::avx2::Avx2Reg;
            use crate::simd::architectures::intrinsics::avx512::{
                Avx512Reg, Avx512Mask,
            };
            use crate::simd::architectures::intrinsics::sse::SseReg;
            use crate::simd::architectures::intrinsics::scalar::{
                ScalarReg, ScalarMask,
            };
            use crate::simd::register::Simd;
            use std::fmt::Debug;
            pub struct Sse;
            #[automatically_derived]
            impl ::core::marker::Copy for Sse { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Sse { }
            #[automatically_derived]
            impl ::core::clone::Clone for Sse {
                #[inline]
                fn clone(&self) -> Sse { *self }
            }
            impl Arch for Sse {
                const SIMD_WIDTH: usize = 16;
                const NUM_SIMD_REG: usize = 16;
                type Block2<T: SimdElement> = [Simd<T, Self>; 4];
                type Block4<T: SimdElement> = [Simd<T, Self>; 2];
                type Vec = SseReg;
                type Mask = SseReg;
                type ScalarArch = Scalar128;
                type Array64<T: Debug + Copy> = [T; 2];
                type Array32<T: Debug + Copy> = [T; 4];
                type Array16<T: Debug + Copy> = [T; 8];
                type Array8<T: Debug + Copy> = [T; 16];
            }
            pub struct Avx2;
            #[automatically_derived]
            impl ::core::marker::Copy for Avx2 { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Avx2 { }
            #[automatically_derived]
            impl ::core::clone::Clone for Avx2 {
                #[inline]
                fn clone(&self) -> Avx2 { *self }
            }
            impl Arch for Avx2 {
                const SIMD_WIDTH: usize = 32;
                const NUM_SIMD_REG: usize = 16;
                type Block2<T: SimdElement> = [Simd<T, Self>; 4];
                type Block4<T: SimdElement> = [Simd<T, Self>; 2];
                type Vec = Avx2Reg;
                type Mask = Avx2Reg;
                type ScalarArch = Scalar256;
                type Array64<T: Debug + Copy> = [T; 4];
                type Array32<T: Debug + Copy> = [T; 8];
                type Array16<T: Debug + Copy> = [T; 16];
                type Array8<T: Debug + Copy> = [T; 32];
            }
            pub struct Avx512;
            #[automatically_derived]
            impl ::core::marker::Copy for Avx512 { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Avx512 { }
            #[automatically_derived]
            impl ::core::clone::Clone for Avx512 {
                #[inline]
                fn clone(&self) -> Avx512 { *self }
            }
            impl Arch for Avx512 {
                const SIMD_WIDTH: usize = 64;
                const NUM_SIMD_REG: usize = 32;
                type Block2<T: SimdElement> = [Simd<T, Self>; 8];
                type Block4<T: SimdElement> = [Simd<T, Self>; 4];
                type Vec = Avx512Reg;
                type Mask = Avx512Mask;
                type ScalarArch = Scalar512;
                type Array64<T: Debug + Copy> = [T; 8];
                type Array32<T: Debug + Copy> = [T; 16];
                type Array16<T: Debug + Copy> = [T; 32];
                type Array8<T: Debug + Copy> = [T; 64];
            }
            pub struct Scalar128;
            #[automatically_derived]
            impl ::core::marker::Copy for Scalar128 { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Scalar128 { }
            #[automatically_derived]
            impl ::core::clone::Clone for Scalar128 {
                #[inline]
                fn clone(&self) -> Scalar128 { *self }
            }
            impl Arch for Scalar128 {
                const SIMD_WIDTH: usize = 16;
                const NUM_SIMD_REG: usize = 16;
                type Block2<T: SimdElement> = [Simd<T, Self>; 4];
                type Block4<T: SimdElement> = [Simd<T, Self>; 2];
                type Vec = ScalarReg<16>;
                type Mask = ScalarMask<16>;
                type ScalarArch = Self;
                type Array64<T: Debug + Copy> = [T; 2];
                type Array32<T: Debug + Copy> = [T; 4];
                type Array16<T: Debug + Copy> = [T; 8];
                type Array8<T: Debug + Copy> = [T; 16];
            }
            pub struct Scalar256;
            #[automatically_derived]
            impl ::core::marker::Copy for Scalar256 { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Scalar256 { }
            #[automatically_derived]
            impl ::core::clone::Clone for Scalar256 {
                #[inline]
                fn clone(&self) -> Scalar256 { *self }
            }
            impl Arch for Scalar256 {
                const SIMD_WIDTH: usize = 32;
                const NUM_SIMD_REG: usize = 16;
                type Block2<T: SimdElement> = [Simd<T, Self>; 4];
                type Block4<T: SimdElement> = [Simd<T, Self>; 2];
                type Vec = ScalarReg<32>;
                type Mask = ScalarMask<32>;
                type ScalarArch = Self;
                type Array64<T: Debug + Copy> = [T; 4];
                type Array32<T: Debug + Copy> = [T; 8];
                type Array16<T: Debug + Copy> = [T; 16];
                type Array8<T: Debug + Copy> = [T; 32];
            }
            pub struct Scalar512;
            #[automatically_derived]
            impl ::core::marker::Copy for Scalar512 { }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Scalar512 { }
            #[automatically_derived]
            impl ::core::clone::Clone for Scalar512 {
                #[inline]
                fn clone(&self) -> Scalar512 { *self }
            }
            impl Arch for Scalar512 {
                const SIMD_WIDTH: usize = 64;
                const NUM_SIMD_REG: usize = 16;
                type Block2<T: SimdElement> = [Simd<T, Self>; 4];
                type Block4<T: SimdElement> = [Simd<T, Self>; 2];
                type Vec = ScalarReg<64>;
                type Mask = ScalarMask<64>;
                type ScalarArch = Self;
                type Array64<T: Debug + Copy> = [T; 8];
                type Array32<T: Debug + Copy> = [T; 16];
                type Array16<T: Debug + Copy> = [T; 32];
                type Array8<T: Debug + Copy> = [T; 64];
            }
        }
        pub mod interface {
            #![allow(clippy::missing_safety_doc)]
            use std::{fmt::Debug, ops::{Index, IndexMut}};
            use crate::simd::{
                SimdElement, array_trait::Array, register::Simd,
            };
            pub trait Arch: Clone + Copy {
                const SIMD_WIDTH: usize;
                const NUM_SIMD_REG: usize;
                type
                    Block2<T: SimdElement>: Index<usize, Output =
                    Simd<T, Self>> + IndexMut<usize> + Default;
                type
                    Block4<T: SimdElement>: Index<usize, Output =
                    Simd<T, Self>> + IndexMut<usize> + Default;
                type Vec: SimdArch + Copy + Clone +
                    SimdLoadImpl<MaskType = Self::Mask> +
                    SimdStoreImpl<MaskType = Self::Mask> +
                    SimdPartialOrdImpl<MaskType = Self::Mask>;
                type Mask: MaskArch + Copy + Clone +
                    SimdVariableBlendImpl<VecType = Self::Vec>;
                type ScalarArch: Arch;
                type Array64<T: Debug + Copy>: Debug + Copy + Array<T>;
                type Array32<T: Debug + Copy>: Debug + Copy + Array<T>;
                type Array16<T: Debug + Copy>: Debug + Copy + Array<T>;
                type Array8<T: Debug + Copy>: Debug + Copy + Array<T>;
            }
            pub trait SimdArch: Copy + Clone + SimdAddImpl + SimdSubImpl +
                SimdMulImpl + SimdDivImpl + SimdBitwiseImpl + SimdShiftImpl +
                SimdLoadImpl + SimdStoreImpl + SimdZeroImpl +
                SimdFloatCastsImpl + SimdIntCastsImpl + SimdPermuteImpl +
                SimdMulAddImpl + SimdRoundImpl + SimdPartialOrdImpl +
                SimdSplatImpl + SimdGatherImpl + SimdSqrtImpl +
                SimdNegateImpl + SimdBlockShiftImpl + SimdImmediateBlendImpl +
                SimdLaneShiftImpl {}
            pub trait MaskArch: Copy + Clone + SimdBitwiseImpl +
                SimdAllBitsImpl + SimdVariableBlendImpl +
                SimdMaskBitConversion {}
            pub trait SimdAddImpl {
                unsafe fn f64_add(self, rhs: Self)
                -> Self;
                unsafe fn f32_add(self, rhs: Self)
                -> Self;
                unsafe fn i64_add(self, rhs: Self)
                -> Self;
                unsafe fn i32_add(self, rhs: Self)
                -> Self;
                unsafe fn i16_add(self, rhs: Self)
                -> Self;
                unsafe fn i8_add(self, rhs: Self)
                -> Self;
            }
            pub trait SimdSubImpl {
                unsafe fn f64_sub(self, rhs: Self)
                -> Self;
                unsafe fn f32_sub(self, rhs: Self)
                -> Self;
                unsafe fn i64_sub(self, rhs: Self)
                -> Self;
                unsafe fn i32_sub(self, rhs: Self)
                -> Self;
                unsafe fn i16_sub(self, rhs: Self)
                -> Self;
                unsafe fn i8_sub(self, rhs: Self)
                -> Self;
            }
            pub trait SimdMulImpl {
                unsafe fn f64_mul(self, rhs: Self)
                -> Self;
                unsafe fn f32_mul(self, rhs: Self)
                -> Self;
                unsafe fn i32_mul(self, rhs: Self)
                -> Self;
                unsafe fn i16_mul(self, rhs: Self)
                -> Self;
            }
            pub trait SimdDivImpl {
                unsafe fn f64_div(self, rhs: Self)
                -> Self;
                unsafe fn f32_div(self, rhs: Self)
                -> Self;
            }
            pub trait SimdBitwiseImpl {
                unsafe fn and(self, rhs: Self)
                -> Self;
                unsafe fn or(self, rhs: Self)
                -> Self;
                unsafe fn xor(self, rhs: Self)
                -> Self;
                unsafe fn not(self)
                -> Self;
                unsafe fn and_not(self, rhs: Self)
                -> Self;
            }
            pub trait SimdShiftImpl {
                unsafe fn sllv_64(self, shift: Self)
                -> Self;
                unsafe fn srlv_64(self, shift: Self)
                -> Self;
                unsafe fn srav_64(self, shift: Self)
                -> Self;
                unsafe fn sllv_32(self, shift: Self)
                -> Self;
                unsafe fn srlv_32(self, shift: Self)
                -> Self;
                unsafe fn srav_32(self, shift: Self)
                -> Self;
                unsafe fn sllv_16(self, shift: Self)
                -> Self;
                unsafe fn srlv_16(self, shift: Self)
                -> Self;
                unsafe fn srav_16(self, shift: Self)
                -> Self;
            }
            pub trait SimdLoadImpl {
                type MaskType;
                unsafe fn load_aligned<T>(ptr: *const T)
                -> Self;
                unsafe fn load_unaligned<T>(ptr: *const T)
                -> Self;
                unsafe fn masked_load_64<T>(ptr: *const T,
                mask: Self::MaskType)
                -> Self;
                unsafe fn masked_load_32<T>(ptr: *const T,
                mask: Self::MaskType)
                -> Self;
            }
            pub trait SimdStoreImpl {
                type MaskType;
                unsafe fn store_aligned<T>(self, ptr: *mut T);
                unsafe fn store_unaligned<T>(self, ptr: *mut T);
                unsafe fn masked_store_64<T>(self, ptr: *mut T,
                mask: Self::MaskType);
                unsafe fn masked_store_32<T>(self, ptr: *mut T,
                mask: Self::MaskType);
            }
            pub trait SimdZeroImpl {
                unsafe fn zero()
                -> Self;
            }
            pub trait SimdFloatCastsImpl {
                unsafe fn float_to_int_trunc(self)
                -> Self;
                unsafe fn float_to_int_round(self)
                -> Self;
            }
            pub trait SimdIntCastsImpl {
                unsafe fn int_to_float(self)
                -> Self;
            }
            pub trait SimdPermuteImpl {
                unsafe fn permute_32(self, rhs: Self)
                -> Self;
                unsafe fn permute_8(self, rhs: Self)
                -> Self;
            }
            pub trait SimdVariableBlendImpl {
                type VecType;
                unsafe fn vblend_64(self, true_values: Self::VecType,
                false_values: Self::VecType)
                -> Self::VecType;
                unsafe fn vblend_32(self, true_values: Self::VecType,
                false_values: Self::VecType)
                -> Self::VecType;
                unsafe fn vblend_8(self, true_values: Self::VecType,
                false_values: Self::VecType)
                -> Self::VecType;
            }
            pub trait SimdImmediateBlendImpl {
                unsafe fn blend_64<const N : i32>(self, false_values: Self)
                -> Self;
                unsafe fn blend_32<const N : i32>(self, false_values: Self)
                -> Self;
            }
            pub trait SimdMulAddImpl {
                unsafe fn mul_add_f64(self, mult: Self, add: Self)
                -> Self;
                unsafe fn mul_sub_f64(self, mult: Self, sub: Self)
                -> Self;
                unsafe fn negated_mul_add_f64(self, mult: Self, add: Self)
                -> Self;
                unsafe fn negated_mul_sub_f64(self, mult: Self, sub: Self)
                -> Self;
                unsafe fn mul_add_f32(self, mult: Self, add: Self)
                -> Self;
                unsafe fn mul_sub_f32(self, mult: Self, sub: Self)
                -> Self;
                unsafe fn negated_mul_add_f32(self, mult: Self, add: Self)
                -> Self;
                unsafe fn negated_mul_sub_f32(self, mult: Self, sub: Self)
                -> Self;
            }
            pub trait SimdRoundImpl {
                unsafe fn round_f64(self)
                -> Self;
                unsafe fn round_f32(self)
                -> Self;
                unsafe fn floor_f64(self)
                -> Self;
                unsafe fn floor_f32(self)
                -> Self;
                unsafe fn ceil_f64(self)
                -> Self;
                unsafe fn ceil_f32(self)
                -> Self;
            }
            pub trait SimdPartialOrdImpl {
                type MaskType;
                unsafe fn cmp_f64_eq(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f64_lt(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f64_le(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f64_gt(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f64_ge(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f64_neq(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f32_eq(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f32_lt(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f32_le(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f32_gt(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f32_ge(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_f32_neq(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_i64_eq(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_i64_gt(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_i32_eq(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_i32_gt(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_i16_eq(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_i16_gt(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_i8_eq(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn cmp_i8_gt(self, rhs: Self)
                -> Self::MaskType;
                unsafe fn max_f64(self, rhs: Self)
                -> Self;
                unsafe fn min_f64(self, rhs: Self)
                -> Self;
                unsafe fn max_f32(self, rhs: Self)
                -> Self;
                unsafe fn min_f32(self, rhs: Self)
                -> Self;
                unsafe fn max_i32(self, rhs: Self)
                -> Self;
                unsafe fn min_i32(self, rhs: Self)
                -> Self;
                unsafe fn max_i16(self, rhs: Self)
                -> Self;
                unsafe fn min_i16(self, rhs: Self)
                -> Self;
                unsafe fn max_i8(self, rhs: Self)
                -> Self;
                unsafe fn min_i8(self, rhs: Self)
                -> Self;
                unsafe fn max_u32(self, rhs: Self)
                -> Self;
                unsafe fn min_u32(self, rhs: Self)
                -> Self;
                unsafe fn max_u16(self, rhs: Self)
                -> Self;
                unsafe fn min_u16(self, rhs: Self)
                -> Self;
                unsafe fn max_u8(self, rhs: Self)
                -> Self;
                unsafe fn min_u8(self, rhs: Self)
                -> Self;
            }
            pub trait SimdSplatImpl {
                unsafe fn splat_64<T>(val: T)
                -> Self;
                unsafe fn splat_32<T>(val: T)
                -> Self;
                unsafe fn splat_16<T>(val: T)
                -> Self;
                unsafe fn splat_8<T>(val: T)
                -> Self;
            }
            pub trait SimdGatherImpl {
                unsafe fn gather_32_from_32<T, const B :
                i32>(self, ptr: *const T)
                -> Self;
                unsafe fn gather_64_from_64<T, const B :
                i32>(self, ptr: *const T)
                -> Self;
            }
            pub trait SimdSqrtImpl {
                unsafe fn sqrt_f64(self)
                -> Self;
                unsafe fn sqrt_f32(self)
                -> Self;
                unsafe fn rsqrt_f32(self)
                -> Self;
            }
            pub trait SimdAllBitsImpl {
                unsafe fn all_zero(self)
                -> bool;
            }
            pub trait SimdNegateImpl {
                unsafe fn negate_f64(self)
                -> Self;
                unsafe fn negate_f32(self)
                -> Self;
            }
            pub trait SimdBlockShiftImpl {
                unsafe fn block_right_byte_shift<const N : i32>(self)
                -> Self;
                unsafe fn block_left_byte_shift<const N : i32>(self)
                -> Self;
            }
            pub trait SimdMaskBitConversion {
                unsafe fn to_bits_64(self)
                -> u64;
                unsafe fn to_bits_32(self)
                -> u64;
                unsafe fn to_bits_8(self)
                -> u64;
                unsafe fn from_bits_64(bitmask: u64)
                -> Self;
                unsafe fn from_bits_32(bitmask: u64)
                -> Self;
                unsafe fn from_bits_16(bitmask: u64)
                -> Self;
                unsafe fn from_bits_8(bitmask: u64)
                -> Self;
            }
            pub trait SimdLaneShiftImpl {
                unsafe fn right_lane_shift_32(self, n: u32)
                -> Self;
                unsafe fn left_lane_shift_32(self, n: u32)
                -> Self;
            }
        }
    }
    pub mod array_trait {
        use std::borrow::{Borrow, BorrowMut};
        use std::fmt::Debug;
        use std::ops::{Index, IndexMut};
        pub trait Array<T>: Clone + Debug + Index<usize, Output = T> +
            IndexMut<usize, Output = T> + AsRef<[T]> + AsMut<[T]> +
            Borrow<[T]> + BorrowMut<[T]> + IntoIterator<Item = T> + Sized {
            const LEN: usize;
            fn from_fn(f: impl FnMut(usize) -> T)
            -> Self;
            fn zeroed() -> Self where T: Default {
                Self::from_fn(|_| T::default())
            }
            fn as_slice(&self)
            -> &[T];
            fn as_mut_slice(&mut self)
            -> &mut [T];
            fn get(&self, i: usize) -> Option<&T> { self.as_slice().get(i) }
            fn get_mut(&mut self, i: usize) -> Option<&mut T> {
                self.as_mut_slice().get_mut(i)
            }
            fn first(&self) -> Option<&T> { self.as_slice().first() }
            fn first_mut(&mut self) -> Option<&mut T> {
                self.as_mut_slice().first_mut()
            }
            fn last(&self) -> Option<&T> { self.as_slice().last() }
            fn last_mut(&mut self) -> Option<&mut T> {
                self.as_mut_slice().last_mut()
            }
            fn len(&self) -> usize { Self::LEN }
            fn is_empty(&self) -> bool { Self::LEN == 0 }
            fn iter(&self) -> std::slice::Iter<'_, T> {
                self.as_slice().iter()
            }
            fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
                self.as_mut_slice().iter_mut()
            }
            fn windows(&self, size: usize) -> std::slice::Windows<'_, T> {
                self.as_slice().windows(size)
            }
            fn chunks(&self, size: usize) -> std::slice::Chunks<'_, T> {
                self.as_slice().chunks(size)
            }
            fn chunks_exact(&self, size: usize)
                -> std::slice::ChunksExact<'_, T> {
                self.as_slice().chunks_exact(size)
            }
            fn rchunks(&self, size: usize) -> std::slice::RChunks<'_, T> {
                self.as_slice().rchunks(size)
            }
            fn rchunks_exact(&self, size: usize)
                -> std::slice::RChunksExact<'_, T> {
                self.as_slice().rchunks_exact(size)
            }
            fn contains(&self, x: &T) -> bool where T: PartialEq {
                self.as_slice().contains(x)
            }
            fn starts_with(&self, needle: &[T]) -> bool where T: PartialEq {
                self.as_slice().starts_with(needle)
            }
            fn ends_with(&self, needle: &[T]) -> bool where T: PartialEq {
                self.as_slice().ends_with(needle)
            }
            fn binary_search(&self, x: &T) -> Result<usize, usize> where
                T: Ord {
                self.as_slice().binary_search(x)
            }
            fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
                where F: FnMut(&'a T) -> std::cmp::Ordering, T: 'a {
                self.as_slice().binary_search_by(f)
            }
            fn binary_search_by_key<'a, B: Ord, F: FnMut(&'a T)
                -> B>(&'a self, b: &B, f: F) -> Result<usize, usize> where
                T: 'a {
                self.as_slice().binary_search_by_key(b, f)
            }
            fn position(&self, f: impl FnMut(&T) -> bool) -> Option<usize> {
                self.as_slice().iter().position(f)
            }
            fn fill(&mut self, value: T) where T: Clone {
                self.as_mut_slice().fill(value)
            }
            fn fill_with(&mut self, f: impl FnMut() -> T) {
                self.as_mut_slice().fill_with(f)
            }
            fn swap(&mut self, a: usize, b: usize) {
                self.as_mut_slice().swap(a, b)
            }
            fn reverse(&mut self) { self.as_mut_slice().reverse() }
            fn rotate_left(&mut self, mid: usize) {
                self.as_mut_slice().rotate_left(mid)
            }
            fn rotate_right(&mut self, mid: usize) {
                self.as_mut_slice().rotate_right(mid)
            }
            fn copy_from_slice(&mut self, src: &[T]) where T: Copy {
                self.as_mut_slice().copy_from_slice(src)
            }
            fn clone_from_slice(&mut self, src: &[T]) where T: Clone {
                self.as_mut_slice().clone_from_slice(src)
            }
            fn swap_with_slice(&mut self, other: &mut [T]) {
                self.as_mut_slice().swap_with_slice(other)
            }
            fn sort(&mut self) where T: Ord { self.as_mut_slice().sort() }
            fn sort_by(&mut self,
                f: impl FnMut(&T, &T) -> std::cmp::Ordering) {
                self.as_mut_slice().sort_by(f)
            }
            fn sort_by_key<K: Ord>(&mut self, f: impl FnMut(&T) -> K) {
                self.as_mut_slice().sort_by_key(f)
            }
            fn sort_unstable(&mut self) where T: Ord {
                self.as_mut_slice().sort_unstable()
            }
            fn sort_unstable_by(&mut self,
                f: impl FnMut(&T, &T) -> std::cmp::Ordering) {
                self.as_mut_slice().sort_unstable_by(f)
            }
            fn sort_unstable_by_key<K: Ord>(&mut self,
                f: impl FnMut(&T) -> K) {
                self.as_mut_slice().sort_unstable_by_key(f)
            }
            fn is_sorted(&self) -> bool where T: PartialOrd {
                self.as_slice().is_sorted()
            }
            fn is_sorted_by(&self, f: impl FnMut(&T, &T) -> bool) -> bool {
                self.as_slice().is_sorted_by(f)
            }
            fn is_sorted_by_key<K: PartialOrd>(&self, f: impl FnMut(&T) -> K)
                -> bool {
                self.as_slice().is_sorted_by_key(f)
            }
            fn as_ptr(&self) -> *const T { self.as_slice().as_ptr() }
            fn as_mut_ptr(&mut self) -> *mut T {
                self.as_mut_slice().as_mut_ptr()
            }
        }
        impl<const N : usize, T> Array<T> for [T; N] where T: Clone + Debug {
            const LEN: usize = N;
            #[inline(always)]
            fn from_fn(f: impl FnMut(usize) -> T) -> Self {
                std::array::from_fn(f)
            }
            #[inline(always)]
            fn as_slice(&self) -> &[T] { self.as_slice() }
            #[inline(always)]
            fn as_mut_slice(&mut self) -> &mut [T] { self.as_mut_slice() }
        }
    }
    pub mod mask {
        use std::marker::PhantomData;
        use crate::simd::architectures::interface::Arch;
        pub mod element {
            use std::marker::PhantomData;
            use std::ops::*;
            use crate::simd::architectures::interface::{
                SimdAllBitsImpl, Arch, *,
            };
            use crate::simd::mask::Mask;
            use crate::simd::register::Simd;
            use crate::simd::traits::*;
            impl<T: SimdElement, F: Arch> Mask<T, F> {
                #[inline(always)]
                pub(crate) fn new(data: F::Mask) -> Self {
                    Self { data, _marker: PhantomData }
                }
                #[inline(always)]
                pub fn raw_cast<S: SimdElement>(self) -> Mask<S, F> {
                    Mask::new(self.data)
                }
                #[inline(always)]
                pub fn all_false(self) -> bool {
                    unsafe { self.data.all_zero() }
                }
                #[inline(always)]
                pub fn first_n_true(n: u32) -> Mask<T, F> {
                    let iota = Simd::iota(0u32);
                    let n_vec = Simd::splat(n);
                    n_vec.simd_gt(iota).raw_cast()
                }
                #[inline(always)]
                pub fn first_n_false(n: u32) -> Mask<T, F> {
                    let iota = Simd::iota(1u32);
                    let n_vec = Simd::splat(n);
                    iota.simd_gt(n_vec).raw_cast()
                }
            }
            impl<T: SimdElement, F: Arch> BitAnd for Mask<T, F> {
                type Output = Self;
                #[inline(always)]
                fn bitand(self, rhs: Self) -> Self {
                    unsafe { Self::new(self.data.and(rhs.data)) }
                }
            }
            impl<T: SimdElement, F: Arch> BitOr for Mask<T, F> {
                type Output = Self;
                #[inline(always)]
                fn bitor(self, rhs: Self) -> Self {
                    unsafe { Self::new(self.data.or(rhs.data)) }
                }
            }
            impl<T: SimdElement, F: Arch> BitXor for Mask<T, F> {
                type Output = Self;
                #[inline(always)]
                fn bitxor(self, rhs: Self) -> Self {
                    unsafe { Self::new(self.data.xor(rhs.data)) }
                }
            }
            impl<T: SimdElement, F: Arch> Not for Mask<T, F> {
                type Output = Self;
                #[inline(always)]
                fn not(self) -> Self { unsafe { Self::new(self.data.not()) } }
            }
            impl<T: SimdElement, F: Arch> Mask<T, F> {
                #[inline(always)]
                pub fn andnot(self, rhs: Self) -> Self {
                    unsafe { Self::new(self.data.and_not(rhs.data)) }
                }
                pub fn select(self, true_values: Simd<T, F>,
                    false_values: Simd<T, F>) -> Simd<T, F> {
                    unsafe {
                        match T::BIT_SIZE {
                            BitSize::Size64 => {
                                Simd::new(self.data.vblend_64(true_values.data,
                                        false_values.data))
                            }
                            BitSize::Size32 => {
                                Simd::new(self.data.vblend_32(true_values.data,
                                        false_values.data))
                            }
                            BitSize::Size8 => {
                                Simd::new(self.data.vblend_8(true_values.data,
                                        false_values.data))
                            }
                            _ => {
                                ::core::panicking::panic_fmt(format_args!("Select for 16 bit types not implemented yet!"));
                            }
                        }
                    }
                }
            }
            impl<T: SimdElement, F: Arch> Mask<T, F> {
                pub fn to_bits(self) -> u64 {
                    unsafe {
                        match T::BIT_SIZE {
                            BitSize::Size64 => self.data.to_bits_64(),
                            BitSize::Size32 => self.data.to_bits_32(),
                            BitSize::Size8 => self.data.to_bits_8(),
                            _ =>
                                ::core::panicking::panic("internal error: entered unreachable code"),
                        }
                    }
                }
                pub fn from_bits(bitmask: u64) -> Self {
                    unsafe {
                        match T::BIT_SIZE {
                            BitSize::Size64 =>
                                Self::new(F::Mask::from_bits_64(bitmask)),
                            BitSize::Size32 =>
                                Self::new(F::Mask::from_bits_32(bitmask)),
                            BitSize::Size16 =>
                                Self::new(F::Mask::from_bits_16(bitmask)),
                            BitSize::Size8 => Self::new(F::Mask::from_bits_8(bitmask)),
                        }
                    }
                }
            }
        }
        pub struct Mask<T, F: Arch> {
            pub(crate) data: F::Mask,
            pub(crate) _marker: PhantomData<T>,
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone, F: ::core::clone::Clone + Arch>
            ::core::clone::Clone for Mask<T, F> where
            F::Mask: ::core::clone::Clone {
            #[inline]
            fn clone(&self) -> Mask<T, F> {
                Mask {
                    data: ::core::clone::Clone::clone(&self.data),
                    _marker: ::core::clone::Clone::clone(&self._marker),
                }
            }
        }
        #[automatically_derived]
        impl<T: ::core::marker::Copy, F: ::core::marker::Copy + Arch>
            ::core::marker::Copy for Mask<T, F> where
            F::Mask: ::core::marker::Copy {
        }
    }
    pub mod register {
        use std::marker::PhantomData;
        use crate::simd::architectures::interface::*;
        use crate::simd::traits::*;
        pub mod element {
            use std::fmt;
            use std::marker::PhantomData;
            use std::ops::*;
            use num_traits::NumCast;
            use crate::simd::architectures::interface::*;
            use crate::simd::array_trait::Array;
            use crate::simd::mask::Mask;
            use crate::simd::register::Simd;
            use crate::simd::traits::*;
            impl<T: SimdElement, F: Arch> Simd<T, F> {
                #[inline(always)]
                pub(crate) fn new(data: F::Vec) -> Self {
                    Self { data, _marker: PhantomData }
                }
                #[inline(always)]
                pub fn zero() -> Self { unsafe { Self::new(F::Vec::zero()) } }
                #[inline(always)]
                pub fn from_aligned_slice(slice: &[T]) -> Self {
                    let ptr = slice.as_ptr();
                    if !(ptr.align_offset(Self::SIMD_WIDTH) == 0) {
                        ::core::panicking::panic("assertion failed: ptr.align_offset(Self::SIMD_WIDTH) == 0")
                    };
                    if !(slice.len() >= Self::LANES) {
                        ::core::panicking::panic("assertion failed: slice.len() >= Self::LANES")
                    };
                    unsafe { Self::new(F::Vec::load_aligned(ptr)) }
                }
                /// # Safety
                /// Does not check if the slice goes out of bounds.
                #[inline(always)]
                pub unsafe fn from_aligned_slice_unchecked(slice: &[T])
                    -> Self {
                    let ptr = slice.as_ptr();
                    if true {
                        if !(ptr.align_offset(Self::SIMD_WIDTH) == 0) {
                            ::core::panicking::panic("assertion failed: ptr.align_offset(Self::SIMD_WIDTH) == 0")
                        };
                    };
                    if true {
                        if !(slice.len() >= Self::LANES) {
                            ::core::panicking::panic("assertion failed: slice.len() >= Self::LANES")
                        };
                    };
                    unsafe { Self::new(F::Vec::load_aligned(ptr)) }
                }
                #[inline(always)]
                pub fn from_slice(slice: &[T]) -> Self {
                    if slice.len() >= Self::LANES {
                        unsafe { Self::new(F::Vec::load_unaligned(slice.as_ptr())) }
                    } else {
                        let mut array = Self::zero().to_array();
                        for (arr, val) in array.iter_mut().zip(slice.iter()) {
                            *arr = *val;
                        }
                        unsafe { Self::from_slice_unchecked(array.as_slice()) }
                    }
                }
                /// # Safety
                /// Requires allocated memory to be behind (left) of the slice.
                /// Bounds are not checked.
                /// Length of the slice must be less than or equal to the number of lanes.
                #[inline(always)]
                pub unsafe fn from_slice_partial(slice: &[T]) -> Self {
                    if true {
                        if !(slice.len() <= Self::LANES) {
                            ::core::panicking::panic("assertion failed: slice.len() <= Self::LANES")
                        };
                    };
                    unsafe {
                        let offset = Self::LANES - slice.len();
                        let raw_ptr = slice.as_ptr().sub(offset);
                        let simd = Self::new(F::Vec::load_unaligned(raw_ptr));
                        simd.left_lane_shift(offset as u32)
                    }
                }
                /// # Safety
                /// Does not check if the slice goes out of bounds.
                #[inline(always)]
                pub unsafe fn from_slice_unchecked(slice: &[T]) -> Self {
                    if true {
                        if !(slice.len() >= Self::LANES) {
                            ::core::panicking::panic("assertion failed: slice.len() >= Self::LANES")
                        };
                    };
                    unsafe { Self::new(F::Vec::load_unaligned(slice.as_ptr())) }
                }
                #[inline(always)]
                pub fn copy_to_aligned_slice(self, slice: &mut [T]) {
                    let ptr = slice.as_mut_ptr();
                    if !(ptr.align_offset(Self::SIMD_WIDTH) == 0) {
                        ::core::panicking::panic("assertion failed: ptr.align_offset(Self::SIMD_WIDTH) == 0")
                    };
                    if !(slice.len() >= Self::LANES) {
                        ::core::panicking::panic("assertion failed: slice.len() >= Self::LANES")
                    };
                    unsafe { self.data.store_aligned(ptr) };
                }
                /// # Safety
                /// Does not check if the slice goes out of bounds.
                #[inline(always)]
                pub unsafe fn copy_to_aligned_slice_unchecked(self,
                    slice: &mut [T]) {
                    let ptr = slice.as_mut_ptr();
                    if true {
                        if !(ptr.align_offset(Self::SIMD_WIDTH) == 0) {
                            ::core::panicking::panic("assertion failed: ptr.align_offset(Self::SIMD_WIDTH) == 0")
                        };
                    };
                    if true {
                        if !(slice.len() >= Self::LANES) {
                            ::core::panicking::panic("assertion failed: slice.len() >= Self::LANES")
                        };
                    };
                    unsafe { self.data.store_aligned(ptr) };
                }
                #[inline(always)]
                pub fn copy_to_slice(self, slice: &mut [T]) {
                    if slice.len() >= Self::LANES {
                        let ptr = slice.as_mut_ptr();
                        unsafe { self.data.store_unaligned(ptr) };
                    } else {
                        let array = self.to_array();
                        slice.iter_mut().zip(array.iter()).for_each(|(src, new)|
                                *src = *new);
                    }
                }
                /// # Safety
                /// Does not check if the slice goes out of bounds.
                #[inline(always)]
                pub unsafe fn copy_to_slice_unchecked(self, slice: &mut [T]) {
                    let ptr = slice.as_mut_ptr();
                    if true {
                        if !(slice.len() >= Self::LANES) {
                            ::core::panicking::panic("assertion failed: slice.len() >= Self::LANES")
                        };
                    };
                    unsafe { self.data.store_unaligned(ptr) };
                }
                /// Converts the Simd register into an array.
                ///
                /// # Example
                ///
                /// TODO
                /// use quick_noise::simd::
                #[inline(always)]
                pub fn to_array(self) -> T::Array<F> {
                    let mut array =
                        T::Array::<F>::from_fn(|_| T::from(0).unwrap());
                    self.copy_to_slice(array.as_mut_slice());
                    array
                }
                #[inline(always)]
                pub fn iota(offset: T) -> Self {
                    let iota_array =
                        T::Array::<F>::from_fn(|i|
                                <T as NumCast>::from(i).unwrap().safe_add(offset));
                    Self::from_slice(iota_array.as_slice())
                }
            }
            impl<T: SimdElement, F: Arch> fmt::Debug for Simd<T, F> {
                #[inline(always)]
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    let buf = self.to_array();
                    f.write_fmt(format_args!("{0:?}", buf))
                }
            }
            impl<T: SimdElement, F: Arch> AddAssign for Simd<T, F> where
                Self: Add<Output = Self> + Copy {
                #[inline(always)]
                fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
            }
            impl<T: SimdElement, F: Arch> SubAssign for Simd<T, F> where
                Self: Sub<Output = Self> + Copy {
                #[inline(always)]
                fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs; }
            }
            impl<T: SimdElement, F: Arch> MulAssign for Simd<T, F> where
                Self: Mul<Output = Self> + Copy {
                #[inline(always)]
                fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs; }
            }
            impl<T: SimdElement, F: Arch> DivAssign for Simd<T, F> where
                Self: Div<Output = Self> + Copy {
                #[inline(always)]
                fn div_assign(&mut self, rhs: Self) { *self = *self / rhs; }
            }
            impl<T: SimdElement, F: Arch> RemAssign for Simd<T, F> where
                Self: Rem<Output = Self> + Copy {
                #[inline(always)]
                fn rem_assign(&mut self, rhs: Self) { *self = *self % rhs; }
            }
            impl<T: SimdElement, F: Arch> BitAndAssign for Simd<T, F> where
                Self: BitAnd<Output = Self> + Copy {
                #[inline(always)]
                fn bitand_assign(&mut self, rhs: Self) {
                    *self = *self & rhs;
                }
            }
            impl<T: SimdElement, F: Arch> BitOrAssign for Simd<T, F> where
                Self: BitOr<Output = Self> + Copy {
                #[inline(always)]
                fn bitor_assign(&mut self, rhs: Self) { *self = *self | rhs; }
            }
            impl<T: SimdElement, F: Arch> BitXorAssign for Simd<T, F> where
                Self: BitXor<Output = Self> + Copy {
                #[inline(always)]
                fn bitxor_assign(&mut self, rhs: Self) {
                    *self = *self ^ rhs;
                }
            }
            impl<T: SimdElement, F: Arch> Neg for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn neg(self) -> Self { Self::zero() - self }
            }
            impl<T: SimdElement, F: Arch> Simd<T, F> {
                #[inline(always)]
                pub fn raw_cast<S: SimdElement>(self) -> Simd<S, F> {
                    Simd::new(self.data)
                }
            }
            impl<T: SimdElement, F: Arch> Default for Simd<T, F> {
                #[inline(always)]
                fn default() -> Self {
                    Self::splat(<T as NumCast>::from(T::default()).unwrap())
                }
            }
            impl<T: SimdElement, F: Arch> Add for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn add(self, rhs: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.f64_add(rhs.data),
                                SimdType::F32 => self.data.f32_add(rhs.data),
                                SimdType::I64 => self.data.i64_add(rhs.data),
                                SimdType::I32 => self.data.i32_add(rhs.data),
                                SimdType::I16 => self.data.i16_add(rhs.data),
                                SimdType::I8 => self.data.i8_add(rhs.data),
                                SimdType::U64 => self.data.i64_add(rhs.data),
                                SimdType::U32 => self.data.i32_add(rhs.data),
                                SimdType::U16 => self.data.i16_add(rhs.data),
                                SimdType::U8 => self.data.i8_add(rhs.data),
                            })
                    }
                }
            }
            impl<T: SimdElement, F: Arch> Sub for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn sub(self, rhs: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.f64_sub(rhs.data),
                                SimdType::F32 => self.data.f32_sub(rhs.data),
                                SimdType::I64 => self.data.i64_sub(rhs.data),
                                SimdType::I32 => self.data.i32_sub(rhs.data),
                                SimdType::I16 => self.data.i16_sub(rhs.data),
                                SimdType::I8 => self.data.i8_sub(rhs.data),
                                SimdType::U64 => self.data.i64_sub(rhs.data),
                                SimdType::U32 => self.data.i32_sub(rhs.data),
                                SimdType::U16 => self.data.i16_sub(rhs.data),
                                SimdType::U8 => self.data.i8_sub(rhs.data),
                            })
                    }
                }
            }
            impl<T: SimdElement, F: Arch> Simd<T, F> {
                /// Clamps the values in a register between two bounds, inclusive.
                ///
                /// # Parameters:
                /// - `lower_bound`: Minimum value after clamping
                /// - `upper_bound`: Maximum value after clamping
                #[inline(always)]
                pub fn clamp(self, lower_bound: Self, upper_bound: Self)
                    -> Self {
                    self.min(upper_bound).max(lower_bound)
                }
                /// Shifts bytes N to the left within 128-bit blocks.
                #[inline(always)]
                pub fn block_left_byte_shift<const N : i32>(self) -> Self {
                    unsafe { Self::new(self.data.block_left_byte_shift::<N>()) }
                }
                /// Shifts bytes N to the right within 128-bit blocks.
                #[inline(always)]
                pub fn block_right_byte_shift<const N : i32>(self) -> Self {
                    unsafe {
                        Self::new(self.data.block_right_byte_shift::<N>())
                    }
                }
                /// Blends two registers using an immediate mask.
                #[inline(always)]
                pub fn blend<const N : i32>(self, false_values: Self)
                    -> Self {
                    unsafe {
                        match T::BIT_SIZE {
                            BitSize::Size64 =>
                                Self::new(self.data.blend_32::<N>(false_values.data)),
                            BitSize::Size32 =>
                                Self::new(self.data.blend_32::<N>(false_values.data)),
                            _ =>
                                ::core::panicking::panic("internal error: entered unreachable code"),
                        }
                    }
                }
            }
            impl<T: SimdElement, F: Arch> Simd<T, F> {
                /// Broadcasts a value across the entire register.
                #[inline(always)]
                pub fn splat(val: T) -> Self {
                    unsafe {
                        Self::new(match T::BIT_SIZE {
                                BitSize::Size64 => F::Vec::splat_64(val),
                                BitSize::Size32 => F::Vec::splat_32(val),
                                BitSize::Size16 => F::Vec::splat_16(val),
                                BitSize::Size8 => F::Vec::splat_8(val),
                            })
                    }
                }
                /// Loads a register according to a mask.
                #[inline(always)]
                pub fn masked_load(slice: &[T], mask: Mask<T, F>) -> Self {
                    unsafe {
                        Self::new(match T::BIT_SIZE {
                                BitSize::Size64 =>
                                    F::Vec::masked_load_64(slice.as_ptr(), mask.data),
                                BitSize::Size32 =>
                                    F::Vec::masked_load_32(slice.as_ptr(), mask.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                /// Loads only the first `amount` elements into the register.
                pub fn partial_load(slice: &[T], amount: usize) -> Self {
                    if !(slice.len() >= Self::LANES) {
                        {
                            ::core::panicking::panic_fmt(format_args!("Attempted to do a partial load, but index is out of bounds!"));
                        }
                    };
                    let amount_vec =
                        Self::splat(<T as NumCast>::from(amount).unwrap());
                    let mask =
                        Self::iota(<T as
                                            NumCast>::from(0).unwrap()).simd_lt(amount_vec);
                    Self::masked_load(slice, mask)
                }
                /// Loads only the first `amount` elements into the register.
                ///
                /// # Safety
                /// - Slice must be greater than or equal to `ArchSimd::<T>::LANES`
                pub unsafe fn partial_load_unchecked(slice: &[T],
                    amount: usize) -> Self {
                    if true {
                        if !(slice.len() >= Self::LANES) {
                            {
                                ::core::panicking::panic_fmt(format_args!("Index is out of bounds in unsafe code!"));
                            }
                        };
                    };
                    let amount_vec =
                        Self::splat(<T as NumCast>::from(amount).unwrap());
                    let mask =
                        Self::iota(<T as
                                            NumCast>::from(0).unwrap()).simd_lt(amount_vec);
                    Self::masked_load(slice, mask)
                }
                /// Stores the register using a given mask.
                #[inline(always)]
                pub fn masked_store(self, slice: &mut [T], mask: Mask<T, F>) {
                    unsafe {
                        match T::BIT_SIZE {
                            BitSize::Size64 => {
                                F::Vec::masked_store_64(self.data, slice.as_mut_ptr(),
                                    mask.data)
                            }
                            BitSize::Size32 => {
                                F::Vec::masked_store_32(self.data, slice.as_mut_ptr(),
                                    mask.data)
                            }
                            _ =>
                                ::core::panicking::panic("internal error: entered unreachable code"),
                        }
                    }
                }
                /// Stores only the first `amount` elements into the register.
                pub fn partial_store(self, slice: &mut [T], amount: usize) {
                    match T::BIT_SIZE {
                        BitSize::Size64 => {
                            let iota = Simd::iota(0u64);
                            let n_vec = Simd::splat(amount as u64);
                            let mask = n_vec.simd_gt(iota);
                            Self::masked_store(self, slice, mask.raw_cast());
                        }
                        BitSize::Size32 => {
                            let iota = Simd::iota(0u32);
                            let n_vec = Simd::splat(amount as u32);
                            let mask = n_vec.simd_gt(iota);
                            Self::masked_store(self, slice, mask.raw_cast());
                        }
                        _ =>
                            ::core::panicking::panic("internal error: entered unreachable code"),
                    }
                }
                #[inline(always)]
                pub fn simd_eq(self, rhs: Self) -> Mask<T, F> {
                    unsafe {
                        Mask::new(match T::TYPE {
                                SimdType::F64 => self.data.cmp_f64_eq(rhs.data),
                                SimdType::F32 => self.data.cmp_f32_eq(rhs.data),
                                SimdType::U64 => self.data.cmp_i64_eq(rhs.data),
                                SimdType::U32 => self.data.cmp_i32_eq(rhs.data),
                                SimdType::U16 => self.data.cmp_i16_eq(rhs.data),
                                SimdType::U8 => self.data.cmp_i8_eq(rhs.data),
                                SimdType::I64 => self.data.cmp_i64_eq(rhs.data),
                                SimdType::I32 => self.data.cmp_i32_eq(rhs.data),
                                SimdType::I16 => self.data.cmp_i16_eq(rhs.data),
                                SimdType::I8 => self.data.cmp_i8_eq(rhs.data),
                            })
                    }
                }
                #[inline(always)]
                pub fn simd_neq(self, rhs: Self) -> Mask<T, F> {
                    unsafe {
                        Mask::new(match T::TYPE {
                                SimdType::F64 => self.data.cmp_f64_neq(rhs.data),
                                SimdType::F32 => self.data.cmp_f32_neq(rhs.data),
                                _ => self.simd_eq(rhs).data,
                            })
                    }
                }
                #[inline(always)]
                pub fn simd_gt(self, rhs: Self) -> Mask<T, F> {
                    unsafe {
                        Mask::new(match T::TYPE {
                                SimdType::F64 => self.data.cmp_f64_gt(rhs.data),
                                SimdType::F32 => self.data.cmp_f32_gt(rhs.data),
                                SimdType::U64 => self.data.cmp_i64_gt(rhs.data),
                                SimdType::U32 => self.data.cmp_i32_gt(rhs.data),
                                SimdType::U16 => self.data.cmp_i16_gt(rhs.data),
                                SimdType::U8 => self.data.cmp_i8_gt(rhs.data),
                                SimdType::I64 => self.data.cmp_i64_gt(rhs.data),
                                SimdType::I32 => self.data.cmp_i32_gt(rhs.data),
                                SimdType::I16 => self.data.cmp_i16_gt(rhs.data),
                                SimdType::I8 => self.data.cmp_i8_gt(rhs.data),
                            })
                    }
                }
                #[inline(always)]
                pub fn simd_ge(self, rhs: Self) -> Mask<T, F> {
                    unsafe {
                        Mask::new(match T::TYPE {
                                SimdType::F64 => self.data.cmp_f64_ge(rhs.data),
                                SimdType::F32 => self.data.cmp_f32_ge(rhs.data),
                                SimdType::I64 =>
                                    self.data.cmp_i64_gt(rhs.data).or(self.data.cmp_i64_eq(rhs.data)),
                                SimdType::I32 =>
                                    self.data.cmp_i32_gt(rhs.data).or(self.data.cmp_i32_eq(rhs.data)),
                                SimdType::I16 =>
                                    self.data.cmp_i16_gt(rhs.data).or(self.data.cmp_i16_eq(rhs.data)),
                                SimdType::I8 =>
                                    self.data.cmp_i8_gt(rhs.data).or(self.data.cmp_i8_eq(rhs.data)),
                                _ => {
                                    ::core::panicking::panic_fmt(format_args!("Unsigned integer types for less than not implemented!"));
                                }
                            })
                    }
                }
                #[inline(always)]
                pub fn simd_lt(self, rhs: Self) -> Mask<T, F> {
                    unsafe {
                        Mask::new(match T::TYPE {
                                SimdType::F64 => self.data.cmp_f64_lt(rhs.data),
                                SimdType::F32 => self.data.cmp_f32_lt(rhs.data),
                                _ => {
                                    ::core::panicking::panic_fmt(format_args!("Less than for integers not implemented!"));
                                }
                            })
                    }
                }
                #[inline(always)]
                pub fn simd_le(self, rhs: Self) -> Mask<T, F> {
                    unsafe {
                        Mask::new(match T::TYPE {
                                SimdType::F64 => self.data.cmp_f64_le(rhs.data),
                                SimdType::F32 => self.data.cmp_f32_le(rhs.data),
                                _ => {
                                    ::core::panicking::panic_fmt(format_args!("Less than or equal not implemented for integers!"));
                                }
                            })
                    }
                }
                pub fn max(self, rhs: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.max_f64(rhs.data),
                                SimdType::F32 => self.data.max_f32(rhs.data),
                                SimdType::I32 => self.data.max_i32(rhs.data),
                                SimdType::I16 => self.data.max_i16(rhs.data),
                                SimdType::I8 => self.data.max_i8(rhs.data),
                                SimdType::U32 => self.data.max_u32(rhs.data),
                                SimdType::U16 => self.data.max_u16(rhs.data),
                                SimdType::U8 => self.data.max_u8(rhs.data),
                                _ => {
                                    ::core::panicking::panic_fmt(format_args!("Max for U64/I64 not implemented!"));
                                }
                            })
                    }
                }
                pub fn min(self, rhs: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.min_f64(rhs.data),
                                SimdType::F32 => self.data.min_f32(rhs.data),
                                SimdType::I32 => self.data.min_i32(rhs.data),
                                SimdType::I16 => self.data.min_i16(rhs.data),
                                SimdType::I8 => self.data.min_i8(rhs.data),
                                SimdType::U32 => self.data.min_u32(rhs.data),
                                SimdType::U16 => self.data.min_u16(rhs.data),
                                SimdType::U8 => self.data.min_u8(rhs.data),
                                _ => {
                                    ::core::panicking::panic_fmt(format_args!("Min for U64/I64 not implemented!"));
                                }
                            })
                    }
                }
            }
            impl<T: SimdElement + SimdElement<BitWidthType = B32>, F: Arch>
                Simd<T, F> {
                #[inline(always)]
                pub fn permute_32(self, indices: Simd<u32, F>) -> Self {
                    unsafe { Self::new(self.data.permute_32(indices.data)) }
                }
            }
            impl<T: SimdElement, F: Arch> Simd<T, F> {
                #[inline(always)]
                pub fn permute_8(self, indices: Simd<u8, F>) -> Self {
                    unsafe { Self::new(self.data.permute_8(indices.data)) }
                }
                #[inline(always)]
                pub fn permute_8_pattern_32(self, indices: [u8; 4]) -> Self {
                    let pattern = u32::from_ne_bytes(indices);
                    let pattern_vec = Simd::<u32, F>::splat(pattern);
                    unsafe { Self::new(self.data.permute_8(pattern_vec.data)) }
                }
            }
            impl<F: Arch> Simd<u32, F> {
                pub fn gather<S: SimdElement +
                    SimdElement<BitWidthType = B32>, const N :
                    usize>(self, slice: &[S; N]) -> Simd<S, F> {
                    if N <= Self::LANES {
                        let data = Simd::<S, F>::from_slice(&slice[..]);
                        data.permute_32(self)
                    } else {
                        unsafe {
                            Simd::new(self.data.gather_32_from_32::<S,
                                    4>(slice.as_ptr()))
                        }
                    }
                }
            }
            impl<F: Arch> Simd<u64, F> {
                pub fn gather<S: SimdElement +
                    SimdElement<BitWidthType = B64>, const N :
                    usize>(self, slice: &[S; N]) -> Simd<S, F> {
                    unsafe {
                        Simd::new(self.data.gather_64_from_64::<S,
                                8>(slice.as_ptr()))
                    }
                }
            }
            impl<T: SimdElement, F: Arch> Simd<T, F> {
                pub fn left_lane_shift(self, n: u32) -> Self {
                    match T::BIT_SIZE {
                        BitSize::Size32 => unsafe {
                            Self::new(self.data.left_lane_shift_32(n))
                        },
                        _ =>
                            ::core::panicking::panic("internal error: entered unreachable code"),
                    }
                }
                pub fn right_lane_shift(self, n: u32) -> Self {
                    match T::BIT_SIZE {
                        BitSize::Size32 => unsafe {
                            Self::new(self.data.right_lane_shift_32(n))
                        },
                        _ =>
                            ::core::panicking::panic("internal error: entered unreachable code"),
                    }
                }
            }
        }
        pub mod integer {
            use std::ops::*;
            use num_traits::NumCast;
            use crate::simd::architectures::interface::*;
            use crate::simd::register::Simd;
            use crate::simd::traits::*;
            impl<T: SimdInteger, F: Arch> BitAnd for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn bitand(self, rhs: Self) -> Self {
                    unsafe { Self::new(F::Vec::and(self.data, rhs.data)) }
                }
            }
            impl<T: SimdInteger, F: Arch> BitOr for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn bitor(self, rhs: Self) -> Self {
                    unsafe { Self::new(F::Vec::or(self.data, rhs.data)) }
                }
            }
            impl<T: SimdInteger, F: Arch> BitXor for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn bitxor(self, rhs: Self) -> Self {
                    unsafe { Self::new(F::Vec::xor(self.data, rhs.data)) }
                }
            }
            impl<T: SimdInteger, F: Arch> Simd<T, F> {
                #[inline(always)]
                pub fn andnot(self, rhs: Self) -> Self {
                    unsafe { Self::new(F::Vec::and_not(self.data, rhs.data)) }
                }
            }
            impl<T: SimdInteger, F: Arch> Not for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn not(self) -> Self {
                    unsafe { Self::new(F::Vec::not(self.data)) }
                }
            }
            impl<T: SimdIntegerNotByte, F: Arch>
                Shl<Simd<<T as SimdInteger>::Unsigned, F>> for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn shl(self, rhs: Simd<<T as SimdInteger>::Unsigned, F>)
                    -> Self {
                    Self::new(unsafe {
                            match T::BIT_SIZE {
                                BitSize::Size64 => self.data.sllv_64(rhs.data),
                                BitSize::Size32 => self.data.sllv_32(rhs.data),
                                BitSize::Size16 => self.data.sllv_16(rhs.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            }
                        })
                }
            }
            impl<T: SimdIntegerNotByte, F: Arch>
                Shr<Simd<<T as SimdInteger>::Unsigned, F>> for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn shr(self, rhs: Simd<<T as SimdInteger>::Unsigned, F>)
                    -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::U64 => self.data.srlv_64(rhs.data),
                                SimdType::U32 => self.data.srlv_32(rhs.data),
                                SimdType::U16 => self.data.srlv_16(rhs.data),
                                SimdType::I64 => self.data.srav_64(rhs.data),
                                SimdType::I32 => self.data.srav_32(rhs.data),
                                SimdType::I16 => self.data.srav_16(rhs.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
            }
            impl<T: SimdIntegerNotByte, F: Arch> Shl<usize> for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn shl(self, rhs: usize) -> Self {
                    let shift =
                        Simd::<<T as SimdInteger>::Unsigned,
                                F>::splat(NumCast::from(rhs).unwrap());
                    self << shift
                }
            }
            impl<T: SimdIntegerNotByte, F: Arch> Shr<usize> for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn shr(self, rhs: usize) -> Self {
                    let shift =
                        Simd::<<T as SimdInteger>::Unsigned,
                                F>::splat(NumCast::from(rhs).unwrap());
                    self >> shift
                }
            }
            impl<T: SimdMulType, F: Arch> Mul for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn mul(self, rhs: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.f64_mul(rhs.data),
                                SimdType::F32 => self.data.f32_mul(rhs.data),
                                SimdType::I32 => self.data.i32_mul(rhs.data),
                                SimdType::I16 => self.data.i16_mul(rhs.data),
                                SimdType::U32 => self.data.i32_mul(rhs.data),
                                SimdType::U16 => self.data.i16_mul(rhs.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
            }
            impl<T: SimdFloat, F: Arch> Div for Simd<T, F> {
                type Output = Self;
                #[inline(always)]
                fn div(self, rhs: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.f64_div(rhs.data),
                                SimdType::F32 => self.data.f32_div(rhs.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
            }
            impl<T: SimdInteger + HasSigned, F: Arch> Simd<T, F> {
                #[inline(always)]
                pub fn cast_signed(self)
                    -> Simd<<T as SimdInteger>::Signed, F> {
                    Simd::new(self.data)
                }
            }
            impl<T: SimdInteger + HasUnsigned, F: Arch> Simd<T, F> {
                #[inline(always)]
                pub fn cast_unsigned(self)
                    -> Simd<<T as SimdInteger>::Unsigned, F> {
                    Simd::new(self.data)
                }
            }
            impl<T: SimdInteger + HasFloat, F: Arch> Simd<T, F> {
                #[inline(always)]
                pub fn cast_float(self) -> Simd<<T as HasFloat>::Float, F> {
                    unsafe { Simd::new(self.data.int_to_float()) }
                }
            }
        }
        pub mod float {
            use num_traits::NumCast;
            use crate::simd::architectures::interface::*;
            use crate::simd::register::Simd;
            use crate::simd::traits::*;
            impl<T: SimdFloat, F: Arch> Simd<T, F> {
                #[inline(always)]
                pub fn floor(self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.floor_f64(),
                                SimdType::F32 => self.data.floor_f32(),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                #[inline(always)]
                pub fn round(self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.round_f64(),
                                SimdType::F32 => self.data.round_f32(),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                #[inline(always)]
                pub fn ceil(self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.ceil_f64(),
                                SimdType::F32 => self.data.ceil_f32(),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                #[inline(always)]
                pub fn fract(self) -> Self { self - self.floor() }
                #[inline(always)]
                pub fn mul_add(self, mult: Self, add: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.mul_add_f64(mult.data, add.data),
                                SimdType::F32 => self.data.mul_add_f32(mult.data, add.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                #[inline(always)]
                pub fn mul_sub(self, mult: Self, sub: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.mul_sub_f64(mult.data, sub.data),
                                SimdType::F32 => self.data.mul_sub_f32(mult.data, sub.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                #[inline(always)]
                pub fn negated_mul_add(self, mult: Self, add: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 =>
                                    self.data.negated_mul_add_f64(mult.data, add.data),
                                SimdType::F32 =>
                                    self.data.negated_mul_add_f32(mult.data, add.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                #[inline(always)]
                pub fn negated_mul_sub(self, mult: Self, sub: Self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 =>
                                    self.data.negated_mul_sub_f64(mult.data, sub.data),
                                SimdType::F32 =>
                                    self.data.negated_mul_sub_f32(mult.data, sub.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                #[inline(always)]
                pub fn cast_int_trunc(self) -> Simd<T::Signed, F> {
                    unsafe { Simd::new(self.data.float_to_int_trunc()) }
                }
                #[inline(always)]
                pub fn cast_int_round(self) -> Simd<T::Signed, F> {
                    unsafe { Simd::new(self.data.float_to_int_round()) }
                }
                #[inline(always)]
                pub fn cast_uint_trunc(self) -> Simd<T::Unsigned, F> {
                    unsafe { Simd::new(self.data.float_to_int_trunc()) }
                }
                #[inline(always)]
                pub fn cast_uint_round(self) -> Simd<T::Unsigned, F> {
                    unsafe { Simd::new(self.data.float_to_int_round()) }
                }
                #[inline(always)]
                pub fn quintic_lerp(self) -> Self {
                    let six = Self::splat(NumCast::from(6.0).unwrap());
                    let ten = Self::splat(NumCast::from(10.0).unwrap());
                    let fifteen = Self::splat(NumCast::from(15.0).unwrap());
                    let t = self;
                    t * t * t * t.mul_add(t.mul_sub(six, fifteen), ten)
                }
                #[inline(always)]
                pub fn cubic_lerp(self) -> Self {
                    let neg_two = Self::splat(NumCast::from(-2.0).unwrap());
                    let three = Self::splat(NumCast::from(3.0).unwrap());
                    let t = self;
                    t * t * t.mul_add(neg_two, three)
                }
                pub fn sqrt(self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 => self.data.sqrt_f64(),
                                SimdType::F32 => self.data.sqrt_f32(),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                pub fn abs(self) -> Self {
                    unsafe {
                        Self::new(match T::TYPE {
                                SimdType::F64 =>
                                    Simd::<u64,
                                                    F>::splat(T::SIGN_MASK as u64).data.and(self.data),
                                SimdType::F32 =>
                                    Simd::<u32,
                                                    F>::splat(T::SIGN_MASK as u32).data.and(self.data),
                                _ =>
                                    ::core::panicking::panic("internal error: entered unreachable code"),
                            })
                    }
                }
                /// Only rsqrt_f32 currently supported.
                pub fn rsqrt(self) -> Self {
                    unsafe { Self::new(self.data.rsqrt_f32()) }
                }
            }
        }
        pub mod iters {
            use std::iter::zip;
            use std::marker::PhantomData;
            use std::ops::{Deref, DerefMut};
            use crate::simd::static_simd::{ArchFamily, StaticSimd};
            use crate::simd::architectures::interface::*;
            use crate::simd::array_trait::Array;
            use crate::simd::register::Simd;
            use crate::simd::traits::*;
            pub trait SimdSliceIterExt<T: SimdElement> {
                fn simd_iter<'a>(&'a self)
                -> SimdSliceIter<'a, T, ArchFamily>;
                fn simd_iter_mut<'a>(&'a mut self)
                -> SimdSliceIterMut<'a, T, ArchFamily>;
            }
            impl<T: SimdElement> SimdSliceIterExt<T> for [T] {
                /// Creates an iterator of simd chunks.
                fn simd_iter<'a>(&'a self)
                    -> SimdSliceIter<'a, T, ArchFamily> {
                    SimdSliceIter {
                        slice: self,
                        index: 0,
                        _architecture: PhantomData::<ArchFamily>,
                    }
                }
                /// Creates an iterator of mutable simd chunks.
                fn simd_iter_mut<'a>(&'a mut self)
                    -> SimdSliceIterMut<'a, T, ArchFamily> {
                    SimdSliceIterMut {
                        slice: self,
                        _architecture: PhantomData::<ArchFamily>,
                    }
                }
            }
            pub struct SimdSliceIter<'a, T: SimdElement, F: Arch> {
                slice: &'a [T],
                index: usize,
                _architecture: PhantomData<F>,
            }
            impl<'a, T: SimdElement, F: Arch> Iterator for
                SimdSliceIter<'a, T, F> {
                type Item = Simd<T, F>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.index == self.slice.len() { return None; }
                    if self.slice.len() < Self::Item::LANES {
                        let mut array = Self::Item::zero().to_array();
                        for i in 0..self.slice.len() { array[i] = self.slice[i]; }
                        let result = Self::Item::from_slice(array.as_mut_slice());
                        self.index = self.slice.len();
                        return Some(result);
                    }
                    let amount_left = self.slice.len() - self.index;
                    if amount_left < Self::Item::LANES {
                        let offset =
                            Self::Item::LANES - (self.slice.len() - self.index);
                        let new_index = self.index - offset;
                        let simd =
                            unsafe {
                                Self::Item::from_slice(self.slice.get_unchecked(new_index..))
                            };
                        let simd_shifted = simd.left_lane_shift(offset as u32);
                        self.index = self.slice.len();
                        return Some(simd_shifted);
                    }
                    let next =
                        unsafe {
                            Self::Item::from_slice_unchecked(self.slice.get_unchecked(self.index..))
                        };
                    self.index += Self::Item::LANES;
                    Some(next)
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    let amount_left = self.slice.len() - self.index;
                    let rem_chunks = amount_left.div_ceil(Self::Item::LANES);
                    (rem_chunks, Some(rem_chunks))
                }
            }
            impl<'a, T: SimdElement, F: Arch> ExactSizeIterator for
                SimdSliceIter<'a, T, F> {}
            pub struct SimdSliceChunk<'a, T: SimdElement, F: Arch> {
                simd: Simd<T, F>,
                slice: &'a mut [T],
            }
            impl<'a, T: SimdElement, F: Arch> SimdSliceChunk<'a, T, F> {
                pub fn new(simd: Simd<T, F>, slice: &'a mut [T]) -> Self {
                    Self { simd, slice }
                }
            }
            impl<'a, T: SimdElement, F: Arch> Deref for
                SimdSliceChunk<'a, T, F> {
                type Target = Simd<T, F>;
                fn deref(&self) -> &Self::Target { &self.simd }
            }
            impl<'a, T: SimdElement, F: Arch> DerefMut for
                SimdSliceChunk<'a, T, F> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.simd
                }
            }
            impl<'a, T: SimdElement, F: Arch> Drop for
                SimdSliceChunk<'a, T, F> {
                #[inline(always)]
                fn drop(&mut self) {
                    if self.slice.len() >= Simd::<T, F>::LANES {
                        unsafe { self.simd.copy_to_slice_unchecked(self.slice) };
                    } else {
                        let array = self.simd.to_array();
                        self.slice.iter_mut().zip(array.iter()).for_each(|(src,
                                    new)| *src = *new);
                    }
                }
            }
            pub struct SimdSliceIterMut<'a, T: SimdElement, F: Arch> {
                slice: &'a mut [T],
                _architecture: PhantomData<F>,
            }
            impl<'a, T: SimdElement, F: Arch> Iterator for
                SimdSliceIterMut<'a, T, F> {
                type Item = SimdSliceChunk<'a, T, F>;
                #[inline(always)]
                fn next(&mut self) -> Option<Self::Item> {
                    let slice_len = self.slice.len();
                    if self.slice.is_empty() { return None; }
                    let slice = std::mem::take(&mut self.slice);
                    if slice_len < Simd::<T, F>::LANES {
                        let mut array = Simd::<T, F>::zero().to_array();
                        for i in 0..slice_len { array[i] = slice[i]; }
                        let simd =
                            unsafe {
                                Simd::<T, F>::from_slice_unchecked(array.as_mut_slice())
                            };
                        let chunk = SimdSliceChunk::new(simd, slice);
                        return Some(chunk);
                    }
                    let (cur_slice, rem_slice) =
                        slice.split_at_mut(Simd::<T, F>::LANES);
                    self.slice = rem_slice;
                    let next_simd =
                        unsafe { Simd::from_slice_unchecked(cur_slice) };
                    let chunk = SimdSliceChunk::new(next_simd, cur_slice);
                    Some(chunk)
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    let rem_chunks =
                        self.slice.len().div_ceil(Simd::<T, F>::LANES);
                    (rem_chunks, Some(rem_chunks))
                }
            }
            impl<'a, T: SimdElement, F: Arch> ExactSizeIterator for
                SimdSliceIterMut<'a, T, F> {}
            pub trait IntoSimdIterator<T: SimdElement> {
                fn into_simd_iter(self)
                -> SimdVecIntoIter<T, ArchFamily>;
            }
            impl<T: SimdElement> IntoSimdIterator<T> for Vec<T> {
                fn into_simd_iter(self) -> SimdVecIntoIter<T, ArchFamily> {
                    SimdVecIntoIter {
                        vec: self,
                        index: 0,
                        _architecture: PhantomData::<ArchFamily>,
                    }
                }
            }
            pub struct SimdVecIntoIter<T: SimdElement, F: Arch> {
                vec: Vec<T>,
                index: usize,
                _architecture: PhantomData<F>,
            }
            impl<T: SimdElement, F: Arch> Iterator for SimdVecIntoIter<T, F> {
                type Item = Simd<T, F>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.index >= self.vec.len() { return None; }
                    if self.vec.len() < Self::Item::LANES {
                        let mut array = Self::Item::zero().to_array();
                        for i in 0..self.vec.len() { array[i] = self.vec[i]; }
                        let result = Self::Item::from_slice(array.as_mut_slice());
                        self.index = self.vec.len();
                        return Some(result);
                    }
                    let amount_left = self.vec.len() - self.index;
                    if amount_left < Self::Item::LANES {
                        let offset =
                            Self::Item::LANES - (self.vec.len() - self.index);
                        let new_index = self.index - offset;
                        let simd =
                            unsafe {
                                Self::Item::from_slice(self.vec.get_unchecked(new_index..))
                            };
                        let simd_shifted = simd.left_lane_shift(offset as u32);
                        self.index = self.vec.len();
                        return Some(simd_shifted);
                    }
                    let next =
                        unsafe {
                            Self::Item::from_slice_unchecked(self.vec.get_unchecked(self.index..))
                        };
                    self.index += Self::Item::LANES;
                    Some(next)
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    let amount_left = self.vec.len() - self.index;
                    let rem_chunks = amount_left.div_ceil(Self::Item::LANES);
                    (rem_chunks, Some(rem_chunks))
                }
            }
            impl<T: SimdElement, F: Arch> ExactSizeIterator for
                SimdVecIntoIter<T, F> {}
            impl<T: SimdElement, F: Arch, const N : usize>
                FromIterator<Simd<T, F>> for [T; N] {
                fn from_iter<I: IntoIterator<Item = Simd<T, F>>>(iter: I)
                    -> Self {
                    let mut array = [T::default(); N];
                    let lane_iter = (0..N).step_by(Simd::<T, F>::LANES);
                    for (i, x) in zip(lane_iter, iter) {
                        x.copy_to_slice(&mut array[i..]);
                    }
                    array
                }
            }
            impl<T: SimdElement, F: Arch> FromIterator<Simd<T, F>> for Vec<T>
                {
                fn from_iter<I: IntoIterator<Item = Simd<T, F>>>(iter: I)
                    -> Self {
                    let iter = iter.into_iter();
                    let (lower_bound, upper_bound) = iter.size_hint();
                    if let Some(upper_bound) = upper_bound {
                        let mut vec =
                            ::alloc::vec::from_elem(T::default(),
                                upper_bound * StaticSimd::<T>::LANES);
                        for (i, x) in iter.enumerate() {
                            x.copy_to_slice(&mut vec[i * StaticSimd::<T>::LANES..]);
                        }
                        vec
                    } else {
                        let mut vec = Vec::with_capacity(lower_bound);
                        for x in iter {
                            let array = x.to_array();
                            vec.extend_from_slice(array.as_slice());
                        }
                        vec
                    }
                }
            }
        }
        #[repr(transparent)]
        pub struct Simd<T: SimdElement, F: Arch> {
            pub(crate) data: F::Vec,
            pub(crate) _marker: PhantomData<T>,
        }
        #[automatically_derived]
        impl<T: ::core::clone::Clone + SimdElement, F: ::core::clone::Clone +
            Arch> ::core::clone::Clone for Simd<T, F> where
            F::Vec: ::core::clone::Clone {
            #[inline]
            fn clone(&self) -> Simd<T, F> {
                Simd {
                    data: ::core::clone::Clone::clone(&self.data),
                    _marker: ::core::clone::Clone::clone(&self._marker),
                }
            }
        }
        #[automatically_derived]
        impl<T: ::core::marker::Copy + SimdElement, F: ::core::marker::Copy +
            Arch> ::core::marker::Copy for Simd<T, F> where
            F::Vec: ::core::marker::Copy {
        }
        impl<T: SimdElement, F: Arch> Simd<T, F> {
            pub const SIMD_WIDTH: usize = F::SIMD_WIDTH;
            pub const LANE_SIZE: usize = std::mem::size_of::<T>();
            pub const LANES: usize = F::SIMD_WIDTH / Self::LANE_SIZE;
        }
    }
    pub mod traits {
        use std::fmt::Debug;
        use num_traits::{NumCast, NumOps};
        use crate::simd::architectures::interface::Arch;
        use crate::simd::array_trait::Array;
        mod private {
            pub trait SealedTypes {}
            impl SealedTypes for f64 {}
            impl SealedTypes for f32 {}
            impl SealedTypes for i64 {}
            impl SealedTypes for i32 {}
            impl SealedTypes for i16 {}
            impl SealedTypes for i8 {}
            impl SealedTypes for u64 {}
            impl SealedTypes for u32 {}
            impl SealedTypes for u16 {}
            impl SealedTypes for u8 {}
            pub trait SealedSizes {}
            impl SealedSizes for super::B64 {}
            impl SealedSizes for super::B32 {}
            impl SealedSizes for super::B16 {}
            impl SealedSizes for super::B8 {}
        }
        pub enum BitSize { Size64, Size32, Size16, Size8, }
        pub enum PrimitiveType { Float, SignedInt, UnsignedInt, }
        pub enum SimdType { F64, F32, I64, I32, I16, I8, U64, U32, U16, U8, }
        pub struct B64;
        pub struct B32;
        pub struct B16;
        pub struct B8;
        pub trait BitWidth: private::SealedSizes {
            const BIT_SIZE: usize;
        }
        impl BitWidth for B64 {
            const BIT_SIZE: usize = 64;
        }
        impl BitWidth for B32 {
            const BIT_SIZE: usize = 32;
        }
        impl BitWidth for B16 {
            const BIT_SIZE: usize = 16;
        }
        impl BitWidth for B8 {
            const BIT_SIZE: usize = 8;
        }
        pub trait SafeAdd {
            fn safe_add(self, rhs: Self)
            -> Self;
        }
        impl SafeAdd for f64 {
            fn safe_add(self, rhs: Self) -> Self { self + rhs }
        }
        impl SafeAdd for f32 {
            fn safe_add(self, rhs: Self) -> Self { self + rhs }
        }
        impl SafeAdd for u64 {
            fn safe_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        }
        impl SafeAdd for u32 {
            fn safe_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        }
        impl SafeAdd for u16 {
            fn safe_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        }
        impl SafeAdd for u8 {
            fn safe_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        }
        impl SafeAdd for i64 {
            fn safe_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        }
        impl SafeAdd for i32 {
            fn safe_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        }
        impl SafeAdd for i16 {
            fn safe_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        }
        impl SafeAdd for i8 {
            fn safe_add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        }
        pub trait SimdElement: private::SealedTypes + PartialEq + Sized +
            Default + Copy + NumCast + NumOps + Debug + SafeAdd {
            const BIT_SIZE: BitSize;
            const PRIMITIVE_TYPE: PrimitiveType;
            const TYPE: SimdType;
            type BitWidthType: BitWidth;
            type Array<F: Arch>: Debug + Copy + Array<Self>;
            type UType: SimdElement;
        }
        impl SimdElement for f64 {
            const BIT_SIZE: BitSize = BitSize::Size64;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::Float;
            const TYPE: SimdType = SimdType::F64;
            type BitWidthType = B64;
            type Array<F: Arch> = F::Array64<f64>;
            type UType = u64;
        }
        impl SimdElement for f32 {
            const BIT_SIZE: BitSize = BitSize::Size32;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::Float;
            const TYPE: SimdType = SimdType::F32;
            type BitWidthType = B32;
            type Array<F: Arch> = F::Array32<f32>;
            type UType = u32;
        }
        impl SimdElement for i64 {
            const BIT_SIZE: BitSize = BitSize::Size64;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::SignedInt;
            const TYPE: SimdType = SimdType::I64;
            type BitWidthType = B64;
            type Array<F: Arch> = F::Array64<i64>;
            type UType = u64;
        }
        impl SimdElement for i32 {
            const BIT_SIZE: BitSize = BitSize::Size32;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::SignedInt;
            const TYPE: SimdType = SimdType::I32;
            type BitWidthType = B32;
            type Array<F: Arch> = F::Array32<i32>;
            type UType = u32;
        }
        impl SimdElement for i16 {
            const BIT_SIZE: BitSize = BitSize::Size16;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::SignedInt;
            const TYPE: SimdType = SimdType::I16;
            type BitWidthType = B16;
            type Array<F: Arch> = F::Array16<i16>;
            type UType = u16;
        }
        impl SimdElement for i8 {
            const BIT_SIZE: BitSize = BitSize::Size8;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::SignedInt;
            const TYPE: SimdType = SimdType::I8;
            type BitWidthType = B8;
            type Array<F: Arch> = F::Array8<i8>;
            type UType = u8;
        }
        impl SimdElement for u64 {
            const BIT_SIZE: BitSize = BitSize::Size64;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::UnsignedInt;
            const TYPE: SimdType = SimdType::U64;
            type BitWidthType = B64;
            type Array<F: Arch> = F::Array64<u64>;
            type UType = u64;
        }
        impl SimdElement for u32 {
            const BIT_SIZE: BitSize = BitSize::Size32;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::UnsignedInt;
            const TYPE: SimdType = SimdType::U32;
            type BitWidthType = B32;
            type Array<F: Arch> = F::Array32<u32>;
            type UType = u32;
        }
        impl SimdElement for u16 {
            const BIT_SIZE: BitSize = BitSize::Size16;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::UnsignedInt;
            const TYPE: SimdType = SimdType::U16;
            type BitWidthType = B16;
            type Array<F: Arch> = F::Array16<u16>;
            type UType = u16;
        }
        impl SimdElement for u8 {
            const BIT_SIZE: BitSize = BitSize::Size8;
            const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::UnsignedInt;
            const TYPE: SimdType = SimdType::U8;
            type BitWidthType = B8;
            type Array<F: Arch> = F::Array8<u8>;
            type UType = u8;
        }
        pub struct Signed;
        pub struct Unsigned;
        pub trait SimdInteger: SimdElement {
            type Type;
            type Unsigned: SimdElement;
            type Signed: SimdElement;
        }
        impl SimdInteger for i64 {
            type Type = Signed;
            type Unsigned = u64;
            type Signed = i64;
        }
        impl SimdInteger for i32 {
            type Type = Signed;
            type Unsigned = u32;
            type Signed = i32;
        }
        impl SimdInteger for i16 {
            type Type = Signed;
            type Unsigned = u16;
            type Signed = i16;
        }
        impl SimdInteger for i8 {
            type Type = Signed;
            type Unsigned = u8;
            type Signed = i8;
        }
        impl SimdInteger for u64 {
            type Type = Unsigned;
            type Unsigned = u64;
            type Signed = i64;
        }
        impl SimdInteger for u32 {
            type Type = Unsigned;
            type Unsigned = u32;
            type Signed = i32;
        }
        impl SimdInteger for u16 {
            type Type = Unsigned;
            type Unsigned = u16;
            type Signed = i16;
        }
        impl SimdInteger for u8 {
            type Type = Unsigned;
            type Unsigned = u8;
            type Signed = i8;
        }
        pub trait SimdFloat: SimdElement + HasSigned + HasUnsigned +
            SimdMulType {
            const SIGN_MASK: usize;
        }
        impl SimdFloat for f64 {
            const SIGN_MASK: usize = 0x7FFFFFFFFFFFFFFF;
        }
        impl SimdFloat for f32 {
            const SIGN_MASK: usize = 0x7FFFFFFF;
        }
        pub trait SimdWideType: SimdElement {}
        impl SimdWideType for f64 {}
        impl SimdWideType for f32 {}
        impl SimdWideType for i64 {}
        impl SimdWideType for i32 {}
        impl SimdWideType for u64 {}
        impl SimdWideType for u32 {}
        pub trait SimdIntegerNotByte: SimdInteger {}
        impl SimdIntegerNotByte for i64 {}
        impl SimdIntegerNotByte for i32 {}
        impl SimdIntegerNotByte for u64 {}
        impl SimdIntegerNotByte for u32 {}
        impl SimdIntegerNotByte for i16 {}
        impl SimdIntegerNotByte for u16 {}
        pub trait SimdMulType: SimdElement {}
        impl SimdMulType for f64 {}
        impl SimdMulType for f32 {}
        impl SimdMulType for i32 {}
        impl SimdMulType for i16 {}
        impl SimdMulType for u32 {}
        impl SimdMulType for u16 {}
        pub trait HasFloat: SimdElement {
            type Float: SimdElement;
        }
        pub trait HasSigned: SimdElement {
            type Signed: SimdElement;
        }
        pub trait HasUnsigned: SimdElement {
            type Unsigned: SimdElement;
        }
        impl HasSigned for f64 {
            type Signed = i64;
        }
        impl HasUnsigned for f64 {
            type Unsigned = u64;
        }
        impl HasSigned for f32 {
            type Signed = i32;
        }
        impl HasUnsigned for f32 {
            type Unsigned = u32;
        }
        impl HasFloat for i64 {
            type Float = f64;
        }
        impl HasUnsigned for i64 {
            type Unsigned = u64;
        }
        impl HasFloat for i32 {
            type Float = f32;
        }
        impl HasUnsigned for i32 {
            type Unsigned = u32;
        }
        impl HasUnsigned for i16 {
            type Unsigned = u16;
        }
        impl HasUnsigned for i8 {
            type Unsigned = u8;
        }
        impl HasFloat for u64 {
            type Float = f64;
        }
        impl HasSigned for u64 {
            type Signed = i64;
        }
        impl HasFloat for u32 {
            type Float = f32;
        }
        impl HasSigned for u32 {
            type Signed = i32;
        }
        impl HasSigned for u16 {
            type Signed = i16;
        }
        impl HasSigned for u8 {
            type Signed = i8;
        }
    }
    pub mod dispatch {
        pub use crate::simd::architectures::arch::Scalar128;
        pub use crate::simd::architectures::arch::{Avx2, Avx512, Sse};
        pub use crate::simd::architectures::interface::Arch;
        pub enum Architecture { Sse, Avx2, Avx512, Scalar, }
        pub fn detect_architecture() -> Architecture {
            {
                if false || ::std_detect::detect::__is_feature_detected::fma()
                    {
                    if false ||
                            ::std_detect::detect::__is_feature_detected::avx512f() {
                        return Architecture::Avx512;
                    } else if false ||
                            ::std_detect::detect::__is_feature_detected::avx2() {
                        return Architecture::Avx2;
                    }
                }
                if false ||
                        ::std_detect::detect::__is_feature_detected::sse4_2() {
                    return Architecture::Sse;
                }
            }
            Architecture::Scalar
        }
        #[macro_export]
        macro_rules! dispatch {
            ($enum:ident, $func:ident($($args:expr),*)) =>
            {
                match $enum
                {
                    #[cfg(target_arch = "x86_64")] Architecture::Sse =>
                    $func::<Sse>($($args),*), #[cfg(target_arch = "x86_64")]
                    Architecture::Avx2 => $func::<Avx2>($($args),*),
                    #[cfg(target_arch = "x86_64")] Architecture::Avx512 =>
                    $func::<Avx512>($($args),*), #[cfg(target_arch = "aarch64")]
                    Architecture::Neon => $func::<Neon>($($args),*),
                    Architecture::Scalar => $func::<Scalar128>($($args),*)
                }
            };
            ($enum:ident,
            $func:ident::<$($generics:ident),+>($($args:expr),*)) =>
            {
                match $enum
                {
                    #[cfg(target_arch = "x86_64")] Architecture::Sse =>
                    $func::<Sse, $($generics),+>($($args),*),
                    #[cfg(target_arch = "x86_64")] Architecture::Avx2 =>
                    $func::<Avx2, $($generics),+>($($args),*),
                    #[cfg(target_arch = "x86_64")] Architecture::Avx512 =>
                    $func::<Avx512, $($generics),+>($($args),*),
                    #[cfg(target_arch = "aarch64")] Architecture::Neon =>
                    $func::<Neon, $($generics),+>($($args),*),
                    Architecture::Scalar => $func::<Scalar128,
                    $($generics),+>($($args),*)
                }
            };
        }
        pub use dispatch;
    }
    pub use traits::{SimdElement, SimdFloat, SimdInteger};
    pub use architectures::interface::Arch;
    pub use static_simd::*;
    pub use register::Simd;
    pub use mask::Mask;
    pub use register::iters::SimdSliceIterExt;
}
pub use api::batch::interface::{BatchGenerator, BatchNoise};
pub use api::defaults::*;
pub use api::grid::interface::{
    Grid, GridGenerator, GridNoise, GridNoiseParams,
};
pub use api::octave::Octave;
pub use api::{
    BatchNoiseBuilder, GridNoiseBuilder, OctaveBatchNoiseBuilder,
    OctaveGridNoiseBuilder,
};
pub use noise::combiners::{
    Billow, Combiner, CombinerArray, CombinerState, Fbm, HybridMulti, Multi,
    PingPong, Ridged, Terrace,
};
pub use noise::generators::{Cellular, Perlin, Simplex, Value};
pub use noise::*;
Some errors have detailed explanations: E0405, E0432.
For more information about an error, try `rustc --explain E0405`.
warning: `quick-noise` (lib) generated 2 warnings
error: could not compile `quick-noise` (lib) due to 4 previous errors; 2 warnings emitted

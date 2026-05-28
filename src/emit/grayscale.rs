use std::{fs, path::Path};

use crate::cellular::Cellular;
use crate::noise::simplex::Simplex;
use crate::noise::value::Value;
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_traits::*;
use crate::{noise::perlin::*, simd::simd_array::SimdArray};
use std::cmp::min;

// TODO: Add error handling here.
pub trait NoiseImageExt: Iterator<Item = ArchSimd<f32>> + Sized {
    fn to_grayscale_image<const X: usize, const Y: usize>(mut self, path: impl AsRef<Path>) {
        if let Some(parent) = path.as_ref().parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).expect("Failed to create parent");
        }

        let size = X * Y;
        let mut pixels = Vec::<u8>::new();
        pixels.resize(size, 0);

        const LANES: usize = ArchSimd::<f32>::LANES;
        for i in (0..size).step_by(LANES) {
            let cur = self
                .next()
                .expect("Given iterator did not fit image dimensions!");

            let adj = (cur + ArchSimd::splat(1.0)) * ArchSimd::splat(127.5);

            // TODO: Equip simd to do this in register
            let slice = adj.to_array();
            let upper_index = min(size, i + LANES);
            let slice_bound = upper_index - i;
            for m in 0..slice_bound {
                pixels[i + m] = slice[m] as u8;
            }
        }

        image::save_buffer(&path, &pixels, X as u32, Y as u32, image::ColorType::L8)
            .expect("Failed to write height map!");
    }
}

impl<I> NoiseImageExt for I where I: Iterator<Item = ArchSimd<f32>> + Sized {}
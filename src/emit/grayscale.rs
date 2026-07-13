#[cfg(feature = "image")]
use std::cmp::min;
use std::fs;
use std::path::Path;

use crate::simd::arch_simd::ArchSimd;

// TODO: Add error handling here.
pub trait NoiseImageExt: Iterator<Item = ArchSimd<f32>> + Sized {
    fn to_grayscale_image(mut self, x: usize, y: usize, path: impl AsRef<Path>) {
        if let Some(parent) = path.as_ref().parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).expect("Failed to create parent");
        }

        let size = x * y;
        let mut pixels = vec![0; size];

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

        image::save_buffer(&path, &pixels, x as u32, y as u32, image::ColorType::L8)
            .expect("Failed to write height map!");
    }
}

impl<I> NoiseImageExt for I where I: Iterator<Item = ArchSimd<f32>> + Sized {}

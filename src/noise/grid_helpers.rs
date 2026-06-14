use crate::math::vec::{ArithmeticVec, BasicVec};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;

#[inline(always)]
pub(super) fn grid_fill_indices<const M: usize>(
    grid_indices: &mut SimdArray<u32, M>,
    distances: &SimdArray<f32, M>,
    num_loops: &mut usize,
) {
    const LANES: usize = ArchSimd::<f32>::LANES;

    // Handle scalar case first.
    if M < LANES {
        let mut grid_index = 0;
        for i in 1..M {
            if distances[i - 1] > distances[i] {
                grid_indices[grid_index] = i as u32;
                *num_loops += 1;
                grid_index += 1;
            }
        }
        grid_indices[grid_index] = M as u32;
        *num_loops += 1;
        return;
    }

    let tail_start = M - ((M - 1) % LANES);
    let tail_size = M - tail_start;
    let has_tail = tail_size > 0;
    let tail_idx = M - LANES;
    let tail_offset = tail_start - tail_idx;
    let tail_bitmask = ((1 << tail_size) - 1) << (LANES - tail_size);

    let mut write_idx = 0usize;
    let indices_ptr = grid_indices.as_mut_ptr();

    let mut cur_index: usize = 1;
    while cur_index < M {
        let base_index = cur_index;
        let mut bits = 0u64;
        let mut bit_index = 0u32;

        unsafe {
            // Full chunk case.
            while cur_index < tail_start && bit_index < 64 {
                let cur = distances.load_simd_unchecked(cur_index);
                let prev = distances.load_simd_unchecked(cur_index - 1);
                let mask_bits = prev.simd_gt(cur).to_bits();
                bits |= mask_bits << bit_index;
                cur_index += LANES;
                bit_index += LANES as u32;
            }

            // Tail chunk case.
            if has_tail && bit_index < 64 && cur_index == tail_start {
                let cur = distances.load_simd_unchecked(tail_idx);
                let prev = distances.load_simd_unchecked(tail_idx - 1);
                let mask_bits = prev.simd_gt(cur).to_bits() & tail_bitmask;
                bits |= (mask_bits << bit_index) >> tail_offset;
                cur_index += LANES;
            }
        }

        // Convert bitmask into index values.
        while bits != 0 {
            let index = bits.trailing_zeros();
            unsafe { indices_ptr.add(write_idx).write(base_index as u32 + index) };
            write_idx += 1;
            bits &= bits - 1;
        }
    }

    // Write sentinel.
    unsafe { indices_ptr.add(write_idx).write(M as u32) };
    *num_loops = write_idx + 1;
}

#[inline(always)]
pub(crate) fn configure_tiling<T: BasicVec<Option<u32>>, F: ArithmeticVec<f32>>(
    tiling: &T,
    frequency: &F,
) -> T {
    // Adjust the tiling.
    let mut octave_tiling = *tiling;
    octave_tiling
        .as_mut_slice()
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| {
            if let Some(val) = x {
                let float = *val as f32 * frequency[i];
                let nearness = (float - float.round()).abs();
                assert!(
                    nearness < 0.001,
                    "Frequency does not align with the tiling!"
                );
                *val = (*val as f32 * frequency[i]) as u32;
            }
        });
    octave_tiling
}

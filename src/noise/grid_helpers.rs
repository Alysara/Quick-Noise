use std::mem::MaybeUninit;

use crate::math::vec::{ArithmeticVec, BasicVec};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;

const STACK_SIZE: usize = 4096;
pub struct ArenaCache {
    heap: Vec<f32>,
    stack: [MaybeUninit<f32>; STACK_SIZE],
}

impl ArenaCache {
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        let heap = if capacity > STACK_SIZE {
            Vec::with_capacity(capacity)
        } else {
            Vec::new()
        };

        let stack: [MaybeUninit<f32>; STACK_SIZE] = std::array::from_fn(|_| MaybeUninit::uninit());

        Self { heap, stack }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [MaybeUninit<f32>] {
        let slice = if self.heap.capacity() > 0 {
            self.heap.spare_capacity_mut()
        } else {
            self.stack.as_mut_slice()
        };

        let offset = slice.as_ptr().align_offset(64);
        &mut slice[offset..]
    }
}

pub struct Arena<'a> {
    slice: &'a mut [MaybeUninit<f32>],
}

impl<'a> Arena<'a> {
    #[inline(always)]
    pub fn with_cache(cache: &'a mut ArenaCache) -> Self {
        let slice = cache.as_mut_slice();
        Self { slice }
    }

    #[inline(always)]
    pub fn allocate<T>(&mut self, capacity: usize) -> &'a mut [MaybeUninit<T>] {
        const {
            assert!(size_of::<T>() == size_of::<f32>());
        }

        let whole = std::mem::take(&mut self.slice);

        let (buf, rem) = whole.split_at_mut(capacity);
        self.slice = rem;
        unsafe { std::mem::transmute(buf) }
    }
}

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

// #[inline(always)]
// pub(super) fn grid_fill_indices_slice(
//     grid_indices: &mut [u32],
//     distances: &[f32],
//     num_loops: &mut usize,
// ) {
//     const LANES: usize = ArchSimd::<f32>::LANES;
//     let len = grid_indices.len();
//
//     // Handle scalar case first.
//     // if len < LANES {
//     //     let mut grid_index = 0;
//     //     for i in 1..len {
//     //         if distances[i - 1] > distances[i] {
//     //             grid_indices[grid_index] = i as u32;
//     //             *num_loops += 1;
//     //             grid_index += 1;
//     //         }
//     //     }
//     //     grid_indices[grid_index] = len as u32;
//     //     *num_loops += 1;
//     //     return;
//     // }
//
//     let tail_start = len - ((len - 1) % LANES);
//     let tail_size = len - tail_start;
//     let has_tail = tail_size > 0;
//     let tail_idx = len - LANES;
//     let tail_offset = tail_start - tail_idx;
//     let tail_bitmask = ((1 << tail_size) - 1) << (LANES - tail_size);
//
//     let mut write_idx = 0usize;
//     let indices_ptr = grid_indices.as_mut_ptr();
//
//     let mut cur_index: usize = 1;
//     while cur_index < len {
//         let base_index = cur_index;
//         let mut bits = 0u64;
//         let mut bit_index = 0u32;
//
//         unsafe {
//             // Full chunk case.
//             while cur_index < tail_start && bit_index < 64 {
//                 let cur = ArchSimd::from_slice_unchecked(distances.get_unchecked(cur_index..));
//                 let prev = ArchSimd::from_slice_unchecked(distances.get_unchecked(cur_index - 1..));
//                 let mask_bits = prev.simd_gt(cur).to_bits();
//                 bits |= mask_bits << bit_index;
//                 cur_index += LANES;
//                 bit_index += LANES as u32;
//             }
//
//             // Tail chunk case.
//             if has_tail && bit_index < 64 && cur_index == tail_start {
//                 let cur = ArchSimd::from_slice_unchecked(distances.get_unchecked(tail_idx..));
//                 let prev = ArchSimd::from_slice_unchecked(distances.get_unchecked(tail_idx - 1..));
//                 let mask_bits = prev.simd_gt(cur).to_bits() & tail_bitmask;
//                 bits |= (mask_bits << bit_index) >> tail_offset;
//                 cur_index += LANES;
//             }
//         }
//
//         // Convert bitmask into index values.
//         while bits != 0 {
//             let index = bits.trailing_zeros();
//             unsafe { indices_ptr.add(write_idx).write(base_index as u32 + index) };
//             write_idx += 1;
//             bits &= bits - 1;
//         }
//     }
//
//     // Write sentinel.
//     unsafe { indices_ptr.add(write_idx).write(len as u32) };
//     *num_loops = write_idx + 1;
// }

#[inline(always)]
pub(super) fn grid_fill_indices_slice(
    grid_indices: &mut [MaybeUninit<u32>],
    distances: &[MaybeUninit<f32>],
    num_loops: &mut usize,
) {
    const LANES: usize = ArchSimd::<f32>::LANES;
    let len = grid_indices.len();
    let mut write_idx = 0usize;
    let indices_ptr = grid_indices.as_mut_ptr();

    let full_block_end = len - len % 64;
    for i in (1..=full_block_end).step_by(64) {
        let mut bits = 0u64;
        for bit_index in (0..64).step_by(LANES) {
            let cur_index = i + bit_index;
            let (cur, prev) = unsafe {
                (
                    ArchSimd::from_slice_unchecked(
                        distances.get_unchecked(cur_index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        distances.get_unchecked(cur_index - 1..).assume_init_ref(),
                    ),
                )
            };

            let mask_bits = prev.simd_gt(cur).to_bits();
            bits |= mask_bits << bit_index;
        }

        while bits != 0 {
            let cur_index = i as u32 + bits.trailing_zeros();
            unsafe {
                indices_ptr
                    .add(write_idx)
                    .write(MaybeUninit::new(cur_index))
            };
            write_idx += 1;
            bits &= bits - 1;
        }
    }

    let tail_len = len - full_block_end;
    let mut bits = 0u64;
    for bit_index in (0..tail_len).step_by(LANES) {
        let cur_index = bit_index + full_block_end + 1;
        let (cur, prev) = unsafe {
            (
                ArchSimd::from_slice_unchecked(
                    distances.get_unchecked(cur_index..).assume_init_ref(),
                ),
                ArchSimd::from_slice_unchecked(
                    distances.get_unchecked(cur_index - 1..).assume_init_ref(),
                ),
            )
        };

        let mask_bits = prev.simd_gt(cur).to_bits();
        bits |= mask_bits << bit_index;
    }
    bits &= (1u64 << tail_len) - 1;

    while bits != 0 {
        let cur_index = full_block_end as u32 + bits.trailing_zeros() + 1;
        unsafe {
            indices_ptr
                .add(write_idx)
                .write(MaybeUninit::new(cur_index))
        };
        write_idx += 1;
        bits &= bits - 1;
    }

    // Write sentinel.
    unsafe {
        indices_ptr
            .add(write_idx)
            .write(MaybeUninit::new(len as u32))
    };
    *num_loops = write_idx + 1;
}

/// Takes a list of value array pairs and broadcasts that value in
/// each array a specified amount of elements starting at a specified
/// index.
///
/// # Parameters
/// * `arrays` - an array of mutable [f32] slices
/// * 'values' - an array of values to set into each corresponding array
/// * `arrray_len` - the length of each [f32] slice
/// * 'index' - the index to start setting values at in all arrays
/// * 'amount' - the amount of elements to fill with the specified value
#[inline(always)]
pub fn multiset_slice<'a, const M: usize>(
    arrays: &mut [&mut &'a mut [MaybeUninit<f32>]; M],
    values: &[f32; M],
    mut index: usize,
    mut amount: isize,
) {
    let vecs: [ArchSimd<f32>; M] = std::array::from_fn(|i| ArchSimd::splat(values[i]));

    // if array_len < ArchSimd::<f32>::LANES {
    //     for i in 0..M {
    //         for n in 0..amount {
    //             unsafe {
    //                 *arrays[i].get_unchecked_mut(n as usize + index) = values[i];
    //             }
    //         }
    //     }
    //     return;
    // }

    while amount > 0 {
        // Full register store case.
        // if index < array_len.saturating_sub(ArchSimd::<f32>::LANES) {
        for i in 0..M {
            unsafe {
                let slice = arrays.get_unchecked_mut(i).get_unchecked_mut(index..);
                vecs.get_unchecked(i)
                    .copy_to_slice_unchecked(slice.assume_init_mut());
            }
        }
        // } else {
        // // Tail store case.
        // let iota = ArchSimd::<i32>::iota(array_len as i32 - ArchSimd::<f32>::LANES as i32);
        // let indices = ArchSimd::splat(index as i32);
        // let mask = iota.simd_ge(indices);
        // let tail_index = array_len.saturating_sub(ArchSimd::<f32>::LANES);
        //
        // for i in 0..M {
        //     unsafe {
        //         arrays.get_unchecked_mut(i).masked_store_simd(
        //             tail_index,
        //             *vecs.get_unchecked(i),
        //             mask.raw_cast(),
        //         );
        //     }
        // }
        // }

        amount -= ArchSimd::<f32>::LANES as isize;
        index += ArchSimd::<f32>::LANES;
    }
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

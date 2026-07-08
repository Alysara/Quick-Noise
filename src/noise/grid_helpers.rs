use std::array::from_fn;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

use crate::api::grid::interface::GridNoiseParams;
use crate::math::vec::{ArithmeticVec, BasicVec};
use crate::simd::arch_simd::{ArchFamily, ArchSimd, SIMD_WIDTH};
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;
use crate::simd::traits::SimdElement;

const LANES: usize = ArchSimd::<f32>::LANES;
const STACK_SIZE: usize = 8192;
pub(super) const CHUNK_SIZE: usize = 64;
pub(super) const PADDED_SIZE: usize = CHUNK_SIZE + LANES;

#[repr(align(64))]
pub struct AlignedBuffer<T>(pub [MaybeUninit<T>; PADDED_SIZE]);

impl<T> AlignedBuffer<T> {
    pub fn new() -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

impl<T: SimdElement> AlignedBuffer<T> {
    /// # Safety
    /// - Range `index..index + ArchSimd::<T>::LANES` must be in bounds for `self.0`
    ///   (`index <= CHUNK_SIZE`)
    /// - Data in the range `index..index + ArchSimd::<T>::LANES` is assumed initialized
    pub unsafe fn load_simd(&self, index: usize) -> ArchSimd<T> {
        unsafe { ArchSimd::from_slice_unchecked(self.0.get_unchecked(index..).assume_init_ref()) }
    }

    /// # Safety
    /// - Range `index..index + ArchSimd::<T>::LANES` must be in bounds for `self.0`
    ///   (`index <= CHUNK_SIZE`)
    /// - Data in the range `index..index + ArchSimd::<T>::LANES` is assumed initialized
    /// - Index must be aligned to `ArchSimd::<T>::LANES` (Multiple of `LANES`)
    pub unsafe fn load_simd_aligned(&self, index: usize) -> ArchSimd<T> {
        unsafe { ArchSimd::from_aligned_slice_unchecked(self.0.get_unchecked(index..).assume_init_ref()) }
    }

    /// # Safety
    /// - Range `index..index + ArchSimd::<T>::LANES` must be in bounds for `self.0`
    ///   (`index <= CHUNK_SIZE`)
    pub unsafe fn store_simd(&mut self, index: usize, simd: ArchSimd<T>) {
        unsafe {
            simd.copy_to_slice_unchecked(self.0.get_unchecked_mut(index..).assume_init_mut())
        };
    }

    /// # Safety
    /// - Range `index..index + ArchSimd::<T>::LANES` must be in bounds for `self.0`
    ///   (`index <= CHUNK_SIZE`)
    /// - Index must be aligned to `ArchSimd::<T>::LANES` (Multiple of `LANES`)
    pub unsafe fn store_simd_aligned(&mut self, index: usize, simd: ArchSimd<T>) {
        unsafe {
            simd.copy_to_aligned_slice_unchecked(self.0.get_unchecked_mut(index..).assume_init_mut())
        };
    }
}

impl<T> Deref for AlignedBuffer<T> {
    type Target = [MaybeUninit<T>; PADDED_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for AlignedBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for AlignedBuffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { self.0.assume_init_ref().fmt(f) }
    }
}

pub struct ArenaCache {
    heap: Vec<f32>,
    stack: [MaybeUninit<f32>; STACK_SIZE],
}

impl ArenaCache {
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity + ArchSimd::<f32>::LANES; // Add LANES for alignment padding.
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

        let offset = slice.as_ptr().align_offset(SIMD_WIDTH);
        unsafe { slice.get_unchecked_mut(offset..) }
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

    #[inline(always)]
    pub fn allocate_arena(&mut self, capacity: usize) -> Self {
        let whole = std::mem::take(&mut self.slice);

        let (slice, rem) = whole.split_at_mut(capacity);
        self.slice = rem;
        Self { slice }
    }
}

#[inline(always)]
pub(super) fn validate_grid_size<const D: usize>(grid_size: [usize; D], slice_len: usize) {
    let num_samples = grid_size.iter().product();
    assert!(
        slice_len >= num_samples,
        "Uniform grid with dimensions {:?} has a size of {num_samples}, which is more than the given slice length of {slice_len}",
        grid_size
    );
}

#[inline(always)]
pub(super) fn pad_grid_size<const D: usize>(grid_size: [usize; D]) -> [usize; D] {
    // from_fn(|i| {
    //     let rem = grid_size[i] % ArchSimd::<f32>::LANES;
    //     if rem == 0 {
    //         grid_size[i]
    //     } else {
    //         ArchSimd::<f32>::LANES - rem + grid_size[i]
    //     }
    // })

    const LANES: usize = ArchSimd::<f32>::LANES;
    from_fn(|i| LANES - grid_size[i] % LANES + grid_size[i])
}

// SAFETY: caller/invariant of this type guarantees these slices are
// fully initialized by the time Debug is used. If that's not
// guaranteed, this is unsound — see note below.
pub unsafe fn assume_init_slice<T>(s: &[MaybeUninit<T>]) -> &[T] {
    unsafe { std::slice::from_raw_parts(s.as_ptr().cast(), s.len()) }
}

// #[inline(always)]
// pub(super) fn grid_fill_indices<const M: usize>(
//     grid_indices: &mut AlignedBuffer<u32>,
//     distances: &AlignedBuffer<f32>,
//     num_loops: &mut usize,
// ) {
//     const LANES: usize = ArchSimd::<f32>::LANES;
//
//     // Handle scalar case first.
//     if M < LANES {
//         let mut grid_index = 0;
//         for i in 1..M {
//             if distances[i - 1] > distances[i] {
//                 grid_indices[grid_index] = i as u32;
//                 *num_loops += 1;
//                 grid_index += 1;
//             }
//         }
//         grid_indices[grid_index] = M as u32;
//         *num_loops += 1;
//         return;
//     }
//
//     let tail_start = M - ((M - 1) % LANES);
//     let tail_size = M - tail_start;
//     let has_tail = tail_size > 0;
//     let tail_idx = M - LANES;
//     let tail_offset = tail_start - tail_idx;
//     let tail_bitmask = ((1 << tail_size) - 1) << (LANES - tail_size);
//
//     let mut write_idx = 0usize;
//     let indices_ptr = grid_indices.as_mut_ptr();
//
//     let mut cur_index: usize = 1;
//     while cur_index < M {
//         let base_index = cur_index;
//         let mut bits = 0u64;
//         let mut bit_index = 0u32;
//
//         unsafe {
//             // Full chunk case.
//             while cur_index < tail_start && bit_index < 64 {
//                 let cur = distances.load_simd_unchecked(cur_index);
//                 let prev = distances.load_simd_unchecked(cur_index - 1);
//                 let mask_bits = prev.simd_gt(cur).to_bits();
//                 bits |= mask_bits << bit_index;
//                 cur_index += LANES;
//                 bit_index += LANES as u32;
//             }
//
//             // Tail chunk case.
//             if has_tail && bit_index < 64 && cur_index == tail_start {
//                 let cur = distances.load_simd_unchecked(tail_idx);
//                 let prev = distances.load_simd_unchecked(tail_idx - 1);
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
//     unsafe { indices_ptr.add(write_idx).write(M as u32) };
//     *num_loops = write_idx + 1;
// }

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
pub(super) fn grid_fill_indices(
    grid_indices: &mut AlignedBuffer<u32>,
    distances: &AlignedBuffer<f32>,
    distances_len: usize,
) -> usize {
    const LANES: usize = ArchSimd::<f32>::LANES;

    let mut write_idx = 0usize;
    let indices_ptr = grid_indices.as_mut_ptr();

    let last_valid = distances_len - 1;
    let full_block_end = last_valid - last_valid % 64;
    for base_index in (1..=full_block_end).step_by(64) {
        let mut bits = 0u64;
        for bit_index in (0..64).step_by(LANES) {
            let cur_index = base_index + bit_index;
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
            let cur_index = base_index as u32 + bits.trailing_zeros();
            unsafe {
                indices_ptr
                    .add(write_idx)
                    .write(MaybeUninit::new(cur_index))
            };
            write_idx += 1;
            bits &= bits - 1;
        }
    }

    let tail_len = last_valid - full_block_end;
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
            .write(MaybeUninit::new(distances_len as u32))
    };
    write_idx + 1
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
pub(crate) fn configure_tiling<const D: usize>(params: &GridNoiseParams<D>) -> [Option<u32>; D] {
    std::array::from_fn(|i| {
        if let Some(val) = params.tiling[i] {
            let float = val as f32 * params.frequency[i];
            let nearness = (float - float.round()).abs();
            assert!(
                nearness < 0.001,
                "Frequency does not align with the tiling!"
            );
            Some(float as u32)
        } else {
            None
        }
    })
}

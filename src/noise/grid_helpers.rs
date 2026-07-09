use std::array::from_fn;
use std::mem::MaybeUninit;

use crate::api::grid::interface::GridNoiseParams;
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG, SIMD_WIDTH};
use crate::simd::traits::SimdElement;

const STACK_SIZE: usize = 8192;
pub struct ArenaBuffer {
    heap: Vec<f32>,
    stack: [MaybeUninit<f32>; STACK_SIZE],
}

impl ArenaBuffer {
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
    pub fn with_cache(cache: &'a mut ArenaBuffer) -> Self {
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

pub const NUM_BLOCKS: usize = NUM_SIMD_REG / 8;
pub const LANES: usize = ArchSimd::<f32>::LANES;
pub const BLOCK_LANES: usize = NUM_BLOCKS * LANES;

pub struct InterpolationConfig {
    pub has_block_head: bool,
    pub has_block_tail: bool,
    pub block_tail_size: usize,
    pub block_tail_start: usize,
}

impl InterpolationConfig {
    pub fn new(x_dim: usize) -> Self {
        Self {
            has_block_head: x_dim >= BLOCK_LANES,
            has_block_tail: !x_dim.is_multiple_of(BLOCK_LANES),
            block_tail_size: (x_dim % BLOCK_LANES).div_ceil(LANES),
            block_tail_start: (x_dim / BLOCK_LANES) * BLOCK_LANES,
        }
    }
}


pub trait MaybeUninitSliceSimdExt<T: SimdElement> {
    /// # Safety
    /// - The range `index..index + ArchSimd::<T>::LANES` must be in bounds.
    /// - Data in range `index..index + ArchSimd::<T>::LANES` must be initialized.
    unsafe fn load_simd(&self, index: usize) -> ArchSimd<T>;

    /// # Safety
    /// - The range `index..index + ArchSimd::<T>::LANES` must be in bounds.
    /// - Data in range `index..index + ArchSimd::<T>::LANES` must be initialized.
    /// - `index` must be aligned according to `SIMD_WIDTH`.
    unsafe fn load_simd_aligned(&self, index: usize) -> ArchSimd<T>;

    /// # Safety
    /// - The range `index..index + ArchSimd::<T>::LANES` must be in bounds.
    unsafe fn write_simd(&mut self, index: usize, simd: ArchSimd<T>);

    /// # Safety
    /// - The range `index..index + ArchSimd::<T>::LANES` must be in bounds.
    /// - `index` must be aligned according to `SIMD_WIDTH`.
    unsafe fn write_simd_aligned(&mut self, index: usize, simd: ArchSimd<T>);
}

impl<T: SimdElement> MaybeUninitSliceSimdExt<T> for [MaybeUninit<T>] {
    unsafe fn load_simd(&self, index: usize) -> ArchSimd<T> {
        unsafe { ArchSimd::from_slice_unchecked(self.get_unchecked(index..).assume_init_ref()) }
    }

    unsafe fn load_simd_aligned(&self, index: usize) -> ArchSimd<T> {
        unsafe {
            ArchSimd::from_aligned_slice_unchecked(self.get_unchecked(index..).assume_init_ref())
        }
    }

    unsafe fn write_simd(&mut self, index: usize, simd: ArchSimd<T>) {
        unsafe { simd.copy_to_slice_unchecked(self.get_unchecked_mut(index..).assume_init_mut()) }
    }

    unsafe fn write_simd_aligned(&mut self, index: usize, simd: ArchSimd<T>) {
        unsafe { simd.copy_to_aligned_slice_unchecked(self.get_unchecked_mut(index..).assume_init_mut()) }
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
    const LANES: usize = ArchSimd::<f32>::LANES;
    from_fn(|i| LANES - grid_size[i] % LANES + grid_size[i] + LANES)
}

// SAFETY: caller/invariant of this type guarantees these slices are
// fully initialized by the time Debug is used. If that's not
// guaranteed, this is unsound — see note below.
pub unsafe fn assume_init_slice<T>(s: &[MaybeUninit<T>]) -> &[T] {
    unsafe { std::slice::from_raw_parts(s.as_ptr().cast(), s.len()) }
}

#[inline(always)]
pub(super) fn grid_fill_indices_slice<const D: usize>(
    grid_indices: &mut [&mut [MaybeUninit<u32>]; D],
    distances: &[&mut [MaybeUninit<f32>]; D],
    distances_len: [usize; D],
) -> [usize; D] {
    const LANES: usize = ArchSimd::<f32>::LANES;

    std::array::from_fn(|i| {
        let mut write_idx = 0usize;
        let indices_ptr = grid_indices[i].as_mut_ptr();

        let last_valid = distances_len[i] - 1;
        let full_block_end = last_valid - last_valid % 64;
        for base_index in (1..=full_block_end).step_by(64) {
            let mut bits = 0u64;
            for bit_index in (0..64).step_by(LANES) {
                let cur_index = base_index + bit_index;
                let (cur, prev) = unsafe {
                    (
                        ArchSimd::from_slice_unchecked(
                            distances[i].get_unchecked(cur_index..).assume_init_ref(),
                        ),
                        ArchSimd::from_aligned_slice_unchecked(
                            distances[i]
                                .get_unchecked(cur_index - 1..)
                                .assume_init_ref(),
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
                        distances[i].get_unchecked(cur_index..).assume_init_ref(),
                    ),
                    ArchSimd::from_aligned_slice_unchecked(
                        distances[i]
                            .get_unchecked(cur_index - 1..)
                            .assume_init_ref(),
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
                .write(MaybeUninit::new(distances_len[i] as u32))
        };
        write_idx + 1
    })
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

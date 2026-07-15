use std::iter::zip;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::simd::arch_simd::{ArchFamily, ArchSimd};
use crate::simd::architectures::interface::*;
use crate::simd::array_trait::Array;
use crate::simd::register::Simd;
use crate::simd::traits::*;

pub trait SimdSliceIterExt<T: SimdElement> {
    fn simd_iter<'a>(&'a self) -> SimdSliceIter<'a, T, ArchFamily>;
    fn simd_iter_mut<'a>(&'a mut self) -> SimdSliceIterMut<'a, T, ArchFamily>;
}

impl<T: SimdElement> SimdSliceIterExt<T> for [T] {
    /// Creates an iterator of simd chunks.
    fn simd_iter<'a>(&'a self) -> SimdSliceIter<'a, T, ArchFamily> {
        SimdSliceIter {
            slice: self,
            index: 0,
            _architecture: PhantomData::<ArchFamily>,
        }
    }

    /// Creates an iterator of mutable simd chunks.
    fn simd_iter_mut<'a>(&'a mut self) -> SimdSliceIterMut<'a, T, ArchFamily> {
        SimdSliceIterMut {
            slice: self,
            _architecture: PhantomData::<ArchFamily>,
        }
    }
}

pub struct SimdSliceIter<'a, T: SimdElement, F: SimdFamily> {
    slice: &'a [T],
    index: usize,
    _architecture: PhantomData<F>,
}

impl<'a, T: SimdElement, F: SimdFamily> Iterator for SimdSliceIter<'a, T, F> {
    type Item = Simd<T, F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.slice.len() {
            return None;
        }

        // Scalar case.
        if self.slice.len() < Self::Item::LANES {
            let mut array = Self::Item::zero().to_array();
            for i in 0..self.slice.len() {
                array[i] = self.slice[i];
            }
            let result = Self::Item::from_slice(array.as_mut_slice());
            self.index = self.slice.len();
            return Some(result);
        }

        let amount_left = self.slice.len() - self.index;
        if amount_left < Self::Item::LANES {
            let offset = Self::Item::LANES - (self.slice.len() - self.index);
            let new_index = self.index - offset;
            let simd = unsafe { Self::Item::from_slice(self.slice.get_unchecked(new_index..)) };
            let simd_shifted = simd.left_lane_shift(offset as u32);
            self.index = self.slice.len();
            return Some(simd_shifted);
        }

        // Regular case.
        let next =
            unsafe { Self::Item::from_slice_unchecked(self.slice.get_unchecked(self.index..)) };
        self.index += Self::Item::LANES;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let amount_left = self.slice.len() - self.index;
        let rem_chunks = amount_left.div_ceil(Self::Item::LANES);
        (rem_chunks, Some(rem_chunks))
    }
}
impl<'a, T: SimdElement, F: SimdFamily> ExactSizeIterator for SimdSliceIter<'a, T, F> {}

pub struct SimdSliceChunk<'a, T: SimdElement, F: SimdFamily> {
    simd: Simd<T, F>,
    slice: &'a mut [T],
}

impl<'a, T: SimdElement, F: SimdFamily> SimdSliceChunk<'a, T, F> {
    pub fn new(simd: Simd<T, F>, slice: &'a mut [T]) -> Self {
        Self { simd, slice }
    }
}

impl<'a, T: SimdElement, F: SimdFamily> Deref for SimdSliceChunk<'a, T, F> {
    type Target = Simd<T, F>;
    fn deref(&self) -> &Self::Target {
        &self.simd
    }
}

impl<'a, T: SimdElement, F: SimdFamily> DerefMut for SimdSliceChunk<'a, T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.simd
    }
}

impl<'a, T: SimdElement, F: SimdFamily> Drop for SimdSliceChunk<'a, T, F> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.slice.len() >= Simd::<T, F>::LANES {
            // Regular case.
            unsafe { self.simd.copy_to_slice_unchecked(self.slice) };
        } else {
            // Tail/Partial case.
            let array = self.simd.to_array();
            self.slice
                .iter_mut()
                .zip(array.iter())
                .for_each(|(src, new)| *src = *new);
        }
    }
}

pub struct SimdSliceIterMut<'a, T: SimdElement, F: SimdFamily> {
    slice: &'a mut [T],
    _architecture: PhantomData<F>,
}

impl<'a, T: SimdElement, F: SimdFamily> Iterator for SimdSliceIterMut<'a, T, F> {
    type Item = SimdSliceChunk<'a, T, F>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Empty case.
        let slice_len = self.slice.len();
        if self.slice.is_empty() {
            return None;
        }
        let slice = std::mem::take(&mut self.slice);

        // Scalar + Tail case.
        if slice_len < Simd::<T, F>::LANES {
            let mut array = Simd::<T, F>::zero().to_array();
            for i in 0..slice_len {
                array[i] = slice[i];
            }
            let simd = unsafe { Simd::<T, F>::from_slice_unchecked(array.as_mut_slice()) };
            let chunk = SimdSliceChunk::new(simd, slice);
            return Some(chunk);

            // let next_simd = unsafe { Simd::from_slice_partial(slice) };
            // let chunk = SimdSliceChunk::new(next_simd, slice);
            // return Some(chunk);
        }

        // Tail case.
        // if slice_len < Simd::<T, F>::LANES {
        //     let offset = Simd::<T, F>::LANES - slice_len;
        //     let new_index = slice_len - offset;
        //     let simd =
        //         Simd::<T, F>::from_slice(&self.slice[new_index..]);
        //     let shifted_simd = simd.left_lane_shift(offset as u32)
        //     let chunk = SimdSliceChunk::new(shifted_result, &mut slice[new_index..new_index + slice_len]);
        //     self.index = slice_len;
        //     return Some(chunk);
        // }

        // Regular case.
        let (cur_slice, rem_slice) = slice.split_at_mut(Simd::<T, F>::LANES);
        self.slice = rem_slice;

        let next_simd = unsafe { Simd::from_slice_unchecked(cur_slice) };
        let chunk = SimdSliceChunk::new(next_simd, cur_slice);
        Some(chunk)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem_chunks = self.slice.len().div_ceil(Simd::<T, F>::LANES);
        (rem_chunks, Some(rem_chunks))
    }
}
impl<'a, T: SimdElement, F: SimdFamily> ExactSizeIterator for SimdSliceIterMut<'a, T, F> {}

// Vec into iter

pub trait IntoSimdIterator<T: SimdElement> {
    fn into_simd_iter(self) -> SimdVecIntoIter<T, ArchFamily>;
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
pub struct SimdVecIntoIter<T: SimdElement, F: SimdFamily> {
    vec: Vec<T>,
    index: usize,
    _architecture: PhantomData<F>,
}

impl<T: SimdElement, F: SimdFamily> Iterator for SimdVecIntoIter<T, F> {
    type Item = Simd<T, F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.len() {
            return None;
        }

        // Scalar case.
        if self.vec.len() < Self::Item::LANES {
            let mut array = Self::Item::zero().to_array();
            for i in 0..self.vec.len() {
                array[i] = self.vec[i];
            }
            let result = Self::Item::from_slice(array.as_mut_slice());
            self.index = self.vec.len();
            return Some(result);
        }

        let amount_left = self.vec.len() - self.index;
        if amount_left < Self::Item::LANES {
            let offset = Self::Item::LANES - (self.vec.len() - self.index);
            let new_index = self.index - offset;
            let simd = unsafe { Self::Item::from_slice(self.vec.get_unchecked(new_index..)) };
            let simd_shifted = simd.left_lane_shift(offset as u32);
            self.index = self.vec.len();
            return Some(simd_shifted);
        }

        // Regular case.
        let next =
            unsafe { Self::Item::from_slice_unchecked(self.vec.get_unchecked(self.index..)) };
        self.index += Self::Item::LANES;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let amount_left = self.vec.len() - self.index;
        let rem_chunks = amount_left.div_ceil(Self::Item::LANES);
        (rem_chunks, Some(rem_chunks))
    }
}
impl<T: SimdElement, F: SimdFamily> ExactSizeIterator for SimdVecIntoIter<T, F> {}

impl<T: SimdElement, F: SimdFamily, const N: usize> FromIterator<Simd<T, F>> for [T; N] {
    fn from_iter<I: IntoIterator<Item = Simd<T, F>>>(iter: I) -> Self {
        let mut array = [T::default(); N];

        let lane_iter = (0..N).step_by(Simd::<T, F>::LANES);
        for (i, x) in zip(lane_iter, iter) {
            x.copy_to_slice(&mut array[i..]);
        }

        array
    }
}

impl<T: SimdElement, F: SimdFamily> FromIterator<Simd<T, F>> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = Simd<T, F>>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        if let Some(upper_bound) = upper_bound {
            let mut vec = vec![T::default(); upper_bound * ArchSimd::<T>::LANES];
            for (i, x) in iter.enumerate() {
                x.copy_to_slice(&mut vec[i * ArchSimd::<T>::LANES..]);
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

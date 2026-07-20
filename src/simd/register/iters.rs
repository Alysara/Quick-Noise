use std::iter::zip;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::simd::StaticArch;
use crate::simd::architectures::interface::*;
use crate::simd::array_trait::Array;
use crate::simd::register::Simd;
use crate::simd::traits::*;

pub trait SimdSliceIterExt<T: SimdElement> {
    fn simd_iter<'a>(&'a self) -> SimdSliceIter<'a, T, StaticArch>;
    fn simd_iter_mut<'a>(&'a mut self) -> SimdSliceIterMut<'a, T, StaticArch>;
}

impl<T: SimdElement> SimdSliceIterExt<T> for [T] {
    /// Creates an iterator of simd chunks.
    fn simd_iter<'a>(&'a self) -> SimdSliceIter<'a, T, StaticArch> {
        SimdSliceIter {
            slice: self,
            index: 0,
            _architecture: PhantomData::<StaticArch>,
        }
    }

    /// Creates an iterator of mutable simd chunks.
    fn simd_iter_mut<'a>(&'a mut self) -> SimdSliceIterMut<'a, T, StaticArch> {
        SimdSliceIterMut {
            slice: self,
            _architecture: PhantomData::<StaticArch>,
        }
    }
}

pub struct SimdSliceIter<'a, T: SimdElement, A: Arch> {
    slice: &'a [T],
    index: usize,
    _architecture: PhantomData<A>,
}

impl<'a, T: SimdElement, A: Arch> Iterator for SimdSliceIter<'a, T, A> {
    type Item = Simd<T, A>;

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
impl<'a, T: SimdElement, A: Arch> ExactSizeIterator for SimdSliceIter<'a, T, A> {}

pub struct SimdSliceChunk<'a, T: SimdElement, A: Arch> {
    simd: Simd<T, A>,
    slice: &'a mut [T],
}

impl<'a, T: SimdElement, A: Arch> SimdSliceChunk<'a, T, A> {
    pub fn new(simd: Simd<T, A>, slice: &'a mut [T]) -> Self {
        Self { simd, slice }
    }
}

impl<'a, T: SimdElement, A: Arch> Deref for SimdSliceChunk<'a, T, A> {
    type Target = Simd<T, A>;
    fn deref(&self) -> &Self::Target {
        &self.simd
    }
}

impl<'a, T: SimdElement, A: Arch> DerefMut for SimdSliceChunk<'a, T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.simd
    }
}

impl<'a, T: SimdElement, A: Arch> Drop for SimdSliceChunk<'a, T, A> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.slice.len() >= Simd::<T, A>::LANES {
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

pub struct SimdSliceIterMut<'a, T: SimdElement, A: Arch> {
    slice: &'a mut [T],
    _architecture: PhantomData<A>,
}

impl<'a, T: SimdElement, A: Arch> Iterator for SimdSliceIterMut<'a, T, A> {
    type Item = SimdSliceChunk<'a, T, A>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Empty case.
        let slice_len = self.slice.len();
        if self.slice.is_empty() {
            return None;
        }
        let slice = std::mem::take(&mut self.slice);

        // Scalar + Tail case.
        if slice_len < Simd::<T, A>::LANES {
            let mut array = Simd::<T, A>::zero().to_array();
            for i in 0..slice_len {
                array[i] = slice[i];
            }
            let simd = unsafe { Simd::<T, A>::from_slice_unchecked(array.as_mut_slice()) };
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
        let (cur_slice, rem_slice) = slice.split_at_mut(Simd::<T, A>::LANES);
        self.slice = rem_slice;

        let next_simd = unsafe { Simd::from_slice_unchecked(cur_slice) };
        let chunk = SimdSliceChunk::new(next_simd, cur_slice);
        Some(chunk)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem_chunks = self.slice.len().div_ceil(Simd::<T, A>::LANES);
        (rem_chunks, Some(rem_chunks))
    }
}
impl<'a, T: SimdElement, A: Arch> ExactSizeIterator for SimdSliceIterMut<'a, T, A> {}

// Vec into iter

pub trait IntoSimdIterator<T: SimdElement, A: Arch = StaticArch> {
    fn into_simd_iter(self) -> SimdVecIntoIter<T, A>;
}

impl<T: SimdElement, A: Arch> IntoSimdIterator<T, A> for Vec<T> {
    fn into_simd_iter(self) -> SimdVecIntoIter<T, A> {
        SimdVecIntoIter {
            vec: self,
            index: 0,
            _architecture: PhantomData::<A>,
        }
    }
}
pub struct SimdVecIntoIter<T: SimdElement, A: Arch> {
    vec: Vec<T>,
    index: usize,
    _architecture: PhantomData<A>,
}

impl<T: SimdElement, A: Arch> Iterator for SimdVecIntoIter<T, A> {
    type Item = Simd<T, A>;

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
impl<T: SimdElement, A: Arch> ExactSizeIterator for SimdVecIntoIter<T, A> {}

impl<T: SimdElement, A: Arch, const N: usize> FromIterator<Simd<T, A>> for [T; N] {
    fn from_iter<I: IntoIterator<Item = Simd<T, A>>>(iter: I) -> Self {
        let mut array = [T::default(); N];

        let lane_iter = (0..N).step_by(Simd::<T, A>::LANES);
        for (i, x) in zip(lane_iter, iter) {
            x.copy_to_slice(&mut array[i..]);
        }

        array
    }
}

impl<T: SimdElement, A: Arch> FromIterator<Simd<T, A>> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = Simd<T, A>>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        if let Some(upper_bound) = upper_bound {
            let mut vec = vec![T::default(); upper_bound * Simd::<T, A>::LANES];
            for (i, x) in iter.enumerate() {
                x.copy_to_slice(&mut vec[i * Simd::<T, A>::LANES..]);
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

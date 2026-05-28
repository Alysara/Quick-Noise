use crate::simd::architectures::arch_impl::*;
use crate::simd::traits::*;
use std::marker::PhantomData;
use crate::simd::simd_traits::*;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Simd<T: SimdElement, F: SimdFamily> {
    pub(crate) data: F::Vec,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    pub const SIMD_WIDTH: usize = F::SIMD_WIDTH;
    pub const LANE_SIZE: usize = std::mem::size_of::<T>();
    pub const LANES: usize = F::SIMD_WIDTH / Self::LANE_SIZE;
}

impl<T: SimdElement, F: SimdFamily> SimdContext for Simd<T, F> {
    type Element = T;
    type Family = F;
}
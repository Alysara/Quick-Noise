use std::marker::PhantomData;

use crate::simd::architectures::interface::*;
use crate::simd::traits::*;

pub mod element;
pub mod integer;
pub mod float;
pub mod iters;

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

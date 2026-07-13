use std::marker::PhantomData;

use crate::simd::architectures::interface::SimdFamily;

pub mod element;

#[derive(Clone, Copy)]
pub struct Mask<T, F: SimdFamily> {
    pub(crate) data: F::Mask,
    pub(crate) _marker: PhantomData<T>,
}

use std::marker::PhantomData;

use crate::simd::architectures::interface::Arch;

pub mod element;

#[derive(Clone, Copy)]
pub struct Mask<T, F: Arch> {
    pub(crate) data: F::Mask,
    pub(crate) _marker: PhantomData<T>,
}

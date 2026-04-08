use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign, Neg};
use num_traits::NumCast;
use num_traits::float::Float;
use crate::simd::simd_array::fmt::Debug;
use crate::simd::arch_simd::{ArchSimd, ArchMask};
use crate::simd::traits::{SimdFloat, SimdElement, SimdInteger};
use crate::simd::simd_traits::*;

// === Macro Helpers ===

macro_rules! impl_simd_array_op {
    ($trait:ident, $assign_trait:ident, $method:ident, $assign_method:ident, $op:tt, $assign_op:tt) => {
        impl<T: SimdElement, const N: usize> $trait for SimdArray<T, N>
        where
            ArchSimd<T>:,
            T: $trait<Output = T>,
            ArchSimd<T>: $trait<Output = ArchSimd<T>>,
        {
            type Output = SimdArray<T, N>;
            fn $method(self, other: SimdArray<T, N>) -> SimdArray<T, N> {
                let mut result = SimdArray::<T, N>::new_uninit();
                for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
                    let caller_vec = self.load_simd(i);
                    let other_vec = other.load_simd(i);
                    result.store_simd(i, caller_vec $op other_vec);
                }
                if Self::HAS_TAIL {
                    let caller_vec = self.load_simd(Self::TAIL_START);
                    let other_vec = other.load_simd(Self::TAIL_START);
                    result.partial_store_simd(Self::TAIL_START, caller_vec $op other_vec, Self::TAIL_SIZE);
                }
                result
            }
        }

        impl<T: SimdElement, const N: usize> $assign_trait for SimdArray<T, N>
        where
            ArchSimd<T>:,
            T: $assign_trait,
            ArchSimd<T>: $assign_trait,
        {
            fn $assign_method(&mut self, other: SimdArray<T, N>) {
                for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
                    let mut caller_vec = self.load_simd(i);
                    caller_vec $assign_op other.load_simd(i);
                    self.store_simd(i, caller_vec);
                }
                if Self::HAS_TAIL {
                    let mut caller_vec = self.load_simd(Self::TAIL_START);
                    caller_vec $assign_op other.load_simd(Self::TAIL_START);
                    self.partial_store_simd(Self::TAIL_START, caller_vec, Self::TAIL_SIZE);
                }
            }
        }
    }
}

#[repr(align(64))]
#[derive(Copy)]
pub struct SimdArray<T: SimdElement, const N: usize> {
    pub data: [MaybeUninit<T>; N],
}

// === Tail Info ===

pub trait TailInfo {
    const TAIL_SIZE: usize;
    const TAIL_START: usize;
    const HAS_TAIL: bool;
}

impl<T: SimdElement, const N: usize> TailInfo for SimdArray<T, N> {
    const TAIL_SIZE: usize = N % ArchSimd::<T>::LANES;
    const TAIL_START: usize = N - Self::TAIL_SIZE;
    const HAS_TAIL: bool = Self::TAIL_SIZE > 0;
}

// === Constructors ===

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    pub fn new_uninit() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    pub fn new(value: T) -> Self {
        Self {
            data: [MaybeUninit::new(value); N],
        }
    }
}

impl<T: SimdElement, const N: usize> Default for SimdArray<T, N> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

// === Indexing ===

impl<T: SimdElement, const N: usize> Index<usize> for SimdArray<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < N);
        unsafe { &self.data[index].assume_init_ref() }
    }
}

impl<T: SimdElement, const N: usize> IndexMut<usize> for SimdArray<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < N);
        unsafe { self.data[index].assume_init_mut() }
    }
}

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> T {
        debug_assert!(index < N);
        unsafe { *self.data.get_unchecked(index).assume_init_ref() }
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < N);
        unsafe { self.data.get_unchecked_mut(index).assume_init_mut() }
    }
}

// === Utility Traits ===

impl<T: SimdElement + Copy, const N: usize> Clone for SimdArray<T, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: SimdElement + fmt::Debug, const N: usize> fmt::Debug for SimdArray<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(unsafe { self.data.assume_init_ref() })
            .finish()
    }
}

// === Simd Access ===

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    #[inline(always)]
    pub fn load_simd(&self, index: usize) -> ArchSimd<T> {
        debug_assert!(index + ArchSimd::<T>::LANES <= N);
        // debug_assert!(index % ArchSimd::<T>::LANES == 0);
        unsafe { ArchSimd::load(&self.data.assume_init_ref().get_unchecked(index..)) }
    }

    #[inline(always)]
    pub fn store_simd(&mut self, index: usize, vec: ArchSimd<T>) {
        debug_assert!(index + ArchSimd::<T>::LANES <= N);
        // debug_assert!(index % ArchSimd::<T>::LANES == 0);
        unsafe { vec.store(&mut self.data.assume_init_mut().get_unchecked_mut(index..)); }
    }

    #[inline(always)]
    pub fn partial_store_simd(&mut self, index: usize, vec: ArchSimd<T>, amount: usize) {
        debug_assert!(index + amount <= N);
        unsafe { vec.partial_store(&mut self.data.assume_init_mut().get_unchecked_mut(index..), amount); }
    }

    #[inline(always)]
    pub fn masked_store_simd(&mut self, index: usize, vec: ArchSimd<T>, mask: ArchMask<T>) {
        debug_assert!(index < N);
        unsafe { vec.masked_store(&mut self.data.assume_init_mut().get_unchecked_mut(index..), mask); }
    }
}

// === Basic Operator Imeplementations ===

impl_simd_array_op!(Add, AddAssign, add, add_assign, +, +=);
impl_simd_array_op!(Sub, SubAssign, sub, sub_assign, -, -=);
impl_simd_array_op!(Mul, MulAssign, mul, mul_assign, *, *=);
impl_simd_array_op!(Div, DivAssign, div, div_assign, /, /=);

impl<T: SimdElement, const N: usize> Neg for SimdArray<T, N> {
    type Output = SimdArray<T, N>;
    fn neg(self) -> SimdArray<T, N> {
        let mut result = SimdArray::<T, N>::new_uninit();
        for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            let data = self.load_simd(i);
            result.store_simd(i, -data);
        }
        if Self::HAS_TAIL {
            let data = self.load_simd(Self::TAIL_START);
            result.partial_store_simd(Self::TAIL_START, -data, Self::TAIL_SIZE);
        }
        result
    }
}

// === Additional Operations ===

// impl<T: SimdElement, const N: usize> SimdArray<T, N> {
//     #[inline(always)]
//     pub fn multiset_many<const M: usize>(arrays: &mut [&mut Self; M], values: &[T; M], mut index: usize, mut amount: isize) {
//         let vecs: [ArchSimd<T>; M] = std::array::from_fn(|i| ArchSimd::<T>::splat(values[i]));

//         let indices = ArchSimd::load(&std::array::from_fn::<i32, N, _>(|i| i as i32));
        
//         // TODO: Make masks work natively different types.
//         while amount > 0 {
//             let amounts = ArchSimd::splat(amount as i32);
//             let mask = amounts.simd_gt(indices);

//             // println!("amounts: {:?}", amounts);
//             // println!("indices: {:?}", indices);

//             for i in 0..M {
//                 debug_assert!(i < M);
//                 unsafe { arrays[i].masked_store_simd(index, *vecs.get_unchecked(i), mask.raw_cast()); }
//             }
//             amount -= ArchSimd::<T>::LANES as isize;
//             index += ArchSimd::<T>::LANES;
//         }
//     }
// }

// impl<T: SimdElement, const N: usize> SimdArray<T, N> {
//     #[inline(always)]
//     pub fn multiset_many<const M: usize>(arrays: &mut [&mut Self; M], values: &[T; M], mut index: usize, mut amount: isize) {
//         let vecs: [ArchSimd<T>; M] = std::array::from_fn(|i| ArchSimd::<T>::splat(values[i]));

//         let indices = ArchSimd::load(&std::array::from_fn::<i32, N, _>(|i| i as i32));
        
//         // TODO: Make masks work natively different types.
//         while amount > 0 {
//             if amount >= ArchSimd::<T>::LANES as isize {
//                 for i in 0..M {
//                     unsafe { arrays[i].store_simd(index, *vecs.get_unchecked(i)); }
//                 }
//             } else {
//                 let amounts = ArchSimd::splat(amount as i32);
//                 let mask = amounts.simd_gt(indices);

//                 // println!("amounts: {:?}", amounts);
//                 // println!("indices: {:?}", indices);

//                 for i in 0..M {
//                     debug_assert!(i < M);
//                     unsafe { arrays[i].masked_store_simd(index, *vecs.get_unchecked(i), mask.raw_cast()); }
//                 }
//             }
//             amount -= ArchSimd::<T>::LANES as isize;
//             index += ArchSimd::<T>::LANES;
//         }
//     }
// }

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    #[inline(always)]
    pub fn multiset_many<const M: usize>(arrays: &mut [&mut Self; M], values: &[T; M], mut index: usize, mut amount: isize) {
        let vecs: [ArchSimd<T>; M] = std::array::from_fn(|i| ArchSimd::<T>::splat(values[i]));

        while amount > 0 {
            if index < N - ArchSimd::<T>::LANES {
                for i in 0..M {
                    unsafe { arrays[i].store_simd(index, *vecs.get_unchecked(i)); }
                }
            } else {
                let iota = ArchSimd::<i32>::iota(N as i32 - ArchSimd::<T>::LANES as i32);
                let indices = ArchSimd::splat(index as i32);
                let mask = iota.simd_ge(indices);
                let tail_index = N - ArchSimd::<T>::LANES;

                for i in 0..M {
                    unsafe { arrays[i].masked_store_simd(tail_index, *vecs.get_unchecked(i), mask.raw_cast()); }
                }
            }

            amount -= ArchSimd::<T>::LANES as isize;
            index += ArchSimd::<T>::LANES;
        }
    }
}

// impl<T: SimdElement, const N: usize> SimdArray<T, N> {
//     pub fn multiset_many<const M: usize>(arrays: &mut [&mut Self; M], values: &[T; M], mut index: usize, mut amount: isize) {
//         let vecs: [ArchSimd<T>; M] = std::array::from_fn(|i| ArchSimd::<T>::splat(values[i]));

//         let indices = ArchSimd::load(&std::array::from_fn::<T, N, _>(|i| <T as NumCast>::from(i).unwrap()));
        
//         while amount > 0 {
//             let amounts = ArchSimd::splat(<T as NumCast>::from(amount).unwrap());
//             let mask = indices.simd_lt(amounts);
//             for i in 0..M {
//                 debug_assert!(i < M);
//                 unsafe { arrays[i].masked_store_simd(index, *vecs.get_unchecked(i), mask); }
//             }
//             amount -= ArchSimd::<T>::LANES as isize;
//             index += ArchSimd::<T>::LANES;
//         }
//     }
// }


impl<T: SimdElement + std::default::Default, const N: usize> SimdArray<T, N> {
    // No easily built-in std::simd solution, let compiler auto-vectorize.
    // #[inline(never)]
    pub fn load_gather<const M: usize>(&mut self, load_index: usize, source_array: &[T; M], indicies: ArchSimd<u32>) {
        let indicies_array = indicies.to_array();
        for i in 0..ArchSimd::<T>::LANES {
            unsafe {
                *self.get_unchecked_mut(load_index + i) = *source_array.get_unchecked(indicies_array[i] as usize);
            }
        }
    }
}

impl<T: SimdElement + NumCast + Debug, const N: usize> SimdArray<T, N>
where
    T: Mul<Output = T>,
    ArchSimd<T>: Mul<Output = ArchSimd<T>>,
{
    pub fn iota_custom(offset: T, increment: T) -> Self {
        let mut result = Self::new_uninit();

        let increment_vec = ArchSimd::splat(increment);
        let lanes_increment_vec = ArchSimd::splat(increment *  NumCast::from(ArchSimd::<T>::LANES).unwrap());
        let iota_vec = ArchSimd::iota(NumCast::from(0).unwrap()) * increment_vec;
        
        let mut cur_vec = ArchSimd::splat(offset) + iota_vec;
        
        result.store_simd(0, cur_vec);
        for i in (ArchSimd::<T>::LANES..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            cur_vec += lanes_increment_vec;
            result.store_simd(i, cur_vec);
        }

        if Self::HAS_TAIL {
            cur_vec += lanes_increment_vec;
            result.partial_store_simd(Self::TAIL_START, cur_vec, Self::TAIL_SIZE);
        }

        result
    }

    pub fn iota(offset: T) -> Self {
        let mut result = Self::new_uninit();

        let lanes_increment_vec = ArchSimd::splat(NumCast::from(ArchSimd::<T>::LANES).unwrap());
        let mut cur_vec = ArchSimd::iota(offset);
        
        result.store_simd(0, cur_vec);
        for i in (ArchSimd::<T>::LANES..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            cur_vec += lanes_increment_vec;
            result.store_simd(i, cur_vec);
        }

        if Self::HAS_TAIL {
            cur_vec += lanes_increment_vec;
            result.partial_store_simd(Self::TAIL_START, cur_vec, Self::TAIL_SIZE);
        }

        result
    }
}

impl<T: SimdFloat, const N: usize> SimdArray<T, N> {
    pub fn fract(&self) -> Self {
        let mut result = Self::new_uninit();
        for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            let data = self.load_simd(i);
            result.store_simd(i, data.fract());
        }

        if Self::HAS_TAIL {
            let data = self.load_simd(Self::TAIL_START);
            result.partial_store_simd(Self::TAIL_START, data.fract(), Self::TAIL_SIZE);
        }

        result
    }

    pub fn max(&self, val: T) -> Self {
        let max_vec = ArchSimd::<T>::splat(val);
        let mut result = Self::new_uninit();
        for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            let data = self.load_simd(i);
            result.store_simd(i, data.max(max_vec));
        }

        if Self::HAS_TAIL {
            let data = self.load_simd(Self::TAIL_START);
            result.partial_store_simd(Self::TAIL_START, data.max(max_vec), Self::TAIL_SIZE);
        }

        result
    }

    pub fn min(&self, val: T) -> Self {
        let min_vec = ArchSimd::<T>::splat(val);
        let mut result = Self::new_uninit();
        for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            let data = self.load_simd(i);
            result.store_simd(i, data.min(min_vec));
        }

        if Self::HAS_TAIL {
            let data = self.load_simd(Self::TAIL_START);
            result.partial_store_simd(Self::TAIL_START, data.min(min_vec), Self::TAIL_SIZE);
        }

        result
    }
}

impl<T: SimdFloat, const N: usize> SimdArray<T, N> {
    pub fn mul_add(self, mult: Self, offset: Self) -> Self {
        let mut result = Self::new_uninit();
        for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            let data_vec = self.load_simd(i);
            let mult_vec = mult.load_simd(i);
            let offset_vec = offset.load_simd(i);

            let new_data = data_vec.mul_add(mult_vec, offset_vec);
            result.store_simd(i, new_data);
        }

        if Self::HAS_TAIL {
            let data_vec = self.load_simd(Self::TAIL_START);
            let mult_vec = mult.load_simd(Self::TAIL_START);
            let offset_vec = offset.load_simd(Self::TAIL_START);

            let new_data = data_vec.mul_add(mult_vec, offset_vec);
            result.partial_store_simd(Self::TAIL_START, new_data, Self::TAIL_SIZE);
        }

        result
    }

    pub fn mul_sub(self, mult: Self, offset: Self) -> Self {
        let mut result = Self::new_uninit();
        for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            let data_vec = self.load_simd(i);
            let mult_vec = mult.load_simd(i);
            let offset_vec = offset.load_simd(i);

            let new_data = data_vec.mul_sub(mult_vec, offset_vec);
            result.store_simd(i, new_data);
        }

        if Self::HAS_TAIL {
            let data_vec = self.load_simd(Self::TAIL_START);
            let mult_vec = mult.load_simd(Self::TAIL_START);
            let offset_vec = offset.load_simd(Self::TAIL_START);

            let new_data = data_vec.mul_sub(mult_vec, offset_vec);
            result.partial_store_simd(Self::TAIL_START, new_data, Self::TAIL_SIZE);
        }

        result
    }
}

// impl<T: SimdFloat, const N: usize> SimdArray<T, N> {
//     pub fn mul_sub(self, mult: Self, offset: Self) -> Self {
//         self.mul_add(mult, -offset)
//     }
// }

impl<T: SimdFloat, const N: usize> SimdArray<T, N> {
    pub fn quintic_lerp(&self) -> Self {
        let mut result = Self::new_uninit();

        let six = ArchSimd::splat(NumCast::from(6.0).unwrap());
        let ten = ArchSimd::splat(NumCast::from(10.0).unwrap());
        let neg_fifteen = ArchSimd::splat(NumCast::from(-15.0).unwrap());
        
        for i in (0..Self::TAIL_START).step_by(ArchSimd::<T>::LANES) {
            let t = self.load_simd(i);
            let new_data = t * t * t * t.mul_add(t.mul_add(six, neg_fifteen), ten);
            result.store_simd(i, new_data);
        }

        if Self::HAS_TAIL {
            let t = self.load_simd(Self::TAIL_START);
            let new_data = t * t * t * t.mul_add(t.mul_add(six, neg_fifteen), ten);
            result.partial_store_simd(Self::TAIL_START, new_data, Self::TAIL_SIZE);
        }

        result
    }
}

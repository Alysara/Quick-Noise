use crate::simd::architectures::arch_impl::*;
use crate::simd::traits::*;
use std::ops::*;
use num_traits::NumCast;
use crate::simd::simd_vec::core::SimdVec;
use crate::simd::simd_traits::*;
use crate::simd::arch_simd::{ArchFamily, ArchSimd, ArchMask};


// === Operations based on bit size ===
impl<T: SimdInteger, F: SimdFamily> BitAnd for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self::new(F::Vec::and(self.data, rhs.data))        
    }
}

impl<T: SimdInteger, F: SimdFamily> BitOr for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self::new(F::Vec::or(self.data, rhs.data))        
    }
}

impl<T: SimdInteger, F: SimdFamily> BitXor for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(F::Vec::xor(self.data, rhs.data))        
    }
}

impl<T: SimdInteger, F: SimdFamily> SimdAndNot for SimdVec<T, F> {
    #[inline(always)]
    fn andnot(self, rhs: Self) -> Self {
        Self::new(F::Vec::and_not(self.data, rhs.data))        
    }
}

// === Shifts ===
impl<T: SimdIntegerNotByte, F: SimdFamily> Shl<Self> for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn shl(self, rhs: Self) -> Self {
        Self::new(
            match T::BIT_SIZE {
                BitSize::Size64 => self.data.sllv_64(rhs.data),
                BitSize::Size32 => self.data.sllv_32(rhs.data),
                BitSize::Size16 => self.data.sllv_16(rhs.data),
                _ => unreachable!()
            }
        )
    }
}

impl<T: SimdIntegerNotByte, F: SimdFamily> Shr<Self> for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn shr(self, rhs: Self) -> Self {
        Self::new(
            match T::TYPE {
                SimdType::U64 => self.data.srlv_64(rhs.data),
                SimdType::U32 => self.data.srlv_32(rhs.data),
                SimdType::U16 => self.data.srlv_16(rhs.data),
                SimdType::I64 => self.data.srav_64(rhs.data),
                SimdType::I32 => self.data.srav_32(rhs.data),
                SimdType::I16 => self.data.srav_16(rhs.data),
                _ => unreachable!()
            }
        )
    }
}

// === Scalar shifts ===

impl<T: SimdIntegerNotByte, F: SimdFamily> Shl<usize> for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn shl(self, rhs: usize) -> Self {
        let shift = Self::splat(NumCast::from(rhs).unwrap());
        self << shift
    }
}

impl<T: SimdIntegerNotByte, F: SimdFamily> Shr<usize> for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn shr(self, rhs: usize) -> Self {
        let shift = Self::splat(NumCast::from(rhs).unwrap());
        self >> shift
    }
}

// === Addition ===

impl<T: SimdElement, F: SimdFamily> Add for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self::new(
            match T::TYPE {
                SimdType::F64 => self.data.f64_add(rhs.data),
                SimdType::F32 => self.data.f32_add(rhs.data),
                SimdType::I64 => self.data.i64_add(rhs.data),
                SimdType::I32 => self.data.i32_add(rhs.data),
                SimdType::I16 => self.data.i16_add(rhs.data),
                SimdType::I8 => self.data.i8_add(rhs.data),
                SimdType::U64 => self.data.i64_add(rhs.data),
                SimdType::U32 => self.data.i32_add(rhs.data),
                SimdType::U16 => self.data.i16_add(rhs.data),
                SimdType::U8 => self.data.i8_add(rhs.data),
            }
        )
    }
}

impl<T: SimdElement, F: SimdFamily> Sub for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self::new(
            match T::TYPE {
                SimdType::F64 => self.data.f64_sub(rhs.data),
                SimdType::F32 => self.data.f32_sub(rhs.data),
                SimdType::I64 => self.data.i64_sub(rhs.data),
                SimdType::I32 => self.data.i32_sub(rhs.data),
                SimdType::I16 => self.data.i16_sub(rhs.data),
                SimdType::I8 => self.data.i8_sub(rhs.data),
                SimdType::U64 => self.data.i64_sub(rhs.data),
                SimdType::U32 => self.data.i32_sub(rhs.data),
                SimdType::U16 => self.data.i16_sub(rhs.data),
                SimdType::U8 => self.data.i8_sub(rhs.data),
            }
        )
    }
}

impl<T: SimdMulType, F: SimdFamily> Mul for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self::new(
            match T::TYPE {
                SimdType::F64 => self.data.f64_mul(rhs.data),
                SimdType::F32 => self.data.f32_mul(rhs.data),
                SimdType::I32 => self.data.i32_mul(rhs.data),
                SimdType::I16 => self.data.i16_mul(rhs.data),
                SimdType::U32 => self.data.i32_mul(rhs.data),
                SimdType::U16 => self.data.i16_mul(rhs.data),
                _ => unreachable!()
            }
        )
    }
}

impl<T: SimdFloat, F: SimdFamily> Div for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        Self::new(
            match T::TYPE {
                SimdType::F64 => self.data.f64_div(rhs.data),
                SimdType::F32 => self.data.f32_div(rhs.data),
                _ => unreachable!()
            }
        )
    }
}


// TODO TODO
// impl<T: SimdInteger, F: SimdFamily> Rem for SimdVec<T, F> {
//     type Output = Self;
//     #[inline(always)]
//     fn rem(self, rhs: Self) -> Self {
//         Self::new(
//             match T::TYPE {
//                 SimdType::F64 => self.data.f64_sub(rhs.data),
//                 SimdType::F32 => self.data.f32_sub(rhs.data),
//                 SimdType::I64 => self.data.i64_sub(rhs.data),
//                 SimdType::I32 => self.data.i32_sub(rhs.data),
//                 SimdType::I16 => self.data.i16_sub(rhs.data),
//                 SimdType::I8 => self.data.i8_sub(rhs.data),
//                 SimdType::U64 => self.data.i64_sub(rhs.data),
//                 SimdType::U32 => self.data.i32_sub(rhs.data),
//                 SimdType::U16 => self.data.i16_sub(rhs.data),
//                 SimdType::U8 => self.data.i8_sub(rhs.data),
//             }
//         )
//     }
// }

// === Casts ===

impl<T: SimdInteger + HasSigned, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn cast_signed(self) -> SimdVec<T::Signed, F> {
        SimdVec::new(self.data)
    }
}

impl<T: SimdInteger + HasUnsigned, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn cast_unsigned(self) -> SimdVec<T::Unsigned, F> {
        SimdVec::new(self.data)
    }
}

impl<T: SimdInteger + HasFloat, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn cast_float(self) -> SimdVec<T::Float, F> {
        SimdVec::new(self.data.int_to_float())
    }
}

// === Clamp ===
// impl<T: SimdElement, F: SimdFamily> SimdVec<T, F> {
//     #[inline(always)]
//     pub fn clamp(self, min: T, max: T) -> Self {
//         let min_vec = Self::splat(min);
//         let max_vec = Self::splat(max);
//         self.blendself.simd_lt(min_vec)
//     }
// }
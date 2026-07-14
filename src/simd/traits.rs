use std::fmt::Debug;

use num_traits::{NumCast, NumOps};

use crate::simd::architectures::interface::SimdFamily;
use crate::simd::array_trait::Array;

mod private {
    pub trait SealedTypes {}
    impl SealedTypes for f64 {}
    impl SealedTypes for f32 {}
    impl SealedTypes for i64 {}
    impl SealedTypes for i32 {}
    impl SealedTypes for i16 {}
    impl SealedTypes for i8 {}
    impl SealedTypes for u64 {}
    impl SealedTypes for u32 {}
    impl SealedTypes for u16 {}
    impl SealedTypes for u8 {}

    pub trait SealedSizes {}
    impl SealedSizes for super::B64 {}
    impl SealedSizes for super::B32 {}
    impl SealedSizes for super::B16 {}
    impl SealedSizes for super::B8 {}
}

pub enum BitSize {
    Size64,
    Size32,
    Size16,
    Size8,
}

pub enum PrimitiveType {
    Float,
    SignedInt,
    UnsignedInt,
}

pub enum SimdType {
    F64,
    F32,
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
}

pub struct B64;
pub struct B32;
pub struct B16;
pub struct B8;

pub trait BitWidth: private::SealedSizes {
    const BIT_SIZE: usize;
}

impl BitWidth for B64 {
    const BIT_SIZE: usize = 64;
}
impl BitWidth for B32 {
    const BIT_SIZE: usize = 32;
}
impl BitWidth for B16 {
    const BIT_SIZE: usize = 16;
}
impl BitWidth for B8 {
    const BIT_SIZE: usize = 8;
}

pub trait SafeAdd {
    fn safe_add(self, rhs: Self) -> Self;
}

impl SafeAdd for f64 {
    fn safe_add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl SafeAdd for f32 {
    fn safe_add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl SafeAdd for u64 {
    fn safe_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl SafeAdd for u32 {
    fn safe_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl SafeAdd for u16 {
    fn safe_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl SafeAdd for u8 {
    fn safe_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl SafeAdd for i64 {
    fn safe_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl SafeAdd for i32 {
    fn safe_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl SafeAdd for i16 {
    fn safe_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl SafeAdd for i8 {
    fn safe_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

// pub trait Array<T> {
//     fn from_fn(f: impl FnMut(usize) -> T) -> Self;
//     fn get_array<const M: usize>() -> [T; M];
//     fn as_slice(&self) -> &[T];
//     fn as_array<const M: usize>(self) -> [T; M];
// }

// impl<const N: usize, T: Default> Array<T> for [T; N] {
//     fn from_fn(f: impl FnMut(usize) -> T) -> [T; N] {
//         std::array::from_fn(f)
//     }

//     fn get_array<const M: usize>() -> [T; M] {
//         std::array::from_fn(|_| T::default())
//     }

//     fn as_slice(&self) -> &[T] {
//         self.as_slice()
//     }

//     fn as_array<const M: usize>(self) -> [T; M] {
//         assert_eq!(N, M, "size mismatch");
//         unsafe { std::mem::transmute_copy(&self) }
//     }
// }

// Need both enum and associated type for matching and bounds.
pub trait SimdElement:
    private::SealedTypes + PartialEq + Sized + Default + Copy + NumCast + NumOps + Debug + SafeAdd
{
    const BIT_SIZE: BitSize;
    const PRIMITIVE_TYPE: PrimitiveType;
    const TYPE: SimdType;
    type BitWidthType: BitWidth;
    type Array<F: SimdFamily>: Debug + Copy + Array<Self>; // Array wrapper to get around const generics limitations.
    type UType: SimdElement;
}

impl SimdElement for f64 {
    const BIT_SIZE: BitSize = BitSize::Size64;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::Float;
    const TYPE: SimdType = SimdType::F64;
    type BitWidthType = B64;
    type Array<F: SimdFamily> = F::Array64<f64>;
    type UType = u64;
}
impl SimdElement for f32 {
    const BIT_SIZE: BitSize = BitSize::Size32;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::Float;
    const TYPE: SimdType = SimdType::F32;
    type BitWidthType = B32;
    type Array<F: SimdFamily> = F::Array32<f32>;
    type UType = u32;
}
impl SimdElement for i64 {
    const BIT_SIZE: BitSize = BitSize::Size64;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::SignedInt;
    const TYPE: SimdType = SimdType::I64;
    type BitWidthType = B64;
    type Array<F: SimdFamily> = F::Array64<i64>;
    type UType = u64;
}
impl SimdElement for i32 {
    const BIT_SIZE: BitSize = BitSize::Size32;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::SignedInt;
    const TYPE: SimdType = SimdType::I32;
    type BitWidthType = B32;
    type Array<F: SimdFamily> = F::Array32<i32>;
    type UType = u32;
}
impl SimdElement for i16 {
    const BIT_SIZE: BitSize = BitSize::Size16;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::SignedInt;
    const TYPE: SimdType = SimdType::I16;
    type BitWidthType = B16;
    type Array<F: SimdFamily> = F::Array16<i16>;
    type UType = u16;
}
impl SimdElement for i8 {
    const BIT_SIZE: BitSize = BitSize::Size8;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::SignedInt;
    const TYPE: SimdType = SimdType::I8;
    type BitWidthType = B8;
    type Array<F: SimdFamily> = F::Array8<i8>;
    type UType = u8;
}
impl SimdElement for u64 {
    const BIT_SIZE: BitSize = BitSize::Size64;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::UnsignedInt;
    const TYPE: SimdType = SimdType::U64;
    type BitWidthType = B64;
    type Array<F: SimdFamily> = F::Array64<u64>;
    type UType = u64;
}
impl SimdElement for u32 {
    const BIT_SIZE: BitSize = BitSize::Size32;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::UnsignedInt;
    const TYPE: SimdType = SimdType::U32;
    type BitWidthType = B32;
    type Array<F: SimdFamily> = F::Array32<u32>;
    type UType = u32;
}
impl SimdElement for u16 {
    const BIT_SIZE: BitSize = BitSize::Size16;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::UnsignedInt;
    const TYPE: SimdType = SimdType::U16;
    type BitWidthType = B16;
    type Array<F: SimdFamily> = F::Array16<u16>;
    type UType = u16;
}
impl SimdElement for u8 {
    const BIT_SIZE: BitSize = BitSize::Size8;
    const PRIMITIVE_TYPE: PrimitiveType = PrimitiveType::UnsignedInt;
    const TYPE: SimdType = SimdType::U8;
    type BitWidthType = B8;
    type Array<F: SimdFamily> = F::Array8<u8>;
    type UType = u8;
}

// pub trait BitSize: private::Sealed + Sized {
//     const BITS: usize;
//     type BitSize;
// }

// impl BitSize for f64 { const BITS: usize = 64; type BitSize = BitSize64; }
// impl BitSize for f32 { const BITS: usize = 32; type BitSize = BitSize32; }
// impl BitSize for i64 { const BITS: usize = 64; type BitSize = BitSize64; }
// impl BitSize for i32 { const BITS: usize = 32; type BitSize = BitSize32; }
// impl BitSize for i16 { const BITS: usize = 16; type BitSize = BitSize16; }
// impl BitSize for i8 { const BITS: usize = 8; type BitSize = BitSize8; }
// impl BitSize for u64 { const BITS: usize = 64; type BitSize = BitSize64; }
// impl BitSize for u32 { const BITS: usize = 32; type BitSize = BitSize32; }
// impl BitSize for u16 { const BITS: usize = 16; type BitSize = BitSize16; }
// impl BitSize for u8 { const BITS: usize = 8; type BitSize = BitSize8; }

pub struct Signed;
pub struct Unsigned;

pub trait SimdInteger: SimdElement {
    type Type;
    type Unsigned: SimdElement;
    type Signed: SimdElement;
}

impl SimdInteger for i64 {
    type Type = Signed;
    type Unsigned = u64;
    type Signed = i64;
}
impl SimdInteger for i32 {
    type Type = Signed;
    type Unsigned = u32;
    type Signed = i32;
}
impl SimdInteger for i16 {
    type Type = Signed;
    type Unsigned = u16;
    type Signed = i16;
}
impl SimdInteger for i8 {
    type Type = Signed;
    type Unsigned = u8;
    type Signed = i8;
}
impl SimdInteger for u64 {
    type Type = Unsigned;
    type Unsigned = u64;
    type Signed = i64;
}
impl SimdInteger for u32 {
    type Type = Unsigned;
    type Unsigned = u32;
    type Signed = i32;
}
impl SimdInteger for u16 {
    type Type = Unsigned;
    type Unsigned = u16;
    type Signed = i16;
}
impl SimdInteger for u8 {
    type Type = Unsigned;
    type Unsigned = u8;
    type Signed = i8;
}

pub trait SimdFloat: SimdElement + HasSigned + HasUnsigned + SimdMulType {
    const SIGN_MASK: usize;
}

impl SimdFloat for f64 {
    const SIGN_MASK: usize = 0x7FFFFFFFFFFFFFFF;
}
impl SimdFloat for f32 {
    const SIGN_MASK: usize = 0x7FFFFFFF;
}

pub trait SimdWideType: SimdElement {}
impl SimdWideType for f64 {}
impl SimdWideType for f32 {}
impl SimdWideType for i64 {}
impl SimdWideType for i32 {}
impl SimdWideType for u64 {}
impl SimdWideType for u32 {}

pub trait SimdIntegerNotByte: SimdInteger {}
impl SimdIntegerNotByte for i64 {}
impl SimdIntegerNotByte for i32 {}
impl SimdIntegerNotByte for u64 {}
impl SimdIntegerNotByte for u32 {}
impl SimdIntegerNotByte for i16 {}
impl SimdIntegerNotByte for u16 {}

pub trait SimdMulType: SimdElement {}
impl SimdMulType for f64 {}
impl SimdMulType for f32 {}
impl SimdMulType for i32 {}
impl SimdMulType for i16 {}
impl SimdMulType for u32 {}
impl SimdMulType for u16 {}

// pub trait BitSize64: Sized + private::Sealed {}
// impl BitSize64 for f64 {}
// impl BitSize64 for i64 {}
// impl BitSize64 for u64 {}

// pub trait BitSize32: Sized + private::Sealed {}
// impl BitSize32 for f32 {}
// impl BitSize32 for i32 {}
// impl BitSize32 for u32 {}

// pub trait BitSize16: Sized + private::Sealed {}
// impl BitSize16 for i16 {}
// impl BitSize16 for u16 {}

// pub trait BitSize8: Sized + private::Sealed {}
// impl BitSize8 for i8 {}
// impl BitSize8 for u8 {}

pub trait HasFloat: SimdElement {
    type Float: SimdElement;
}

pub trait HasSigned: SimdElement {
    type Signed: SimdElement;
}

pub trait HasUnsigned: SimdElement {
    type Unsigned: SimdElement;
}

// Floats.
impl HasSigned for f64 {
    type Signed = i64;
}
impl HasUnsigned for f64 {
    type Unsigned = u64;
}
impl HasSigned for f32 {
    type Signed = i32;
}
impl HasUnsigned for f32 {
    type Unsigned = u32;
}

// Signed Integers.
impl HasFloat for i64 {
    type Float = f64;
}
impl HasUnsigned for i64 {
    type Unsigned = u64;
}
impl HasFloat for i32 {
    type Float = f32;
}
impl HasUnsigned for i32 {
    type Unsigned = u32;
}
impl HasUnsigned for i16 {
    type Unsigned = u16;
}
impl HasUnsigned for i8 {
    type Unsigned = u8;
}

// Unsigned Integers.
impl HasFloat for u64 {
    type Float = f64;
}
impl HasSigned for u64 {
    type Signed = i64;
}
impl HasFloat for u32 {
    type Float = f32;
}
impl HasSigned for u32 {
    type Signed = i32;
}
impl HasSigned for u16 {
    type Signed = i16;
}
impl HasSigned for u8 {
    type Signed = i8;
}

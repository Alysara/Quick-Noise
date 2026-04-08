// TODO: Potentially make this a wrapper for simd_array under the hood.

use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Rem, RemAssign, Index, IndexMut};
use num_traits::float::*;
use std::cmp::PartialOrd;

#[derive(Debug, Clone, Copy)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T
}

#[derive(Debug, Clone, Copy)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

// === Constructors ===

impl<T> Vec2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Vec2::<T> {x, y}
    }
}

impl<T: Copy> Vec2<T> {
    pub fn splat(val: T) -> Self {
        Vec2::<T> {x: val, y: val}
    }
}

impl<T> Vec3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Vec3::<T> {x, y, z}
    }
}

impl<T: Copy> Vec3<T> {
    pub fn splat(val: T) -> Self {
        Vec3::<T> {x: val, y: val, z: val}
    }
}

// === From Constructors ===

impl<T> From<(T, T)> for Vec2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}

impl<T> From<(T, T, T)> for Vec3<T> {
    fn from((x, y, z): (T, T, T)) -> Self {
        Self::new(x, y, z)
    }
}

impl<T: Copy> From<T> for Vec2<T> {
    fn from(val: T) -> Self {
        Self::splat(val)
    }
}

impl<T: Copy> From<T> for Vec3<T> {
    fn from(val: T) -> Self {
        Self::splat(val)
    }
}

// === Indexing ===

impl<T> IndexMut<usize> for Vec2<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Vec2 index out of bounds: {i}"),
        }
    }
}

impl<T> IndexMut<usize> for Vec3<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Vec3 index out of bounds: {i}"),
        }
    }
}

impl<T> Index<usize> for Vec2<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vec2 index out of bounds: {i}"),
        }
    }
}

impl<T> Index<usize> for Vec3<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vec3 index out of bounds: {i}"),
        }
    }
}

// === Conversions and Casting ===

impl<T> Vec2<T> {
    pub fn map<U, F>(self, f: F) -> Vec2<U>
    where
        F: Fn(T) -> U,
    {
        Vec2 {
            x: f(self.x),
            y: f(self.y),
        }
    }
}

impl<T> Vec2<T> {
    pub fn cast<U>(self) -> Vec2<U>
    where
        T: Into<U>,
    {
        Vec2 {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

impl<T> Vec3<T> {
    pub fn map<U, F>(self, f: F) -> Vec3<U>
    where
        F: Fn(T) -> U,
    {
        Vec3 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }
}

impl<T> Vec3<T> {
    pub fn cast<U>(self) -> Vec3<U>
    where
        T: Into<U>,
    {
        Vec3 {
            x: self.x.into(),
            y: self.y.into(),
            z: self.z.into(),
        }
    }
}

impl Vec2<f32> {
    pub fn as_i32(self) -> Vec2<i32> {
        self.map(|x| x as i32)
    }
}

impl Vec2<i32> {
    pub fn as_f32(self) -> Vec2<f32> {
        self.map(|x| x as f32)
    }
}

impl Vec3<f32> {
    pub fn as_i32(self) -> Vec3<i32> {
        self.map(|x| x as i32)
    }
}

impl Vec3<i32> {
    pub fn as_f32(self) -> Vec3<f32> {
        self.map(|x| x as f32)
    }
}

impl Vec2<f32> {
    pub fn as_u32(self) -> Vec2<u32> {
        self.map(|x| x as u32)
    }
}

impl Vec2<u32> {
    pub fn as_f32(self) -> Vec2<f32> {
        self.map(|x| x as f32)
    }
}

impl Vec3<f32> {
    pub fn as_u32(self) -> Vec3<u32> {
        self.map(|x| x as u32)
    }
}

impl Vec3<u32> {
    pub fn as_f32(self) -> Vec3<f32> {
        self.map(|x| x as f32)
    }
}

// === Comparison-based Operators ===

impl<T: Ord> Vec2<T> {
    pub fn max(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    pub fn min(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }
}

impl<T: Float> Vec2<T> {
    pub fn float_max(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    pub fn float_min(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }
}

impl<T: Ord> Vec3<T> {
    pub fn max(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    pub fn min(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }
}

impl<T: Float> Vec3<T> {
    pub fn float_max(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    pub fn float_min(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }
}

// === Floating point operations ===

impl<T: Float> Vec2<T> {
    pub fn floor(&mut self) -> &Self {
        self.x = self.x.floor();
        self.y = self.y.floor();
        self
    }

    pub fn ceil(&mut self) -> &Self {
        self.x = self.x.ceil();
        self.y = self.y.ceil();
        self
    }

    pub fn round(&mut self) -> &Self {
        self.x = self.x.round();
        self.y = self.y.round();
        self
    }

    pub fn trunc(&mut self) -> &Self {
        self.x = self.x.trunc();
        self.y = self.y.trunc();
        self
    }
}

impl<T: Float> Vec3<T> {
    pub fn floor(&mut self) -> &Self {
        self.x = self.x.floor();
        self.y = self.y.floor();
        self.z = self.z.floor();
        self
    }

    pub fn ceil(&mut self) -> &Self {
        self.x = self.x.ceil();
        self.y = self.y.ceil();
        self.z = self.z.ceil();
        self
    }

    pub fn round(&mut self) -> &Self {
        self.x = self.x.round();
        self.y = self.y.round();
        self.z = self.z.round();
        self
    }

    pub fn trunc(&mut self) -> &Self {
        self.x = self.x.trunc();
        self.y = self.y.trunc();
        self.z = self.z.trunc();
        self
    }
}

// === Horizontal Operations ===

impl<T: Add<Output = T>> Vec2<T> {
    pub fn sum(self) -> T {
        self.x + self.y
    }
}

impl<T: Add<Output = T>> Vec3<T> {
    pub fn sum(self) -> T {
        self.x + self.y + self.z
    }
}

// === Basic Operations ===

macro_rules! impl_vec_ops {(
        $VecType:ident { $($field:ident),+ },
        $OpTrait:ident,
        $op_method:ident,
        $op:tt
    ) => {
        impl<T: $OpTrait<Output = T>> $OpTrait for $VecType<T> {
            type Output = $VecType<T>;
            fn $op_method(self, other: $VecType<T>) -> Self::Output {
                $VecType {
                    $($field: self.$field $op other.$field,)+
                }
            }
        }
    };
    
    (
        $VecType:ident { $($field:ident),+ },
        $OpTrait:ident,
        $op_method:ident,
        $op:tt,
        assign
    ) => {
        impl<T: $OpTrait> $OpTrait for $VecType<T> {
            fn $op_method(&mut self, other: $VecType<T>) {
                $(self.$field $op other.$field;)+
            }
        }
    };
    
    (
        $VecType:ident { $($field:ident),+ },
        $OpTrait:ident, $op_method:ident,
        $op:tt,
        scalar
    ) => {
        impl<T: $OpTrait<Output = T> + Copy> $OpTrait<T> for $VecType<T> {
            type Output = $VecType<T>;
            fn $op_method(self, scalar: T) -> Self::Output {
                $VecType {
                    $($field: self.$field $op scalar,)+
                }
            }
        }
    };
    
    (
        $VecType:ident { $($field:ident),+ },
        $OpTrait:ident,
        $op_method:ident,
        $op:tt,
        scalar_assign
    ) => {
        impl<T: $OpTrait + Copy> $OpTrait<T> for $VecType<T> {
            fn $op_method(&mut self, scalar: T) {
                $(self.$field $op scalar;)+
            }
        }
    };
}

macro_rules! impl_all_vec_ops {
    ($VecType:ident { $($field:ident),+ }) => {
        impl_vec_ops!($VecType { $($field),+ }, Add, add, +);
        impl_vec_ops!($VecType { $($field),+ }, Sub, sub, -);
        impl_vec_ops!($VecType { $($field),+ }, Mul, mul, *);
        impl_vec_ops!($VecType { $($field),+ }, Div, div, /);
        impl_vec_ops!($VecType { $($field),+ }, Rem, rem, %);
        
        impl_vec_ops!($VecType { $($field),+ }, AddAssign, add_assign, +=, assign);
        impl_vec_ops!($VecType { $($field),+ }, SubAssign, sub_assign, -=, assign);
        impl_vec_ops!($VecType { $($field),+ }, MulAssign, mul_assign, *=, assign);
        impl_vec_ops!($VecType { $($field),+ }, DivAssign, div_assign, /=, assign);
        impl_vec_ops!($VecType { $($field),+ }, RemAssign, rem_assign, %=, assign);
        
        impl_vec_ops!($VecType { $($field),+ }, Add, add, +, scalar);
        impl_vec_ops!($VecType { $($field),+ }, Sub, sub, -, scalar);
        impl_vec_ops!($VecType { $($field),+ }, Mul, mul, *, scalar);
        impl_vec_ops!($VecType { $($field),+ }, Div, div, /, scalar);
        impl_vec_ops!($VecType { $($field),+ }, Rem, rem, %, scalar);
        
        impl_vec_ops!($VecType { $($field),+ }, AddAssign, add_assign, +=, scalar_assign);
        impl_vec_ops!($VecType { $($field),+ }, SubAssign, sub_assign, -=, scalar_assign);
        impl_vec_ops!($VecType { $($field),+ }, MulAssign, mul_assign, *=, scalar_assign);
        impl_vec_ops!($VecType { $($field),+ }, DivAssign, div_assign, /=, scalar_assign);
        impl_vec_ops!($VecType { $($field),+ }, RemAssign, rem_assign, %=, scalar_assign);
    };
}

impl_all_vec_ops!(Vec2 { x, y });
impl_all_vec_ops!(Vec3 { x, y, z });

macro_rules! impl_scalar_ops {(
        $ScalarType:ty,
        $VecType:ident { $($field:ident),+ },
        $OpTrait:ident,
        $op_method:ident,
        $op:tt
    ) => {
        impl $OpTrait<$VecType<$ScalarType>> for $ScalarType {
            type Output = $VecType<$ScalarType>;
            
            fn $op_method(self, vec: $VecType<$ScalarType>) -> Self::Output {
                $VecType {
                    $($field: self $op vec.$field,)+
                }
            }
        }
    };
}

macro_rules! impl_all_scalar_ops {
    ($ScalarType:ty) => {
        impl_scalar_ops!($ScalarType, Vec2 {x, y}, Add, add, +);
        impl_scalar_ops!($ScalarType, Vec2 {x, y}, Sub, sub, -);
        impl_scalar_ops!($ScalarType, Vec2 {x, y}, Mul, mul, *);
        impl_scalar_ops!($ScalarType, Vec2 {x, y}, Div, div, /);
        impl_scalar_ops!($ScalarType, Vec2 {x, y}, Rem, rem, %);

        impl_scalar_ops!($ScalarType, Vec3 {x, y, z}, Add, add, +);
        impl_scalar_ops!($ScalarType, Vec3 {x, y, z}, Sub, sub, -);
        impl_scalar_ops!($ScalarType, Vec3 {x, y, z}, Mul, mul, *);
        impl_scalar_ops!($ScalarType, Vec3 {x, y, z}, Div, div, /);
        impl_scalar_ops!($ScalarType, Vec3 {x, y, z}, Rem, rem, %);
    };
}

impl_all_scalar_ops!(f32);
impl_all_scalar_ops!(f64);
impl_all_scalar_ops!(u8);
impl_all_scalar_ops!(u16);
impl_all_scalar_ops!(u32);
impl_all_scalar_ops!(u64);
impl_all_scalar_ops!(i32);
impl_all_scalar_ops!(i64);

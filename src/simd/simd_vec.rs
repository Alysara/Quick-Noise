use crate::simd::arch_simd::{ArchMask, ArchSimd};
use crate::simd::simd_traits::*;
use crate::simd::traits::{SimdElement, SimdFloat};
use itertools::izip;
use num_traits::NumCast;
use std::cmp::min;
use std::fmt;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::ops::*;

// ————————————————————————————————————————————————————————————————
// ————— Struct ———————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// A SIMD-optimized array wrapper with compile-time size `N` of element type `T`.
///
/// The array is aligned to 64 bytes for optimal SIMD performance across all
/// supported architectures.
/// [`MaybeUninit`] is used internally to support uninitialized elements.
///
/// # Type Parameters
/// * `T` - Element type, must implement [`SimdElement`]. [`SimdFloat`] Offers float operations
/// * `N` - Number of elements
///
/// # Example
/// ```
/// use quick_noise::simd::simd_array::SimdVec;
/// let arr = SimdVec::<f32, 8>::new(1.0);
/// assert_eq!(arr[0], 1.0);
/// ```
#[derive(Clone, Default)]
pub struct SimdVec<T: SimdElement> {
    data: Vec<T>,
}

// ————————————————————————————————————————————————————————————————
// ————— Constructors —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement> SimdVec<T> {
    /// Creates a new simd array and initializes all elements to a given value.
    ///
    /// # Parameters
    /// * `value` - The value to fill every element of the array with.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    ///
    /// // Becomes [4.0, 4.0, 4.0, 4.0, 4.0]
    /// let arr = SimdVec::<f32, 5>::new(4.0);
    /// assert_eq!(arr[0], 4.0);
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn from_vec(vec: Vec<T>) -> Self {
        Self { data: vec }
    }

    pub fn take_vec(vec: &mut Vec<T>) -> Self {
        Self {
            data: std::mem::take(vec),
        }
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    #[inline(always)]
    pub unsafe fn new_uninit(capacity: usize) -> Self {
        let mut vec = Self::with_capacity(capacity);
        unsafe { vec.set_len(capacity); }
        vec
    }
}

#[macro_export]
macro_rules! simd_vec {
    ($($tt:tt)*) => {
        SimdVec::from_vec(vec![$($tt)*])
    };
}

impl<T: SimdElement> SimdVec<T> {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    #[inline(always)]
    pub fn push(&mut self, val: T) {
        self.data.push(val);
    }

    #[inline(always)]
    pub fn reserve(&mut self, amount: usize) {
        self.data.reserve(amount);
    }

    #[inline(always)]
    pub fn push_simd(&mut self, vec: ArchSimd<T>) {
        self.reserve(ArchSimd::<T>::LANES);
        let slice = self.data.spare_capacity_mut();
        unsafe {
            vec.store(slice.assume_init_mut());
            self.set_len(self.len() + ArchSimd::<T>::LANES);
        }
    }

    #[inline(always)]
    pub fn to_vec(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    #[inline(always)]
    pub unsafe fn set_len(&mut self, size: usize) {
        unsafe {
            self.data.set_len(size);
        }
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Indexing —————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement> Index<usize> for SimdVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: SimdElement> IndexMut<usize> for SimdVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T: SimdElement> SimdVec<T> {
    #[inline(always)]
    fn check_bounds_range(&self, start: usize, size: usize) {
        let end = start + size;
        assert!(
            end <= self.data.len(),
            "Attempted to store to an out of bounds index! (Indices [{start}-{end}] > [0-{}])",
            self.len()
        );
    }

    #[inline(always)]
    fn debug_check_bounds_range(&self, start: usize, size: usize) {
        let end = start + size;
        debug_assert!(
            end <= self.len(),
            "Attempted to store to an out of bounds index in unsafe code! (Indices [{start}-{end}] > [0-{}])",
            self.len()
        );
    }

    /// Obtains the value at a given index without bounds checking.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// let arr = SimdVec::<i32, 5>::iota(0);
    /// assert_eq!(unsafe { arr.get_unchecked(2) }, 2);
    /// ```
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        debug_assert!(index < self.len());
        unsafe { self.data.get_unchecked(index) }
    }

    /// Obtains the mutable reference of a given index without bounds checking.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// let mut arr = SimdVec::<i32, 5>::new(0);
    /// unsafe { *arr.get_unchecked_mut(0) = 100; }
    /// assert_eq!(arr[0], 100);
    /// ```
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.len());
        unsafe { self.data.get_unchecked_mut(index) }
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Utility Traits ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement + fmt::Debug> fmt::Debug for SimdVec<T> {
    /// Prints the values of the [`SimdVec`] with debug formatting.`.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// let arr = SimdVec::<i32>::iota(0);
    /// println!("arr: {:?}", arr);
    /// // Prints "arr: [0, 1, 2, 3]".
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.data.clone()).finish()
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Simd Access ——————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

// TODO: Add partial load.
impl<T: SimdElement> SimdVec<T> {
    /// Returns a simd register containing `ArchSimd::LANES` number
    /// of values starting from a given index.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    /// use quick_noise::simd::simd_traits::SimdToArray; // TODO: Consolidate these.
    ///
    /// let arr = SimdVec::<i32, 12>::iota(0);
    /// let register = arr.load_simd(0);
    /// assert_eq!(register.to_array()[3], 3);
    /// ```
    #[inline(always)]
    pub fn load_simd(&self, index: usize) -> ArchSimd<T> {
        self.check_bounds_range(index, ArchSimd::<T>::LANES);
        ArchSimd::load(&self.data[index..])
    }

    #[inline(always)]
    pub unsafe fn load_simd_unchecked(&self, index: usize) -> ArchSimd<T> {
        self.debug_check_bounds_range(index, ArchSimd::<T>::LANES);
        unsafe { ArchSimd::load(&self.data.get_unchecked(index..)) }
    }

    /// Stores a SIMD register containing `ArchSimd::LANES` number
    /// of values starting from a given index.
    ///
    /// # Parameters
    /// * `index` - Starting index
    /// * `vec` - SIMD vector containing the values to store
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    ///
    /// let mut arr = SimdVec::<i32, 12>::iota(0);
    /// let register = ArchSimd::<i32>::splat(100);
    /// arr.store_simd(0, register);
    /// assert_eq!(arr[0], 100);
    /// ```
    #[inline(always)]
    pub fn store_simd(&mut self, index: usize, vec: ArchSimd<T>) {
        self.check_bounds_range(index, ArchSimd::<T>::LANES);
        vec.store(&mut self.data[index..]);
    }

    #[inline(always)]
    pub unsafe fn store_simd_unchecked(&mut self, index: usize, vec: ArchSimd<T>) {
        self.debug_check_bounds_range(index, ArchSimd::<T>::LANES);
        unsafe { vec.store(&mut self.data.get_unchecked_mut(index..)) };
    }

    /// Stores the first `amount` values in a SIMD register to a [`SimdVec`]
    /// starting from a given index.
    ///
    /// # Parameters
    /// * `index` - Starting index
    /// * `vec` - SIMD vector containing the values to store
    /// * `amount` - Number of elements to store from the SIMD vector
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    ///
    /// let mut arr = SimdVec::<i32, 10>::iota(0);
    /// let register = ArchSimd::<i32>::splat(100);
    /// arr.partial_store_simd(8, register, 2);
    /// assert_eq!(arr[9], 100);
    /// ```
    #[inline(always)]
    pub fn partial_store_simd(&mut self, index: usize, vec: ArchSimd<T>, amount: usize) {
        self.check_bounds_range(index, amount);
        vec.partial_store(&mut self.data[index..], amount);
    }

    #[inline(always)]
    pub unsafe fn partial_store_simd_unchecked(
        &mut self,
        index: usize,
        vec: ArchSimd<T>,
        amount: usize,
    ) {
        self.debug_check_bounds_range(index, amount);
        unsafe { vec.partial_store(self.data.get_unchecked_mut(index..), amount) };
    }

    /// Stores values from a SIMD register based on a mask to a [`SimdVec`]
    /// starting from a given index.
    ///
    /// # Parameters
    /// * `index` - Starting index
    /// * `vec` - SIMD vector containing the values to store
    /// * `mask` - SIMD mask specifying which lanes to store from the SIMD vector
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// use quick_noise::simd::arch_simd::{ArchSimd, ArchMask};
    /// use quick_noise::simd::simd_traits::{SimdIota, SimdPartialOrd};
    ///
    /// let iota = ArchSimd::<i32>::iota(0);
    /// let splat = ArchSimd::<i32>::splat(1);
    ///
    /// // First two lanes false, rest true.
    /// let mask = iota.simd_gt(splat);
    ///
    /// let mut arr = SimdVec::<i32, 12>::new(0);
    /// let register = ArchSimd::<i32>::splat(100);
    /// arr.masked_store_simd(0, register, mask);
    /// assert_eq!(arr[0], 0);
    /// assert_eq!(arr[2], 100);
    /// ```
    #[inline(always)]
    pub unsafe fn masked_store_simd(&mut self, index: usize, vec: ArchSimd<T>, mask: ArchMask<T>) {
        self.debug_check_bounds_range(index, ArchSimd::<T>::LANES);
        unsafe {
            vec.masked_store(&mut self.data.get_unchecked_mut(index..), mask);
        }
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Basic Operator Imeplementations ——————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement> Add for SimdVec<T> {
    type Output = SimdVec<T>;
    fn add(self, rhs: Self) -> SimdVec<T> {
        izip!(self.iter(), rhs.iter()).map(|(x, y)| x + y).collect()
    }
}

impl<T: SimdElement> Sub for SimdVec<T> {
    type Output = SimdVec<T>;
    fn sub(self, rhs: Self) -> SimdVec<T> {
        izip!(self.iter(), rhs.iter()).map(|(x, y)| x - y).collect()
    }
}

// TODO: Flesh out what types can multiply and divide, and ensure where clause won't be needed.
impl<T: SimdElement> Mul for SimdVec<T>
where
    ArchSimd<T>: Mul<Output = ArchSimd<T>>,
{
    type Output = SimdVec<T>;
    fn mul(self, rhs: Self) -> SimdVec<T> {
        izip!(self.iter(), rhs.iter()).map(|(x, y)| x * y).collect()
    }
}

impl<T: SimdElement> Div for SimdVec<T>
where
    ArchSimd<T>: Div<Output = ArchSimd<T>>,
{
    type Output = SimdVec<T>;
    fn div(self, rhs: Self) -> SimdVec<T> {
        izip!(self.iter(), rhs.iter()).map(|(x, y)| x / y).collect()
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Basic Assign Ops —————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement> AddAssign for SimdVec<T> {
    fn add_assign(&mut self, rhs: Self) {
        izip!(self.iter_mut(), rhs.iter())
            .map(|(mut x, y)| *x += y)
            .collect()
    }
}

impl<T: SimdElement> SubAssign for SimdVec<T> {
    fn sub_assign(&mut self, rhs: Self) {
        izip!(self.iter_mut(), rhs.iter())
            .map(|(mut x, y)| *x -= y)
            .collect()
    }
}

impl<T: SimdElement> MulAssign for SimdVec<T>
where
    ArchSimd<T>: MulAssign,
{
    fn mul_assign(&mut self, rhs: Self) {
        izip!(self.iter_mut(), rhs.iter())
            .map(|(mut x, y)| *x *= y)
            .collect()
    }
}

impl<T: SimdElement> DivAssign for SimdVec<T>
where
    ArchSimd<T>: DivAssign,
{
    fn div_assign(&mut self, rhs: Self) {
        izip!(self.iter_mut(), rhs.iter())
            .map(|(mut x, y)| *x /= y)
            .collect()
    }
}

impl<T: SimdElement> Neg for SimdVec<T> {
    type Output = SimdVec<T>;
    fn neg(self) -> SimdVec<T> {
        self.iter().map(|x| -x).collect()
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Additional Operations ————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement> SimdVec<T> {
    /// Takes a list of value vec pairs and broadcasts that value in
    /// each vec a specified amount of elements starting at a specified
    /// index. All vectors must have the same length.
    ///
    /// # Parameters
    /// * `vecs` - an vec of mutable [`SimdVec`]'s
    /// * 'values' - an vec of values to set into each corresponding vec
    /// * 'index' - the index to start setting values at in all vecs
    /// * 'amount' - the amount of elements to fill with the specified value
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_vec::SimdVec;
    ///
    /// let mut arr1 = SimdVec::<f32, 32>::new(10.0);
    /// let mut arr2 = SimdVec::<f32, 32>::new(0.0);
    ///
    /// let mut vecs = [&mut arr1, &mut arr2];
    /// let values = [2.0, 4.0];
    ///
    /// SimdVec::multiset_many(&mut vecs, &values, 2, 10);
    ///
    /// assert_eq!(arr1[2], 2.0);
    /// assert_eq!(arr2[2], 4.0);
    /// ```
    #[inline(always)]
    pub unsafe fn multiset_many<const M: usize>(
        vecs: &mut [&mut Self; M],
        values: &[T; M],
        len: usize,
        mut index: usize,
        mut amount: isize,
    ) {

        // println!("len: {len}, index: {index}, amount: {amount}");
        let registers: [ArchSimd<T>; M] = std::array::from_fn(|i| ArchSimd::<T>::splat(values[i]));

        while amount > 0 {
            // Tail store case.
            if index >= len - ArchSimd::<T>::LANES {
                let tail_size = min(amount as usize, len - index);

                for i in 0..M {
                    unsafe {
                        vecs[i].partial_store_simd_unchecked(
                            index,
                            *registers.get_unchecked(i),
                            tail_size,
                        );
                    }
                }
            }

            // Full register store case.
            for i in 0..M {
                unsafe {
                    vecs[i].store_simd_unchecked(index, *registers.get_unchecked(i));
                }
            }

            amount -= ArchSimd::<T>::LANES as isize;
            index += ArchSimd::<T>::LANES;
        }
    }
}

impl<T: SimdElement + NumCast + Debug> SimdVec<T>
where
    T: Mul<Output = T>,
    ArchSimd<T>: Mul<Output = ArchSimd<T>>,
{
    /// Creates a [`SimdVec`] according to a given linear function,
    /// starting at a specified value at index 0 and incrementing each
    /// subsequent element by a specific increment.
    ///
    /// # Parameters
    /// * `offset` - The starting value
    /// * `increment`` - The amount increased by every subsequent element
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    ///
    /// // Becomes [10.0, 10.5, 11.0, 11.5]
    /// let arr = SimdVec::<f32, 4>::iota_custom(10.0, 0.5);
    ///
    /// assert_eq!(arr[2], 11.0);
    /// ```
    pub fn iota_custom(size: usize, offset: T, increment: T) -> Self {
        // Set the base iota vec, and then repeatedly add an increment each iteration.
        let increment_vec = ArchSimd::splat(increment);
        let lanes_increment_vec =
            ArchSimd::splat(increment * NumCast::from(ArchSimd::<T>::LANES).unwrap());
        let iota_vec = ArchSimd::iota(NumCast::from(0).unwrap()) * increment_vec;

        let mut cur_vec = ArchSimd::splat(offset) + iota_vec;
        let mut result = Self::with_capacity(size);
        unsafe { result.set_len(size) };

        // Set first chunk first to avoid unnecessary tail increment.
        // (Also profiled +2% performance improvement).
        let mut iter  = result.iter_mut();
        *iter.next().unwrap() = cur_vec;
        for mut chunk in iter {
            cur_vec += lanes_increment_vec;
            *chunk = cur_vec;
        }

        // let tail_size = size % ArchSimd::<T>::LANES;
        // let tail_start = size - tail_size;
        // for i in (0..tail_start).step_by(ArchSimd::<T>::LANES) {
        //     result.store_simd(i, cur_vec);
        //     cur_vec += lanes_increment_vec;
        // }
        // result.partial_store_simd(tail_start, cur_vec, tail_size);

        result
    }

    /// Creates a [`SimdVec`] starting at a specific offset at index
    /// 0, and incrementing each subsequent element by 1.
    ///
    /// # Parameters
    /// * `offset` - The starting value
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    ///
    /// // Becomes [2, 3, 4, 5]
    /// let arr = SimdVec::<i32, 4>::iota(2);
    ///
    /// assert_eq!(arr[2], 4);
    pub fn iota(size: usize, offset: T) -> Self {
        // Set the base iota vec, and then repeatedly add an increment each iteration.
        let lanes_increment_vec = ArchSimd::splat(NumCast::from(ArchSimd::<T>::LANES).unwrap());
        let mut cur_vec = ArchSimd::iota(offset);

        let mut result = Self::from_vec(vec![NumCast::from(0).unwrap(); size]);
        let mut iter = result.iter_mut();

        // Set first chunk first to avoid unnecessary tail increment.
        // (Also profiled +2% performance improvement).
        *iter.next().unwrap() = cur_vec;
        for mut chunk in iter {
            cur_vec += lanes_increment_vec;
            *chunk = cur_vec;
        }

        result
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Float Ops ————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdFloat> SimdVec<T> {
    /// Creates a new [`SimdVec`] of floating point values that truncates
    /// the whole number portion off an existing [`SimdVec`], leaving
    /// only the decimal behind.
    ///
    /// Only available when `T` implements [`SimdFloat`].
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    ///
    /// let arr = SimdVec::<f32, 4>::new(14.5);
    /// let new_arr = arr.fract();
    ///
    /// assert_eq!(new_arr[0], 0.5);
    pub fn fract(&self) -> Self {
        self.iter().map(|x| x.fract()).collect()
    }

    // TODO: extend max and min to integer types.
    /// Creates a new [`SimdVec`] containing the values of another
    /// [`SimdVec`] unless it is less than a specified value, in that case
    /// it becomes that value.
    ///
    /// Only available when `T` implements [`SimdFloat`].
    ///
    /// # Parameters
    /// * `val` - The value that is compared to every element in the array
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    ///
    /// let arr = SimdVec::<f32, 4>::new(14.5);
    /// let max_arr = arr.max(15.0);
    ///
    /// assert_eq!(max_arr[0], 15.0);
    pub fn max(&self, val: T) -> Self {
        let max_vec = ArchSimd::splat(val);
        self.iter().map(|x| x.max(max_vec)).collect()
    }

    /// Creates a new [`SimdVec`] containing the values of another
    /// [`SimdVec`] unless it is greater than a specified value, in that case
    /// it becomes that value.
    ///
    /// Only available when `T` implements [`SimdFloat`].
    ///
    /// # Parameters
    /// * `val` - The value that is compared to every element in the array
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    ///
    /// let arr = SimdVec::<f32, 4>::new(14.5);
    /// let min_arr = arr.min(15.0);
    ///
    /// assert_eq!(min_arr[0], 14.5);
    pub fn min(&self, val: T) -> Self {
        let min_vec = ArchSimd::splat(val);
        self.iter().map(|x| x.min(min_vec)).collect()
    }
}

impl<T: SimdFloat> SimdVec<T> {
    /// Creates a new [`SimdVec`] containing the product of two
    /// arrays added with a third. Equivalent to a * b + c.
    ///
    /// Only available when `T` implements [`SimdFloat`].
    ///
    /// # Parameters
    /// * `mult` - The array that is being multiplied with self
    /// * `offset` - The array being added to the product result
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    ///
    /// let a = SimdVec::<f32, 4>::new(2.0);
    /// let b = SimdVec::<f32, 4>::new(4.0);
    /// let c = SimdVec::<f32, 4>::new(6.0);
    /// let result = a.mul_add(b, c);
    ///
    /// assert_eq!(result[0], 14.0);
    /// ```
    pub fn mul_add(self, mult: Self, offset: Self) -> Self {
        izip!(self.iter(), mult.iter(), offset.iter())
            .map(|(a, b, c)| a.mul_add(b, c))
            .collect()
    }

    /// Creates a new [`SimdVec`] containing the product of two
    /// arrays subtracted with a third. Equivalent to a * b - c.
    ///
    /// Only available when `T` implements [`SimdFloat`].
    ///
    /// # Parameters
    /// * `mult` - The array that is being multiplied with self
    /// * `offset` - The array being subtracted from the product result
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    ///
    /// let a = SimdVec::<f32, 4>::new(2.0);
    /// let b = SimdVec::<f32, 4>::new(4.0);
    /// let c = SimdVec::<f32, 4>::new(6.0);
    /// let result = a.mul_sub(b, c);
    ///
    /// assert_eq!(result[0], 2.0);
    /// ```
    pub fn mul_sub(&self, mult: &Self, offset: &Self) -> Self {
        izip!(self.iter(), mult.iter(), offset.iter())
            .map(|(a, b, c)| a.mul_sub(b, c))
            .collect()
    }
}

impl<T: SimdFloat> SimdVec<T> {
    /// Applies the quintic interpolation function to
    /// all elements in an array.
    pub fn quintic_lerp(&self) -> Self {
        let six = ArchSimd::splat(NumCast::from(6.0).unwrap());
        let ten = ArchSimd::splat(NumCast::from(10.0).unwrap());
        let neg_fifteen = ArchSimd::splat(NumCast::from(-15.0).unwrap());

        self.iter()
            .map(|t| t * t * t * t.mul_add(t.mul_add(six, neg_fifteen), ten))
            .collect()
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Iterators ————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

pub struct SimdVecIter<'a, T: SimdElement> {
    array: &'a SimdVec<T>,
    index: usize,
}

impl<'a, T: SimdElement> Iterator for SimdVecIter<'a, T> {
    type Item = ArchSimd<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Add partial load here for tail, so tail has 0's instead of uninitialized memory.
        if self.index < self.array.len() {
            let item = unsafe { self.array.load_simd_unchecked(self.index) };
            self.index += ArchSimd::<T>::LANES;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining =
            (self.array.len() - self.index + ArchSimd::<T>::LANES - 1) / ArchSimd::<T>::LANES;
        (remaining, Some(remaining))
    }
}

impl<T: SimdElement> FromIterator<ArchSimd<T>> for SimdVec<T> {
    fn from_iter<I: IntoIterator<Item = ArchSimd<T>>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();

        // TODO: Handle this.
        if upper.is_none() || lower != upper.unwrap() {
            panic!("ArchSimd<T> Iterators without known size are not yet supported!");
        }

        let size = upper.unwrap() * ArchSimd::<T>::LANES;
        let mut result = Self::with_capacity(size);
        unsafe { result.set_len(size) };

        // Full chunks loop.
        for i in (0..size).step_by(ArchSimd::<T>::LANES) {
            unsafe {
                result.store_simd_unchecked(i, iter.next().unwrap());
            }
        }
        // iter.for_each(|x| result.push_simd(x));
        result
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Mut Iterator —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

pub struct SimdVecIterMut<'a, T: SimdElement> {
    array: &'a mut SimdVec<T>,
    index: usize,
    tail_start: usize,
    tail_size: usize,
}

impl<'a, T: SimdElement> Iterator for SimdVecIterMut<'a, T> {
    type Item = SimdVecChunk<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.len() {
            let ptr = self.array as *mut SimdVec<T>;
            let item = SimdVecChunk::new(
                unsafe { &mut *ptr },
                self.index,
                self.tail_start,
                self.tail_size,
            );
            self.index += ArchSimd::<T>::LANES;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining =
            (self.array.len() - self.index + ArchSimd::<T>::LANES - 1) / ArchSimd::<T>::LANES;
        (remaining, Some(remaining))
    }
}

pub struct SimdVecChunk<'a, T: SimdElement> {
    register: ArchSimd<T>,
    array: &'a mut SimdVec<T>,
    pub index: usize,
    tail_start: usize,
    tail_size: usize,
}

impl<'a, T: SimdElement> SimdVecChunk<'a, T> {
    pub fn new(
        array: &'a mut SimdVec<T>,
        index: usize,
        tail_start: usize,
        tail_size: usize,
    ) -> Self {
        Self {
            register: unsafe { array.load_simd_unchecked(index) },
            array,
            index,
            tail_start,
            tail_size,
        }
    }
}

impl<'a, T: SimdElement> Drop for SimdVecChunk<'a, T> {
    fn drop(&mut self) {
        unsafe {
            if self.index < self.tail_start {
                // Normal chunk case.
                self.array.store_simd_unchecked(self.index, self.register);
            } else {
                // Tail chunk case.
                self.array
                    .partial_store_simd_unchecked(self.tail_start, self.register, self.tail_size);
            }
        }
    }
}

impl<'a, T: SimdElement> Deref for SimdVecChunk<'a, T> {
    type Target = ArchSimd<T>;
    fn deref(&self) -> &Self::Target {
        &self.register
    }
}

impl<'a, T: SimdElement> DerefMut for SimdVecChunk<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.register
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Into Iterator ————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

pub struct SimdVecIntoIter<T: SimdElement> {
    array: SimdVec<T>,
    index: usize,
}

impl<T: SimdElement> SimdVecIntoIter<T> {
    pub fn new(array: SimdVec<T>) -> Self {
        Self { array, index: 0 }
    }
}

impl<T: SimdElement> Iterator for SimdVecIntoIter<T> {
    type Item = ArchSimd<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.len() {
            let item = self.array.load_simd(self.index);
            self.index += ArchSimd::<T>::LANES;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining =
            (self.array.len() - self.index + ArchSimd::<T>::LANES - 1) / ArchSimd::<T>::LANES;
        (remaining, Some(remaining))
    }
}

impl<T: SimdElement> IntoIterator for SimdVec<T> {
    type Item = ArchSimd<T>;
    type IntoIter = SimdVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        SimdVecIntoIter::new(self)
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Iterator API —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement> SimdVec<T> {
    /// Returns an immutable iterator that automatically handles
    /// the tail.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    ///
    /// let arr = SimdVec::<f32, 35>::iota_custom(0.0, 0.4);
    /// let scale = ArchSimd::splat(2.0);
    ///
    /// let new_arr: SimdVec::<f32, 35> =
    ///     arr.iter()
    ///        .map(|x| x * scale)
    ///        .collect();
    ///
    /// assert_eq!(new_arr[2], 1.6);
    /// ```
    pub fn iter(&self) -> SimdVecIter<'_, T> {
        SimdVecIter {
            array: self,
            index: 0,
        }
    }

    /// Returns a mutable iterator that automatically handles
    /// the tail.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdVec;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    ///
    /// let mut arr = SimdVec::<f32, 35>::iota_custom(0.0, 0.4);
    /// let scale = ArchSimd::splat(2.0);
    ///
    /// arr.iter_mut()
    ///    .for_each(|mut x| *x *= scale);
    ///
    /// assert_eq!(arr[2], 1.6);
    /// ```
    pub fn iter_mut(&mut self) -> SimdVecIterMut<'_, T> {
        let tail_size = self.len() % ArchSimd::<T>::LANES;
        let tail_start = self.len() - tail_size;
        SimdVecIterMut {
            array: self,
            index: 0,
            tail_start,
            tail_size,
        }
    }
}

use crate::simd::arch_simd::{ArchMask, ArchSimd};
use crate::simd::simd_array::fmt::Debug;
use crate::simd::simd_traits::*;
use crate::simd::traits::{SimdElement, SimdFloat};
use itertools::izip;
use num_traits::NumCast;
use std::fmt;
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
/// use quick_noise::simd::simd_array::SimdArray;
/// let arr = SimdArray::<f32, 8>::new(1.0);
/// assert_eq!(arr[0], 1.0);
/// ```
#[repr(align(64))]
#[derive(Copy, Clone)]
pub struct SimdArray<T: SimdElement, const N: usize> {
    /// Internal storage using `MaybeUninit` for uninitialized elements.
    pub data: [MaybeUninit<T>; N],
}

// ————————————————————————————————————————————————————————————————
// ————— Tail Info ————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// Provides compile-time tail information for SIMD arrays based on lane count.
///
/// This trait is automatically implemented for [`SimdArray`] and [`SimdArrayChunk`],
/// providing compile-time constants useful for handling the tail of an array that
/// does not fit neatly in a single SIMD register. It can enable the avoidance of
/// runtime costs (such as checking every iteration if it is the tail).
///
/// NOTE: Consider using `iter()` and `iter_mut` for automatic performant tail handling.
///
/// # Constants
/// * `TAIL_SIZE` - Number of elements in the final partial chunk (0 if exact fit)
/// * `TAIL_START` - Starting index of the tail chunk
/// * `HAS_TAIL` - Whether the array has a partial tail chunk
///
/// # Example
///
/// ```
/// use quick_noise::simd::simd_array::SimdArray;
/// use quick_noise::simd::simd_array::TailInfo;
/// use quick_noise::simd::arch_simd::ArchSimd;
///
/// // Creates an iota array [0, 1, 2, 3 ... 9, 10];
/// // Size 11 has awkward tail, cannot divide evenly by any power of 2.
/// let mut arr = SimdArray::<i32, 11>::iota(0);
///
/// // Handle full chunks first.
/// for i in (0..SimdArray::<i32, 11>::TAIL_START).step_by(ArchSimd::<i32>::LANES) {
///     let register = arr.load_simd(i);
///     let new_register = register * ArchSimd::splat(2);
///     arr.store_simd(i, new_register);
/// }
///    
/// // Now handle tail if it exists. Folded at compile-time.
/// if SimdArray::<i32, 11>::HAS_TAIL {
///     let register = arr.load_simd(SimdArray::<i32, 11>::TAIL_START);
///     let new_register = register * ArchSimd::splat(2);
///     // Store only the tail.
///     arr.partial_store_simd(
///         SimdArray::<i32, 11>::TAIL_START,
///         new_register,
///         SimdArray::<i32, 11>::TAIL_SIZE
///     );
/// }
///
/// assert_eq!(arr[10], 20);
/// ```
pub trait TailInfo {
    const TAIL_SIZE: usize;
    const TAIL_START: usize;
    const HAS_TAIL: bool;
}

impl<T: SimdElement, const N: usize> TailInfo for SimdArray<T, N> {
    /// Number of elements that are 'leftover' and cannot fit fully in a simd register.
    const TAIL_SIZE: usize = N % ArchSimd::<T>::LANES;

    /// Starting index of the tail (0 if N < LANES).
    const TAIL_START: usize = if Self::TAIL_SIZE == N {
        0 // Set to zero if the entire array is a tail.
    } else {
        N - Self::TAIL_SIZE
    };
    const HAS_TAIL: bool = Self::TAIL_SIZE > 0;
}

// ————————————————————————————————————————————————————————————————
// ————— Constructors —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    /// Creates a new simd array with uninitialized memory.
    /// Ideal for avoiding initialization overhead in performance-critical code.
    pub fn new_uninit() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    /// Creates a new simd array and initializes all elements to a given value.
    ///
    /// # Parameters
    /// * `value` - The value to fill every element of the array with.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// // Becomes [4.0, 4.0, 4.0, 4.0, 4.0]
    /// let arr = SimdArray::<f32, 5>::new(4.0);
    /// assert_eq!(arr[0], 4.0);
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            data: [MaybeUninit::new(value); N],
        }
    }
}

impl<T: SimdElement, const N: usize> Default for SimdArray<T, N> {
    /// Creates a new simd array and initializes every element to
    /// the default value of the array's type.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// // Becomes [0.0, 0.0, 0.0, 0.0, 0.0]
    /// let arr = SimdArray::<f32, 5>::default();
    /// assert_eq!(arr[0], 0.0);
    /// ```
    fn default() -> Self {
        Self::new(T::default())
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Indexing —————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————
impl<T: SimdElement, const N: usize> Index<usize> for SimdArray<T, N> {
    type Output = T;

    /// Obtains the value at a given index.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// let arr = SimdArray::<i32, 5>::iota(0);
    /// assert_eq!(arr[2], 2);
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < N);
        unsafe { &self.data[index].assume_init_ref() }
    }
}

impl<T: SimdElement, const N: usize> IndexMut<usize> for SimdArray<T, N> {
    /// Obtains the mutable reference of a given index.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// let mut arr = SimdArray::<i32, 5>::new(0);
    /// arr[0] = 100;
    /// assert_eq!(arr[0], 100);
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < N);
        unsafe { self.data[index].assume_init_mut() }
    }
}

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    /// Obtains the value at a given index without bounds checking.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// let arr = SimdArray::<i32, 5>::iota(0);
    /// assert_eq!(unsafe { arr.get_unchecked(2) }, 2);
    /// ```
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> T {
        debug_assert!(index < N);
        unsafe { *self.data.get_unchecked(index).assume_init_ref() }
    }

    /// Obtains the mutable reference of a given index without bounds checking.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// let mut arr = SimdArray::<i32, 5>::new(0);
    /// unsafe { *arr.get_unchecked_mut(0) = 100; }
    /// assert_eq!(arr[0], 100);
    /// ```
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < N);
        unsafe { self.data.get_unchecked_mut(index).assume_init_mut() }
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Utility Traits ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement + fmt::Debug, const N: usize> fmt::Debug for SimdArray<T, N> {
    /// Prints the values of the [`SimdArray`] with debug formatting.`.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// let arr = SimdArray::<i32, 4>::iota(0);
    /// println!("arr: {:?}", arr);
    /// // Prints "arr: [0, 1, 2, 3]".
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(unsafe { self.data.assume_init_ref() })
            .finish()
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Simd Access ——————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

// TODO: Add partial load.
impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    /// Returns a simd register containing `ArchSimd::LANES` number
    /// of values starting from a given index.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    /// use quick_noise::simd::simd_traits::SimdToArray; // TODO: Consolidate these.
    ///
    /// let arr = SimdArray::<i32, 12>::iota(0);
    /// let register = arr.load_simd(0);
    /// assert_eq!(register.to_array()[3], 3);
    /// ```
    #[inline(always)]
    pub fn load_simd(&self, index: usize) -> ArchSimd<T> {
        // debug_assert!(index + ArchSimd::<T>::LANES <= N);
        // debug_assert!(index % ArchSimd::<T>::LANES == 0);
        unsafe { ArchSimd::load(&self.data.assume_init_ref().get_unchecked(index..)) }
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
    /// use quick_noise::simd::simd_array::SimdArray;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    ///
    /// let mut arr = SimdArray::<i32, 12>::iota(0);
    /// let register = ArchSimd::<i32>::splat(100);
    /// arr.store_simd(0, register);
    /// assert_eq!(arr[0], 100);
    /// ```
    #[inline(always)]
    pub fn store_simd(&mut self, index: usize, vec: ArchSimd<T>) {
        debug_assert!(index + ArchSimd::<T>::LANES <= N);
        // debug_assert!(index % ArchSimd::<T>::LANES == 0);
        unsafe {
            vec.store(&mut self.data.assume_init_mut().get_unchecked_mut(index..));
        }
    }

    /// Stores the first `amount` values in a SIMD register to a [`SimdArray`]
    /// starting from a given index.
    ///
    /// # Parameters
    /// * `index` - Starting index
    /// * `vec` - SIMD vector containing the values to store
    /// * `amount` - Number of elements to store from the SIMD vector
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    ///
    /// let mut arr = SimdArray::<i32, 10>::iota(0);
    /// let register = ArchSimd::<i32>::splat(100);
    /// arr.partial_store_simd(8, register, 2);
    /// assert_eq!(arr[9], 100);
    /// ```
    #[inline(always)]
    pub fn partial_store_simd(&mut self, index: usize, vec: ArchSimd<T>, amount: usize) {
        debug_assert!(index + amount <= N);
        unsafe {
            vec.partial_store(
                &mut self.data.assume_init_mut().get_unchecked_mut(index..),
                amount,
            );
        }
    }

    /// Stores values from a SIMD register based on a mask to a [`SimdArray`]
    /// starting from a given index.
    ///
    /// # Parameters
    /// * `index` - Starting index
    /// * `vec` - SIMD vector containing the values to store
    /// * `mask` - SIMD mask specifying which lanes to store from the SIMD vector
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// use quick_noise::simd::arch_simd::{ArchSimd, ArchMask};
    /// use quick_noise::simd::simd_traits::{SimdIota, SimdPartialOrd};
    ///
    /// let iota = ArchSimd::<i32>::iota(0);
    /// let splat = ArchSimd::<i32>::splat(1);
    ///
    /// // First two lanes false, rest true.
    /// let mask = iota.simd_gt(splat);
    ///
    /// let mut arr = SimdArray::<i32, 12>::new(0);
    /// let register = ArchSimd::<i32>::splat(100);
    /// arr.masked_store_simd(0, register, mask);
    /// assert_eq!(arr[0], 0);
    /// assert_eq!(arr[2], 100);
    /// ```
    #[inline(always)]
    pub fn masked_store_simd(&mut self, index: usize, vec: ArchSimd<T>, mask: ArchMask<T>) {
        debug_assert!(index < N);
        unsafe {
            vec.masked_store(
                &mut self.data.assume_init_mut().get_unchecked_mut(index..),
                mask,
            );
        }
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Basic Operator Imeplementations ——————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement, const N: usize> Add for SimdArray<T, N> {
    type Output = SimdArray<T, N>;
    fn add(self, rhs: Self) -> SimdArray<T, N> {
        izip!(self.iter(), rhs.iter()).map(|(x, y)| x + y).collect()
    }
}

impl<T: SimdElement, const N: usize> Sub for SimdArray<T, N> {
    type Output = SimdArray<T, N>;
    fn sub(self, rhs: Self) -> SimdArray<T, N> {
        izip!(self.iter(), rhs.iter()).map(|(x, y)| x - y).collect()
    }
}

// TODO: Flesh out what types can multiply and divide, and ensure where clause won't be needed.
impl<T: SimdElement, const N: usize> Mul for SimdArray<T, N>
where
    ArchSimd<T>: Mul<Output = ArchSimd<T>>,
{
    type Output = SimdArray<T, N>;
    fn mul(self, rhs: Self) -> SimdArray<T, N> {
        izip!(self.iter(), rhs.iter()).map(|(x, y)| x * y).collect()
    }
}

impl<T: SimdElement, const N: usize> Div for SimdArray<T, N>
where
    ArchSimd<T>: Div<Output = ArchSimd<T>>,
{
    type Output = SimdArray<T, N>;
    fn div(self, rhs: Self) -> SimdArray<T, N> {
        izip!(self.iter(), rhs.iter()).map(|(x, y)| x / y).collect()
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Basic Assign Ops —————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement, const N: usize> AddAssign for SimdArray<T, N> {
    fn add_assign(&mut self, rhs: Self) {
        izip!(self.iter_mut(), rhs.iter())
            .map(|(mut x, y)| *x += y)
            .collect()
    }
}

impl<T: SimdElement, const N: usize> SubAssign for SimdArray<T, N> {
    fn sub_assign(&mut self, rhs: Self) {
        izip!(self.iter_mut(), rhs.iter())
            .map(|(mut x, y)| *x -= y)
            .collect()
    }
}

impl<T: SimdElement, const N: usize> MulAssign for SimdArray<T, N>
where
    ArchSimd<T>: MulAssign,
{
    fn mul_assign(&mut self, rhs: Self) {
        izip!(self.iter_mut(), rhs.iter())
            .map(|(mut x, y)| *x *= y)
            .collect()
    }
}

impl<T: SimdElement, const N: usize> DivAssign for SimdArray<T, N>
where
    ArchSimd<T>: DivAssign,
{
    fn div_assign(&mut self, rhs: Self) {
        izip!(self.iter_mut(), rhs.iter())
            .map(|(mut x, y)| *x /= y)
            .collect()
    }
}

impl<T: SimdElement, const N: usize> Neg for SimdArray<T, N> {
    type Output = SimdArray<T, N>;
    fn neg(self) -> SimdArray<T, N> {
        self.iter().map(|x| -x).collect()
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Additional Operations ————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    /// Takes a list of value array pairs and broadcasts that value in
    /// each array a specified amount of elements starting at a specified
    /// index.
    ///
    /// # Parameters
    /// * `arrays` - an array of mutable [`SimdArray`]'s
    /// * 'values' - an array of values to set into each corresponding array
    /// * 'index' - the index to start setting values at in all arrays
    /// * 'amount' - the amount of elements to fill with the specified value
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// let mut arr1 = SimdArray::<f32, 32>::new(10.0);
    /// let mut arr2 = SimdArray::<f32, 32>::new(0.0);
    ///
    /// let mut arrays = [&mut arr1, &mut arr2];
    /// let values = [2.0, 4.0];
    ///
    /// SimdArray::multiset_many(&mut arrays, &values, 2, 10);
    ///
    /// assert_eq!(arr1[2], 2.0);
    /// assert_eq!(arr2[2], 4.0);
    /// ```
    #[inline(always)]
    pub fn multiset_many<const M: usize>(
        arrays: &mut [&mut Self; M],
        values: &[T; M],
        mut index: usize,
        mut amount: isize,
    ) {
        let vecs: [ArchSimd<T>; M] = std::array::from_fn(|i| ArchSimd::<T>::splat(values[i]));

        while amount > 0 {
            // Full register store case.
            if index < N - ArchSimd::<T>::LANES {
                for i in 0..M {
                    unsafe {
                        arrays[i].store_simd(index, *vecs.get_unchecked(i));
                    }
                }
            } else {
                // Tail store case.
                let iota = ArchSimd::<i32>::iota(N as i32 - ArchSimd::<T>::LANES as i32);
                let indices = ArchSimd::splat(index as i32);
                let mask = iota.simd_ge(indices);
                let tail_index = N - ArchSimd::<T>::LANES;

                for i in 0..M {
                    unsafe {
                        arrays[i].masked_store_simd(
                            tail_index,
                            *vecs.get_unchecked(i),
                            mask.raw_cast(),
                        );
                    }
                }
            }

            amount -= ArchSimd::<T>::LANES as isize;
            index += ArchSimd::<T>::LANES;
        }
    }
}

impl<T: SimdElement + NumCast + Debug, const N: usize> SimdArray<T, N>
where
    T: Mul<Output = T>,
    ArchSimd<T>: Mul<Output = ArchSimd<T>>,
{
    /// Creates a [`SimdArray`] according to a given linear function,
    /// starting at a specified value at index 0 and incrementing each
    /// subsequent element by a specific increment.
    ///
    /// # Parameters
    /// * `offset` - The starting value
    /// * `increment`` - The amount increased by every subsequent element
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// // Becomes [10.0, 10.5, 11.0, 11.5]
    /// let arr = SimdArray::<f32, 4>::iota_custom(10.0, 0.5);
    ///
    /// assert_eq!(arr[2], 11.0);
    /// ```
    pub fn iota_custom(offset: T, increment: T) -> Self {
        // Set the base iota vec, and then repeatedly add an increment each iteration.
        let increment_vec = ArchSimd::splat(increment);
        let lanes_increment_vec =
            ArchSimd::splat(increment * NumCast::from(ArchSimd::<T>::LANES).unwrap());
        let iota_vec = ArchSimd::iota(NumCast::from(0).unwrap()) * increment_vec;

        let mut cur_vec = ArchSimd::splat(offset) + iota_vec;
        let mut result = Self::new_uninit();
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

    /// Creates a [`SimdArray`] starting at a specific offset at index
    /// 0, and incrementing each subsequent element by 1.
    ///
    /// # Parameters
    /// * `offset` - The starting value
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// // Becomes [2, 3, 4, 5]
    /// let arr = SimdArray::<i32, 4>::iota(2);
    ///
    /// assert_eq!(arr[2], 4);
    pub fn iota(offset: T) -> Self {
        // Set the base iota vec, and then repeatedly add an increment each iteration.
        let lanes_increment_vec = ArchSimd::splat(NumCast::from(ArchSimd::<T>::LANES).unwrap());
        let mut cur_vec = ArchSimd::iota(offset);

        let mut result = Self::new_uninit();
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

impl<T: SimdFloat, const N: usize> SimdArray<T, N> {
    /// Creates a new [`SimdArray`] of floating point values that truncates
    /// the whole number portion off an existing [`SimdArray`], leaving
    /// only the decimal behind.
    ///
    /// Only available when `T` implements [`SimdFloat`].
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// let arr = SimdArray::<f32, 4>::new(14.5);
    /// let new_arr = arr.fract();
    ///
    /// assert_eq!(new_arr[0], 0.5);
    pub fn fract(&self) -> Self {
        self.iter()
            .map(|x| x.fract())
            .collect()
    }

    // TODO: extend max and min to integer types.
    /// Creates a new [`SimdArray`] containing the values of another
    /// [`SimdArray`] unless it is less than a specified value, in that case
    /// it becomes that value.
    ///
    /// Only available when `T` implements [`SimdFloat`].
    ///
    /// # Parameters
    /// * `val` - The value that is compared to every element in the array
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// let arr = SimdArray::<f32, 4>::new(14.5);
    /// let max_arr = arr.max(15.0);
    ///
    /// assert_eq!(max_arr[0], 15.0);
    pub fn max(&self, val: T) -> Self {
        let max_vec = ArchSimd::splat(val);
        self.iter()
            .map(|x| x.max(max_vec))
            .collect()
    }

    /// Creates a new [`SimdArray`] containing the values of another
    /// [`SimdArray`] unless it is greater than a specified value, in that case
    /// it becomes that value.
    ///
    /// Only available when `T` implements [`SimdFloat`].
    ///
    /// # Parameters
    /// * `val` - The value that is compared to every element in the array
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// let arr = SimdArray::<f32, 4>::new(14.5);
    /// let min_arr = arr.min(15.0);
    ///
    /// assert_eq!(min_arr[0], 14.5);
    pub fn min(&self, val: T) -> Self {
        let min_vec = ArchSimd::splat(val);
        self.iter()
            .map(|x| x.min(min_vec))
            .collect()
    }
}

impl<T: SimdFloat, const N: usize> SimdArray<T, N> {
    /// Creates a new [`SimdArray`] containing the product of two
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
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// let a = SimdArray::<f32, 4>::new(2.0);
    /// let b = SimdArray::<f32, 4>::new(4.0);
    /// let c = SimdArray::<f32, 4>::new(6.0);
    /// let result = a.mul_add(b, c);
    ///
    /// assert_eq!(result[0], 14.0);
    /// ```
    pub fn mul_add(self, mult: Self, offset: Self) -> Self {
        izip!(self.iter(), mult.iter(), offset.iter())
            .map(|(a, b, c)| a.mul_add(b, c))
            .collect()
    }

    /// Creates a new [`SimdArray`] containing the product of two
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
    /// use quick_noise::simd::simd_array::SimdArray;
    ///
    /// let a = SimdArray::<f32, 4>::new(2.0);
    /// let b = SimdArray::<f32, 4>::new(4.0);
    /// let c = SimdArray::<f32, 4>::new(6.0);
    /// let result = a.mul_sub(b, c);
    ///
    /// assert_eq!(result[0], 2.0);
    /// ```
    pub fn mul_sub(self, mult: Self, offset: Self) -> Self {
        izip!(self.iter(), mult.iter(), offset.iter())
            .map(|(a, b, c)| a.mul_sub(b, c))
            .collect()
    }
}

impl<T: SimdFloat, const N: usize> SimdArray<T, N> {
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

pub struct SimdArrayIter<'a, T: SimdElement, const N: usize> {
    array: &'a SimdArray<T, N>,
    index: usize,
}

impl<'a, T: SimdElement, const N: usize> TailInfo for SimdArrayIter<'a, T, N> {
    const TAIL_SIZE: usize = N % ArchSimd::<T>::LANES;
    const TAIL_START: usize = if Self::TAIL_SIZE == N {
        0
    } else {
        N - Self::TAIL_SIZE
    };
    const HAS_TAIL: bool = Self::TAIL_SIZE > 0;
}

impl<'a, T: SimdElement, const N: usize> Iterator for SimdArrayIter<'a, T, N> {
    type Item = ArchSimd<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Add partial load here for tail, so tail has 0's instead of uninitialized memory.
        if self.index < N {
            let item = self.array.load_simd(self.index);
            self.index += ArchSimd::<T>::LANES;
            Some(item)
        } else {
            None
        }
    }
}

impl<T: SimdElement, const N: usize> FromIterator<ArchSimd<T>> for SimdArray<T, N> {
    fn from_iter<I: IntoIterator<Item = ArchSimd<T>>>(iter: I) -> Self {
        let mut result = Self::new_uninit();

        let mut cur_index = 0;
        for chunk in iter {
            // Handle tail case, use compile-constants to help compiler optimize.
            if Self::HAS_TAIL && (cur_index >= Self::TAIL_START) {
                result.partial_store_simd(Self::TAIL_START, chunk, Self::TAIL_SIZE)
            } else {
                result.store_simd(cur_index, chunk);
                cur_index += ArchSimd::<T>::LANES;
            }
        }

        result
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Mut Iterator —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

pub struct SimdArrayIterMut<'a, T: SimdElement, const N: usize> {
    array: &'a mut SimdArray<T, N>,
    index: usize,
}

impl<'a, T: SimdElement, const N: usize> Iterator for SimdArrayIterMut<'a, T, N> {
    type Item = SimdArrayChunk<'a, T, N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < N {
            let ptr = self.array as *mut SimdArray<T, N>;
            let item = SimdArrayChunk::new(unsafe { &mut *ptr }, self.index);
            self.index += ArchSimd::<T>::LANES;
            Some(item)
        } else {
            None
        }
    }
}

pub struct SimdArrayChunk<'a, T: SimdElement, const N: usize> {
    register: ArchSimd<T>,
    array: &'a mut SimdArray<T, N>,
    pub index: usize,
}

impl<'a, T: SimdElement, const N: usize> TailInfo for SimdArrayChunk<'a, T, N> {
    const TAIL_SIZE: usize = N % ArchSimd::<T>::LANES;
    const TAIL_START: usize = if Self::TAIL_SIZE == N {
        0
    } else {
        N - Self::TAIL_SIZE
    };
    const HAS_TAIL: bool = Self::TAIL_SIZE > 0;
}

impl<'a, T: SimdElement, const N: usize> SimdArrayChunk<'a, T, N> {
    pub fn new(array: &'a mut SimdArray<T, N>, index: usize) -> Self {
        Self {
            register: array.load_simd(index),
            array,
            index,
        }
    }
}

impl<'a, T: SimdElement, const N: usize> Drop for SimdArrayChunk<'a, T, N> {
    fn drop(&mut self) {
        if self.index < Self::TAIL_START {
            // Normal chunk case.
            self.array.store_simd(self.index, self.register);
        } else {
            // Tail chunk case.
            self.array
                .partial_store_simd(Self::TAIL_START, self.register, Self::TAIL_SIZE);
        }
    }
}

impl<'a, T: SimdElement, const N: usize> Deref for SimdArrayChunk<'a, T, N> {
    type Target = ArchSimd<T>;
    fn deref(&self) -> &Self::Target {
        &self.register
    }
}

impl<'a, T: SimdElement, const N: usize> DerefMut for SimdArrayChunk<'a, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.register
    }
}

// ————————————————————————————————————————————————————————————————
// ————— Iterator API —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<T: SimdElement, const N: usize> SimdArray<T, N> {
    /// Returns an immutable iterator that automatically handles
    /// the tail.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    ///
    /// let arr = SimdArray::<f32, 35>::iota_custom(0.0, 0.4);
    /// let scale = ArchSimd::splat(2.0);
    ///
    /// let new_arr: SimdArray::<f32, 35> =
    ///     arr.iter()
    ///        .map(|x| x * scale)
    ///        .collect();
    ///
    /// assert_eq!(new_arr[2], 1.6);
    /// ```
    pub fn iter(&self) -> SimdArrayIter<'_, T, N> {
        SimdArrayIter {
            array: self,
            index: 0,
        }
    }

    /// Returns a mutable iterator that automatically handles
    /// the tail.
    ///
    /// # Example
    /// ```
    /// use quick_noise::simd::simd_array::SimdArray;
    /// use quick_noise::simd::arch_simd::ArchSimd;
    ///
    /// let mut arr = SimdArray::<f32, 35>::iota_custom(0.0, 0.4);
    /// let scale = ArchSimd::splat(2.0);
    ///
    /// arr.iter_mut().for_each(|mut x| *x *= scale);
    ///
    /// assert_eq!(arr[2], 1.6);
    /// ```
    pub fn iter_mut(&mut self) -> SimdArrayIterMut<'_, T, N> {
        SimdArrayIterMut {
            array: self,
            index: 0,
        }
    }
}

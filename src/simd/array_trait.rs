use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::ops::{Index, IndexMut};
// use std::hash::Hash;

// This is unfortunately needed to avoid generic const expr or excessive bounding issues. TODO: Come up with better solution.
pub trait Array<T>:
    Clone +
    Debug +
    // PartialEq +
    // Eq +
    // PartialOrd +
    Index<usize, Output = T> +
    IndexMut<usize, Output = T> +
    AsRef<[T]> +
    AsMut<[T]> +
    Borrow<[T]> +
    BorrowMut<[T]> +
    IntoIterator<Item = T> +
    Sized
{
    const LEN: usize;

    // ---- Construction ----
    fn from_fn(f: impl FnMut(usize) -> T) -> Self;
    fn zeroed() -> Self where T: Default { Self::from_fn(|_| T::default()) }

    // ---- Slice access ----
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];

    // ---- Element access ----
    fn get(&self, i: usize) -> Option<&T>                { self.as_slice().get(i) }
    fn get_mut(&mut self, i: usize) -> Option<&mut T>    { self.as_mut_slice().get_mut(i) }
    fn first(&self) -> Option<&T>                        { self.as_slice().first() }
    fn first_mut(&mut self) -> Option<&mut T>            { self.as_mut_slice().first_mut() }
    fn last(&self) -> Option<&T>                         { self.as_slice().last() }
    fn last_mut(&mut self) -> Option<&mut T>             { self.as_mut_slice().last_mut() }

    // ---- Info ----
    fn len(&self) -> usize                               { Self::LEN }
    fn is_empty(&self) -> bool                           { Self::LEN == 0 }

    // ---- Iteration ----
    fn iter(&self) -> std::slice::Iter<'_, T>            { self.as_slice().iter() }
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> { self.as_mut_slice().iter_mut() }
    fn windows(&self, size: usize) -> std::slice::Windows<'_, T>          { self.as_slice().windows(size) }
    fn chunks(&self, size: usize) -> std::slice::Chunks<'_, T>            { self.as_slice().chunks(size) }
    fn chunks_exact(&self, size: usize) -> std::slice::ChunksExact<'_, T> { self.as_slice().chunks_exact(size) }
    fn rchunks(&self, size: usize) -> std::slice::RChunks<'_, T>          { self.as_slice().rchunks(size) }
    fn rchunks_exact(&self, size: usize) -> std::slice::RChunksExact<'_, T> { self.as_slice().rchunks_exact(size) }

    // ---- Search ----
    fn contains(&self, x: &T) -> bool where T: PartialEq        { self.as_slice().contains(x) }
    fn starts_with(&self, needle: &[T]) -> bool where T: PartialEq { self.as_slice().starts_with(needle) }
    fn ends_with(&self, needle: &[T]) -> bool where T: PartialEq { self.as_slice().ends_with(needle) }
    fn binary_search(&self, x: &T) -> Result<usize, usize> where T: Ord { self.as_slice().binary_search(x) }
    fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
        where F: FnMut(&'a T) -> std::cmp::Ordering, T: 'a  { self.as_slice().binary_search_by(f) }

    fn binary_search_by_key<'a, B: Ord, F: FnMut(&'a T) -> B>(&'a self, b: &B, f: F) -> Result<usize, usize>
        where T: 'a                                          { self.as_slice().binary_search_by_key(b, f) }
    fn position(&self, f: impl FnMut(&T) -> bool) -> Option<usize> { self.as_slice().iter().position(f) }

    // ---- Mutation ----
    fn fill(&mut self, value: T) where T: Clone          { self.as_mut_slice().fill(value) }
    fn fill_with(&mut self, f: impl FnMut() -> T)        { self.as_mut_slice().fill_with(f) }
    fn swap(&mut self, a: usize, b: usize)               { self.as_mut_slice().swap(a, b) }
    fn reverse(&mut self)                                { self.as_mut_slice().reverse() }
    fn rotate_left(&mut self, mid: usize)                { self.as_mut_slice().rotate_left(mid) }
    fn rotate_right(&mut self, mid: usize)               { self.as_mut_slice().rotate_right(mid) }
    fn copy_from_slice(&mut self, src: &[T]) where T: Copy { self.as_mut_slice().copy_from_slice(src) }
    fn clone_from_slice(&mut self, src: &[T]) where T: Clone { self.as_mut_slice().clone_from_slice(src) }
    fn swap_with_slice(&mut self, other: &mut [T])       { self.as_mut_slice().swap_with_slice(other) }

    // ---- Sorting ----
    fn sort(&mut self) where T: Ord                      { self.as_mut_slice().sort() }
    fn sort_by(&mut self, f: impl FnMut(&T, &T) -> std::cmp::Ordering) { self.as_mut_slice().sort_by(f) }
    fn sort_by_key<K: Ord>(&mut self, f: impl FnMut(&T) -> K) { self.as_mut_slice().sort_by_key(f) }
    fn sort_unstable(&mut self) where T: Ord             { self.as_mut_slice().sort_unstable() }
    fn sort_unstable_by(&mut self, f: impl FnMut(&T, &T) -> std::cmp::Ordering) { self.as_mut_slice().sort_unstable_by(f) }
    fn sort_unstable_by_key<K: Ord>(&mut self, f: impl FnMut(&T) -> K) { self.as_mut_slice().sort_unstable_by_key(f) }
    fn is_sorted(&self) -> bool where T: PartialOrd      { self.as_slice().is_sorted() }
    fn is_sorted_by(&self, f: impl FnMut(&T, &T) -> bool) -> bool { self.as_slice().is_sorted_by(f) }
    fn is_sorted_by_key<K: PartialOrd>(&self, f: impl FnMut(&T) -> K) -> bool { self.as_slice().is_sorted_by_key(f) }

    // ---- Pointers ----
    fn as_ptr(&self) -> *const T { self.as_slice().as_ptr() }
    fn as_mut_ptr(&mut self) -> *mut T { self.as_mut_slice().as_mut_ptr() }
}

impl<const N: usize, T> Array<T> for [T; N]
where
    T: Clone + Debug,
{
    const LEN: usize = N;
    #[inline(always)]
    fn from_fn(f: impl FnMut(usize) -> T) -> Self {
        std::array::from_fn(f)
    }
    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }
    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}


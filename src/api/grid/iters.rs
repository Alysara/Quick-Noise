use crate::Grid3D;
use crate::api::grid::interface::Grid2D;
use crate::simd::arch_simd::{ArchMask, ArchSimd};
use crate::simd::simd_traits::*;

impl<const X: usize, const Y: usize, const N: usize> Grid2D<X, Y, N> {
    #[inline(always)]
    pub fn x_iter(&self) -> RowIter {
        let start = (self.config.position.x * X as i32) as f32;
        RowIter::new(X, Y, start)
    }

    #[inline(always)]
    pub fn y_iter(&self) -> SliceIter {
        let start = (self.config.position.y * Y as i32) as f32;
        SliceIter::new(X, Y, 1, start)
    }
}

impl<const X: usize, const Y: usize, const Z: usize, const N: usize> Grid3D<X, Y, Z, N> {
    #[inline(always)]
    pub fn x_iter(&self) -> SliceIter {
        let start = (self.config.position.x * X as i32) as f32;
        SliceIter::new(Z * Y, X, 1, start)
    }

    #[inline(always)]
    pub fn y_iter(&self) -> SliceIter {
        let start = (self.config.position.y * Y as i32) as f32;
        SliceIter::new(Z, Y, X, start)
    }

    #[inline(always)]
    pub fn z_iter(&self) -> RowIter {
        let start = (self.config.position.z * Z as i32) as f32;
        RowIter::new(Z, Y * Z, start)
    }
}

const LANES: usize = ArchSimd::<f32>::LANES;

pub struct RowIter {
    row_size: usize,
    left_in_row: usize,
    rows_left: usize,
    cur_vec: ArchSimd<f32>,
    start_vec: ArchSimd<f32>,
}

impl RowIter {
    fn new(row_size: usize, num_rows: usize, start_val: f32) -> Self {
        Self {
            row_size,
            left_in_row: row_size,
            rows_left: num_rows,
            cur_vec: ArchSimd::iota(start_val - LANES as f32),
            start_vec: ArchSimd::iota(start_val),
        }
    }
}

impl Iterator for RowIter {
    type Item = ArchSimd<f32>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Full columns.
        if self.left_in_row >= LANES {
            self.left_in_row -= LANES;
            self.cur_vec += ArchSimd::splat(LANES as f32);
            return Some(self.cur_vec);
        }

        // Partial/Transition columns.
        if self.rows_left > 0 {
            let mask = ArchMask::first_n_true(self.left_in_row as u32);
            let old = self.cur_vec + ArchSimd::splat(LANES as f32);
            self.cur_vec = self.start_vec - ArchSimd::splat(self.left_in_row as f32);
            self.left_in_row += self.row_size - LANES;
            self.rows_left -= 1;
            return Some(mask.select(old, self.cur_vec));
        }

        // Tail.
        if self.left_in_row > 0 {
            let mask = ArchMask::first_n_true(self.left_in_row as u32);
            let vec = self.cur_vec + ArchSimd::splat(LANES as f32);
            self.rows_left = 0;
            return Some(mask.select(vec, ArchSimd::zero()));
        }

        // Finished iter.
        None
    }
}

pub struct SliceIter {
    row_size: usize,
    slice_size: usize,
    left_in_row: usize,
    left_in_slice: usize,
    slices_left: usize,
    cur_val: f32,
    start_val: f32,
}

impl SliceIter {
    pub fn new(row_size: usize, slice_size: usize, num_slices: usize, start_val: f32) -> Self {
        Self {
            row_size,
            slice_size,
            left_in_row: row_size,
            left_in_slice: slice_size,
            slices_left: num_slices - 1,
            cur_val: start_val,
            start_val,
        }
    }
}

impl Iterator for SliceIter {
    type Item = ArchSimd<f32>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Full rows.
        if self.left_in_row >= LANES {
            self.left_in_row -= LANES;
            return Some(ArchSimd::splat(self.cur_val));
        }

        if self.left_in_slice > 0 {
            let old = ArchSimd::splat(self.cur_val);
            self.cur_val += 1.0;
            let new = ArchSimd::splat(self.cur_val);

            let mask = ArchMask::first_n_true(self.left_in_row as u32);
            self.left_in_row += self.row_size - LANES;
            self.left_in_slice -= 1;
            return Some(mask.select(old, new));
        }

        if self.slices_left > 0 {
            let old = ArchSimd::splat(self.cur_val);
            let new = ArchSimd::splat(self.start_val);
            self.cur_val = self.start_val;

            let mask = ArchMask::first_n_true(self.left_in_row as u32);
            self.left_in_row += self.row_size - LANES;
            self.left_in_slice = self.slice_size;
            self.slices_left -= 1;
            return Some(mask.select(old, new));
        }

        // Tail.
        if self.left_in_row > 0 {
            let old = ArchSimd::splat(self.cur_val);
            let mask = ArchMask::first_n_true(self.left_in_row as u32);
            self.left_in_row = 0;
            return Some(mask.select(old, ArchSimd::zero()));
        }

        // Finished iter.
        None
    }
}

use std::array::from_fn;

use crate::Grid;
use crate::simd::arch_simd::{ArchMask, ArchSimd};

impl Grid<2> {
    #[inline(always)]
    pub fn x_iter(&self) -> RowIter {
        let pos = self.config.position;
        let dim = self.config.grid_size;
        RowIter::new(dim[0], dim[1], pos[0] as f32)
    }

    #[inline(always)]
    pub fn y_iter(&self) -> SliceIter {
        let pos = self.config.position;
        let dim = self.config.grid_size;
        SliceIter::new(dim[0], dim[1], 1, pos[1] as f32)
    }
}

impl Grid<3> {
    #[inline(always)]
    pub fn x_iter(&self) -> RowIter {
        let pos = self.config.position;
        let dim = self.config.grid_size;
        RowIter::new(dim[0], dim[1] * dim[2], pos[0] as f32)
    }

    #[inline(always)]
    pub fn y_iter(&self) -> SliceIter {
        let pos = self.config.position;
        let dim = self.config.grid_size;
        SliceIter::new(dim[0], dim[1], dim[2], pos[1] as f32)
    }

    #[inline(always)]
    pub fn z_iter(&self) -> SliceIter {
        let pos = self.config.position;
        let dim = self.config.grid_size;
        SliceIter::new(dim[0] * dim[1], dim[2], 1, pos[2] as f32)
    }
}

const LANES: usize = ArchSimd::<f32>::LANES;

#[derive(Debug)]
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
            rows_left: num_rows - 1,
            cur_vec: ArchSimd::iota(start_val),
            start_vec: ArchSimd::iota(start_val),
        }
    }
}

impl Iterator for RowIter {
    type Item = ArchSimd<f32>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Scalar case.
        if self.row_size < LANES {
            if self.left_in_row == 0 && self.rows_left == 0 {
                return None;
            }

            let mut cur = self.cur_vec.to_array()[0];
            let start = self.start_vec.to_array()[0];

            let array: [f32; ArchSimd::<f32>::LANES] = from_fn(|_| {
                if self.left_in_row > 0 {
                    let next = cur;
                    cur += 1.0;
                    self.left_in_row -= 1;
                    next
                } else if self.rows_left > 0 {
                    cur = start + 1.0;
                    self.rows_left -= 1;
                    self.left_in_row = self.row_size - 1;
                    start
                } else {
                    0.0
                }
            });

            self.cur_vec = ArchSimd::iota(cur);
            return Some(ArchSimd::from_slice(array.as_slice()));
        }

        // Full columns.
        if self.left_in_row >= LANES {
            let next = self.cur_vec;
            self.cur_vec += ArchSimd::splat(LANES as f32);
            self.left_in_row -= LANES;
            return Some(next);
        }

        // Partial/Transition columns.
        if self.rows_left > 0 {
            let mask = ArchMask::first_n_true(self.left_in_row as u32);
            let old = self.cur_vec;
            let next = self.start_vec - ArchSimd::splat(self.left_in_row as f32);
            self.left_in_row += self.row_size - LANES;
            self.rows_left -= 1;
            self.cur_vec = next + ArchSimd::splat(LANES as f32);
            return Some(mask.select(old, next));
        }

        // Tail.
        if self.left_in_row > 0 {
            let mask = ArchMask::first_n_true(self.left_in_row as u32);
            let vec = self.cur_vec;
            self.left_in_row = 0;
            return Some(mask.select(vec, ArchSimd::zero()));
        }

        // Finished iter.
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.left_in_row + self.rows_left * self.row_size;
        let chunks_left = left.div_ceil(ArchSimd::<f32>::LANES);
        (chunks_left, Some(chunks_left))
    }
}

#[derive(Debug)]
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
            left_in_slice: slice_size - 1,
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
        // Scalar case.
        if self.row_size < LANES {
            if self.left_in_row == 0 && self.left_in_slice == 0 && self.slices_left == 0 {
                return None;
            }

            let array: [f32; ArchSimd::<f32>::LANES] = from_fn(|_| {
                if self.left_in_row > 0 {
                    self.left_in_row -= 1;
                    self.cur_val
                } else if self.left_in_slice > 0 {
                    self.cur_val += 1.0;
                    self.left_in_row = self.row_size - 1;
                    self.left_in_slice -= 1;
                    self.cur_val
                } else if self.slices_left > 0 {
                    self.cur_val = self.start_val;
                    self.left_in_row = self.row_size - 1;
                    self.left_in_slice = self.slice_size - 1;
                    self.slices_left -= 1;
                    self.start_val
                } else {
                    0.0
                }
            });
            return Some(ArchSimd::from_slice(array.as_slice()));
        }

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
            self.left_in_slice = self.slice_size - 1;
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left_in_slice = self.left_in_row + self.left_in_slice * self.row_size;
        let left_after_slice = self.slices_left * self.row_size * self.slice_size;
        let left = left_in_slice + left_after_slice;
        let chunks_left = left.div_ceil(ArchSimd::<f32>::LANES);
        (chunks_left, Some(chunks_left))
    }
}

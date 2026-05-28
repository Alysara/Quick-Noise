use crate::Grid3D;
use crate::api::grid::interface::Grid2D;
use crate::simd::arch_simd::{ArchMask, ArchSimd};
use crate::simd::simd_traits::*;

// ————————————————————————————————————————————————————————————————
// ————— 2D Grid Iters ————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<const X: usize, const Y: usize, const N: usize> Grid2D<X, Y, N> {
    #[inline(always)]
    pub fn x_iter(&self) -> NodeIter2DX<X, Y> {
        let start_vec = ArchSimd::iota((X as i32 * self.config.position.x) as f32);
        let cur_vec = start_vec - ArchSimd::splat(ArchSimd::<f32>::LANES as f32);
        NodeIter2DX {
            rows_left: Y,
            columns_left: X,
            start_vec,
            cur_vec,
        }
    }

    #[inline(always)]
    pub fn y_iter(&self) -> NodeIter2DY<X, Y> {
        NodeIter2DY {
            rows_left: Y,
            columns_left: X,
            cur_val: (Y as i32 * self.config.position.y) as f32,
        }
    }
}

impl<const X: usize, const Y: usize, const Z: usize, const N: usize> Grid3D<X, Y, Z, N> {
    #[inline(always)]
    pub fn x_iter(&self) -> NodeIter2DX<X, Y> {
        let start_vec = ArchSimd::iota((X as i32 * self.config.position.x) as f32);
        let cur_vec = start_vec - ArchSimd::splat(ArchSimd::<f32>::LANES as f32);
        NodeIter2DX {
            rows_left: Y * Z,
            columns_left: X,
            start_vec,
            cur_vec,
        }
    }

    #[inline(always)]
    pub fn y_iter(&self) -> NodeIter2DY<X, Y> {
        NodeIter2DY {
            rows_left: Y,
            columns_left: X,
            cur_val: (Y as i32 * self.config.position.y) as f32,
        }
    }
}


pub struct NodeIter2DX<const X: usize, const Y: usize> {
    rows_left: usize,
    columns_left: usize,
    cur_vec: ArchSimd<f32>,
    start_vec: ArchSimd<f32>,
}

impl<const X: usize, const Y: usize> NodeIter2DX<X, Y> {
    const LANES: usize = ArchSimd::<f32>::LANES;
}

impl<const X: usize, const Y: usize> Iterator for NodeIter2DX<X, Y> {
    type Item = ArchSimd<f32>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Full columns.
        if self.columns_left >= Self::LANES {
            self.columns_left -= Self::LANES;
            self.cur_vec += ArchSimd::splat(Self::LANES as f32);
            return Some(self.cur_vec);
        }

        // Partial/Transition columns.
        if self.rows_left > 0 {
            let mask = ArchMask::first_n_true(self.columns_left as u32);
            let old = self.cur_vec + ArchSimd::splat(Self::LANES as f32);
            let new = self.start_vec - ArchSimd::splat(self.columns_left as f32);
            self.cur_vec = new;
            self.columns_left = (X - Self::LANES) + self.columns_left;
            self.rows_left -= 1;
            return Some(mask.select(old, new));
        }

        // Tail.
        if self.columns_left > 0 {
            let mask = ArchMask::first_n_true(self.columns_left as u32);
            let vec = self.cur_vec + ArchSimd::splat(Self::LANES as f32);
            self.columns_left = 0;
            return Some(mask.select(vec, ArchSimd::zero()));
        }

        // Finished iter.
        None
    }
}

pub struct NodeIter2DY<const X: usize, const Y: usize> {
    rows_left: usize,
    columns_left: usize,
    cur_val: f32,
}

impl<const X: usize, const Y: usize> NodeIter2DY<X, Y> {
    const LANES: usize = ArchSimd::<f32>::LANES;
}

impl<const X: usize, const Y: usize> Iterator for NodeIter2DY<X, Y> {
    type Item = ArchSimd<f32>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Full rows.
        if self.rows_left > Self::LANES {
            self.rows_left -= Self::LANES;
            return Some(ArchSimd::splat(self.cur_val));
        }

        // Partial/Transition rows.
        if self.columns_left > 0 {
            let new_val = self.cur_val + 1.0;
            let mask = ArchMask::first_n_true(self.rows_left as u32);
            let old = ArchSimd::splat(self.cur_val);
            let new = ArchSimd::splat(new_val);
            self.cur_val = new_val;
            self.rows_left = (Y - Self::LANES) + self.rows_left;
            self.columns_left -= 1;
            return Some(mask.select(old, new));
        }

        // Tail.
        if self.rows_left > 0 {
            let mask = ArchMask::first_n_true(self.rows_left as u32);
            let vec = ArchSimd::splat(self.cur_val);
            self.rows_left = 0;
            return Some(mask.select(vec, ArchSimd::zero()));
        }

        // Finished iter.
        None
    }
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Grid Iters ————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————


// impl<const X: usize, const Y: usize, const N: usize> Grid2D<X, Y, N> {
//     #[inline(always)]
//     pub fn x_iter(&self) -> NodeIter2DX<X, Y> {
//         let start_vec = ArchSimd::iota((X as i32 * self.config.position.x) as f32);
//         let cur_vec = start_vec - ArchSimd::splat(ArchSimd::<f32>::LANES as f32);
//         NodeIter2DX {
//             rows_left: Y,
//             columns_left: X,
//             start_vec,
//             cur_vec,
//         }
//     }

//     #[inline(always)]
//     pub fn y_iter(&self) -> NodeIter2DY<X, Y> {
//         NodeIter2DY {
//             rows_left: Y,
//             columns_left: X,
//             cur_val: (Y as i32 * self.config.position.y) as f32,
//         }
//     }
// }

// pub struct NodeIter3DX<const X: usize, const Y: usize, const Z: usize> {
//     rows_left: usize,
//     columns_left: usize,
//     cur_vec: ArchSimd<f32>,
//     start_vec: ArchSimd<f32>,
// }

// impl<const X: usize, const Y: usize, const Z: usize> NodeIter3DX<X, Y, Z> {
//     const LANES: usize = ArchSimd::<f32>::LANES;
// }

// impl<const X: usize, const Y: usize, const Z: usize> Iterator for NodeIter3DX<X, Y, Z> {
//     type Item = ArchSimd<f32>;

//     #[inline(always)]
//     fn next(&mut self) -> Option<Self::Item> {
//         // Full columns.
//         if self.columns_left >= Self::LANES {
//             self.columns_left -= Self::LANES;
//             self.cur_vec += ArchSimd::splat(Self::LANES as f32);
//             return Some(self.cur_vec);
//         }

//         // Partial/Transition columns.
//         if self.rows_left > 0 {
//             let mask = ArchMask::first_n_true(self.columns_left as u32);
//             let old = self.cur_vec + ArchSimd::splat(Self::LANES as f32);
//             let new = self.start_vec - ArchSimd::splat(self.columns_left as f32);
//             self.cur_vec = new;
//             self.columns_left = (X - Self::LANES) + self.columns_left;
//             self.rows_left -= 1;
//             return Some(mask.select(old, new));
//         }

//         // Tail.
//         if self.columns_left > 0 {
//             let mask = ArchMask::first_n_true(self.columns_left as u32);
//             let vec = self.cur_vec + ArchSimd::splat(Self::LANES as f32);
//             self.columns_left = 0;
//             return Some(mask.select(vec, ArchSimd::zero()));
//         }

//         // Finished iter.
//         None
//     }
// }

// pub struct NodeIter3DY<const X: usize, const Y: usize> {
//     rows_left: usize,
//     columns_left: usize,
//     cur_val: f32,
// }

// impl<const X: usize, const Y: usize> NodeIter2DY<X, Y> {
//     const LANES: usize = ArchSimd::<f32>::LANES;
// }

// impl<const X: usize, const Y: usize> Iterator for NodeIter2DY<X, Y> {
//     type Item = ArchSimd<f32>;

//     #[inline(always)]
//     fn next(&mut self) -> Option<Self::Item> {
//         // Full rows.
//         if self.rows_left > Self::LANES {
//             self.rows_left -= Self::LANES;
//             return Some(ArchSimd::splat(self.cur_val));
//         }

//         // Partial/Transition rows.
//         if self.columns_left > 0 {
//             let new_val = self.cur_val + 1.0;
//             let mask = ArchMask::first_n_true(self.rows_left as u32);
//             let old = ArchSimd::splat(self.cur_val);
//             let new = ArchSimd::splat(new_val);
//             self.cur_val = new_val;
//             self.rows_left = (Y - Self::LANES) + self.rows_left;
//             self.columns_left -= 1;
//             return Some(mask.select(old, new));
//         }

//         // Tail.
//         if self.rows_left > 0 {
//             let mask = ArchMask::first_n_true(self.rows_left as u32);
//             let vec = ArchSimd::splat(self.cur_val);
//             self.rows_left = 0;
//             return Some(mask.select(vec, ArchSimd::zero()));
//         }

//         // Finished iter.
//         None
//     }
// }
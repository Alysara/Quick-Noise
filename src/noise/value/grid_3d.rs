use crate::grid_helpers::{configure_tiling, grid_fill_indices};
use crate::math::vec::{BasicVec, Vec3};
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
use crate::simd::simd_array::{SimdArray, TailInfo};
use crate::simd::simd_traits::*;

pub struct ValueContainer3D<const N: usize> {
    vecs: [SimdArray<f32, N>; 8],
    tlf: usize, // Top left front.
    trf: usize, // Top right front.
    tlb: usize, // Top left back.
    trb: usize, // Top right back.
    blf: usize, // Bottom left front.
    brf: usize, // Bottom right front.
    blb: usize, // Bottom left back.
    brb: usize, // Bottom right back.
}

impl<const N: usize> ValueContainer3D<N> {
    pub unsafe fn new_uninit() -> Self {
        unsafe {
            ValueContainer3D {
                vecs: std::array::from_fn(|_| SimdArray::new_uninit()),
                tlf: 0,
                trf: 1,
                tlb: 2,
                trb: 3,
                blf: 4,
                brf: 5,
                blb: 6,
                brb: 7,
            }
        }
    }

    pub fn tlf(&self) -> &SimdArray<f32, N> {
        unsafe { &self.vecs.get_unchecked(self.tlf) }
    }
    pub fn trf(&self) -> &SimdArray<f32, N> {
        unsafe { &self.vecs.get_unchecked(self.trf) }
    }
    pub fn blf(&self) -> &SimdArray<f32, N> {
        unsafe { &self.vecs.get_unchecked(self.blf) }
    }
    pub fn brf(&self) -> &SimdArray<f32, N> {
        unsafe { &self.vecs.get_unchecked(self.brf) }
    }
    pub fn tlb(&self) -> &SimdArray<f32, N> {
        unsafe { &self.vecs.get_unchecked(self.tlb) }
    }
    pub fn trb(&self) -> &SimdArray<f32, N> {
        unsafe { &self.vecs.get_unchecked(self.trb) }
    }
    pub fn blb(&self) -> &SimdArray<f32, N> {
        unsafe { &self.vecs.get_unchecked(self.blb) }
    }
    pub fn brb(&self) -> &SimdArray<f32, N> {
        unsafe { &self.vecs.get_unchecked(self.brb) }
    }

    pub fn tlf_trf_tlb_trb_mut(
        &mut self,
    ) -> (
        &mut SimdArray<f32, N>,
        &mut SimdArray<f32, N>,
        &mut SimdArray<f32, N>,
        &mut SimdArray<f32, N>,
    ) {
        debug_assert!(self.tlf < self.trf);
        debug_assert!(self.trf < self.tlb);
        debug_assert!(self.tlb < self.trb);
        debug_assert!(self.trb < self.vecs.len());
        unsafe {
            let ptr = self.vecs.as_mut_ptr();
            (
                &mut *ptr.add(self.tlf),
                &mut *ptr.add(self.trf),
                &mut *ptr.add(self.tlb),
                &mut *ptr.add(self.trb),
            )
        }
    }

    pub fn blf_brf_blb_brb_mut(
        &mut self,
    ) -> (
        &mut SimdArray<f32, N>,
        &mut SimdArray<f32, N>,
        &mut SimdArray<f32, N>,
        &mut SimdArray<f32, N>,
    ) {
        debug_assert!(self.blf < self.brf);
        debug_assert!(self.brf < self.blb);
        debug_assert!(self.blb < self.brb);
        debug_assert!(self.brb < self.vecs.len());
        unsafe {
            let ptr = self.vecs.as_mut_ptr();
            (
                &mut *ptr.add(self.blf),
                &mut *ptr.add(self.brf),
                &mut *ptr.add(self.blb),
                &mut *ptr.add(self.brb),
            )
        }
    }

    pub fn swap_top_bottom(&mut self) {
        std::mem::swap(&mut self.tlf, &mut self.blf);
        std::mem::swap(&mut self.trf, &mut self.brf);
        std::mem::swap(&mut self.tlb, &mut self.blb);
        std::mem::swap(&mut self.trb, &mut self.brb);
    }
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Value Grid ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

const NUM_BLOCKS: usize = NUM_SIMD_REG / 4;
const LANES: usize = ArchSimd::<f32>::LANES;
const BLOCK_LANES: usize = NUM_BLOCKS * LANES;

pub struct ValueGridNoise3D<const X: usize, const Y: usize, const Z: usize, const N: usize> {}

impl<const X: usize, const Y: usize, const Z: usize, const N: usize> ValueGridNoise3D<X, Y, Z, N> {
    const HAS_SIMD_TAIL: bool = SimdArray::<f32, X>::HAS_TAIL;
    const HAS_BLOCK_HEAD: bool = X >= BLOCK_LANES;
    const HAS_BLOCK_TAIL: bool = !X.is_multiple_of(BLOCK_LANES);

    const BLOCK_TAIL_SIZE: usize = (X % BLOCK_LANES).div_ceil(LANES);
    const SIMD_TAIL_SIZE: usize = SimdArray::<f32, X>::TAIL_SIZE;

    const BLOCK_TAIL_START: usize = (X / BLOCK_LANES) * BLOCK_LANES;
    const SIMD_TAIL_START: usize = SimdArray::<f32, X>::TAIL_START;

    #[inline(never)]
    pub(crate) fn grid_3d<const INITIALIZE: bool>(
        seed: u32,
        result: &mut SimdArray<f32, N>,
        position: Vec3<i32>,
        frequency: Vec3<f32>,
        weight: f32,
        magnification: f32,
        tiling: Vec3<Option<u32>>,
    ) {
        let increment: Vec3<f32> = frequency * magnification;
        let block_pos: Vec3<i32> = position * Vec3::new(Z as i32, Y as i32, X as i32);

        // Get the starting gradient coordinates and how far the first sample is to the next one.
        let grid_start: Vec3<i32> = (block_pos.as_f32() * increment).floor().as_i32();
        let frac_start: Vec3<f32> =
            (block_pos.as_f32() * increment - grid_start.as_f32()).float_max(Vec3::splat(0.0));

        // Get the distances from the gradient gridpoints.
        let z_distances = SimdArray::<f32, Z>::iota_custom(frac_start.z, increment.z).fract();
        let y_distances = SimdArray::<f32, Y>::iota_custom(frac_start.y, increment.y).fract();
        let x_distances = SimdArray::<f32, X>::iota_custom(frac_start.x, increment.x).fract();

        // Quintic lerp the distances to get the fade factor.
        let z_lerp = z_distances.quintic_lerp();
        let y_lerp = y_distances.quintic_lerp();
        let x_lerp = x_distances.quintic_lerp();

        let mut z_grid_indices = unsafe { SimdArray::<u32, Z>::new_uninit() };
        let mut y_grid_indices = unsafe { SimdArray::<u32, Y>::new_uninit() };
        let mut x_grid_indices = unsafe { SimdArray::<u32, X>::new_uninit() };
        let mut num_loops = Vec3::splat(0);

        // Identify the cutoff points between frequency-based grid boundaries .
        grid_fill_indices(&mut z_grid_indices, &z_distances, &mut num_loops.z);
        grid_fill_indices(&mut y_grid_indices, &y_distances, &mut num_loops.y);
        grid_fill_indices(&mut x_grid_indices, &x_distances, &mut num_loops.x);

        // Adjust tiling.
        let octave_tiling = configure_tiling(&tiling, &frequency);

        // Initialize gradient vectors.
        let mut d_vecs: ValueContainer3D<X> = unsafe { ValueContainer3D::new_uninit() };

        // println!("x_distances: {:?}\n\nx_grid_indices: {:?}\n\nnum_loops: {}", x_distances, x_grid_indices, num_loops.x);

        // Iterate through single x chunks but full y chunks.
        let mut z_cur_index: usize = 0;
        for z_it in 0..num_loops.z {
            let z_cur_fract = unsafe { z_distances.get_unchecked(z_cur_index) };
            let z_next_index = unsafe { z_grid_indices.get_unchecked(z_it) as usize };

            // Set the top gradients.
            let (tlf, trf, tlb, trb) = d_vecs.tlf_trf_tlb_trb_mut();
            Self::grid_gradients_3d(
                seed,
                tlf,
                trf,
                tlb,
                trb,
                grid_start.z + z_it as i32,
                grid_start.y,
                grid_start.x,
                &x_grid_indices,
                num_loops.x,
                &octave_tiling,
            );

            // Iterate through single x chunks but full y chunks.
            let mut y_cur_index: usize = 0;
            for y_it in 0..num_loops.y {
                let y_cur_fract = unsafe { y_distances.get_unchecked(y_cur_index) };
                let y_next_index = unsafe { y_grid_indices.get_unchecked(y_it) as usize };

                debug_assert!(y_cur_fract >= 0.0 && y_cur_fract.is_finite());

                // Set the bottom gradients.
                let (blf, brf, blb, brb) = d_vecs.blf_brf_blb_brb_mut();
                Self::grid_gradients_3d(
                    seed,
                    blf,
                    brf,
                    blb,
                    brb,
                    grid_start.z + z_it as i32,
                    grid_start.y + y_it as i32 + 1,
                    grid_start.x,
                    &x_grid_indices,
                    num_loops.x,
                    &octave_tiling,
                );

                // Perform dot products on x,y and trilinear interpolation (with quintic fade).
                Self::grid_dotted_trilerp::<INITIALIZE>(
                    &d_vecs,
                    z_cur_fract,
                    y_cur_fract,
                    increment.z,
                    increment.y,
                    &z_lerp,
                    &y_lerp,
                    &x_lerp,
                    z_cur_index,
                    y_cur_index,
                    z_next_index,
                    y_next_index,
                    weight,
                    result,
                );

                // Reuse the top and bottom gradients.
                d_vecs.swap_top_bottom();

                y_cur_index = y_next_index;
            }
            z_cur_index = z_next_index;
        }
    }

    #[inline(always)]
    pub(super) fn grid_gradients_3d(
        seed: u32,
        lf: &mut SimdArray<f32, X>,
        rf: &mut SimdArray<f32, X>,
        lb: &mut SimdArray<f32, X>,
        rb: &mut SimdArray<f32, X>,
        z_start: i32,
        y_start: i32,
        x_start: i32,
        x_grid_indices: &SimdArray<u32, X>,
        x_num_loops: usize,
        tiling: &Vec3<Option<u32>>,
    ) {
        let (z1, z2) = match tiling.z {
            None => (
                (z_start as u32).wrapping_mul(seed),
                (z_start as u32).wrapping_mul(seed).wrapping_add(seed),
            ),
            Some(t) => (
                (z_start % t as i32) as u32,
                ((z_start + 1) % t as i32) as u32,
            ),
        };
        let (z_vec_front, z_vec_back) = (ArchSimd::splat(z1), ArchSimd::splat(z2));

        let y_rem = tiling.y.map_or(y_start, |t| y_start % t as i32);
        let y_vec = ArchSimd::splat((y_rem as u32).wrapping_mul(seed));

        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        let shuffle_indices = ArchSimd::<u8>::load(&BYTE_SHUFFLE[..]);

        let prime = ArchSimd::splat(0x85ebca6b_u32);
        let z_shuf_front = z_vec_front.permute_8(shuffle_indices) ^ prime;
        let z_shuf_back = z_vec_back.permute_8(shuffle_indices) ^ prime;
        let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;

        let xy_mix_front = z_shuf_front * y_shuf;
        let xy_mix_back = z_shuf_back * y_shuf;

        // Temporary buffer to store indices for gradient values.
        let mut grad_array_front = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut grad_array_back = unsafe { SimdArray::<f32, X>::new_uninit() };

        // Main vectorized bit mixing loop.
        let end_index = x_num_loops + 1;
        let hash_mask: ArchSimd<u32> = ArchSimd::splat(0x007FFFFF);
        let exp_bits: ArchSimd<u32> = ArchSimd::splat(0x40000000);
        let three: ArchSimd<f32> = ArchSimd::splat(3.0);
        match tiling.x {
            None => {
                let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(seed);
                let mut x_vec = ArchSimd::splat((x_start as u32).wrapping_mul(seed)) + iota_vec;
                let x_vec_stride =
                    ArchSimd::splat((ArchSimd::<f32>::LANES as u32).wrapping_mul(seed));

                for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
                    let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;

                    let hash_front = xy_mix_front + y_shuf * x_shuf;
                    let hash_back = xy_mix_back + y_shuf * x_shuf;
                    let grad_front = ((hash_front & hash_mask) | exp_bits).raw_cast() - three;
                    let grad_back = ((hash_back & hash_mask) | exp_bits).raw_cast() - three;
                    unsafe {
                        grad_array_front.store_simd_tail_checked(i, grad_front);
                        grad_array_back.store_simd_tail_checked(i, grad_back);
                    }
                    x_vec += x_vec_stride;
                }
            }
            Some(x_tiling) => {
                let tiling_vec = ArchSimd::splat(x_tiling as f32);
                let mut x_vec = ArchSimd::splat(x_start) + ArchSimd::iota(0);
                let x_vec_stride = ArchSimd::splat(ArchSimd::<f32>::LANES as i32);
                let seed_vec = ArchSimd::splat(seed);

                for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
                    let x_floats = x_vec.cast_float();
                    let x_rem = x_floats - (x_floats / tiling_vec).floor() * tiling_vec;
                    let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;

                    let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
                    let hash_front = xy_mix_front + y_shuf * x_shuf;
                    let hash_back = xy_mix_back + y_shuf * x_shuf;
                    let grad_front = ((hash_front & hash_mask) | exp_bits).raw_cast() - three;
                    let grad_back = ((hash_back & hash_mask) | exp_bits).raw_cast() - three;
                    unsafe {
                        grad_array_front.store_simd_tail_checked(i, grad_front);
                        grad_array_back.store_simd_tail_checked(i, grad_back);
                    }
                    x_vec += x_vec_stride;
                }
            }
        }

        Self::grid_gradients_3d_set_loop(&grad_array_front, lf, rf, x_grid_indices, x_num_loops);
        Self::grid_gradients_3d_set_loop(&grad_array_back, lb, rb, x_grid_indices, x_num_loops);
    }

    #[inline(always)]
    pub(super) fn grid_gradients_3d_set_loop(
        grad_array: &SimdArray<f32, X>,
        left: &mut SimdArray<f32, X>,
        right: &mut SimdArray<f32, X>,
        x_grid_indices: &SimdArray<u32, X>,
        x_num_loops: usize,
    ) {
        let mut arrays = [left, right];

        let mut x_cur_index = 0;
        for x_it in 0..x_num_loops {
            let x_next_index = x_grid_indices[x_it];
            let set_amount: u32 = x_next_index - x_cur_index;

            unsafe {
                let l = grad_array.get_unchecked(x_it);
                let r = grad_array.get_unchecked(x_it + 1);
                let values = [l, r];

                SimdArray::multiset_many(
                    &mut arrays,
                    &values,
                    x_cur_index as usize,
                    set_amount as isize,
                );
            }
            x_cur_index = x_next_index;
        }
    }

    #[inline(always)]
    pub(super) fn grid_dotted_trilerp<const INITIALIZE: bool>(
        gradients: &ValueContainer3D<X>,
        z_frac_start: f32,
        y_frac_start: f32,
        z_increment: f32,
        y_increment: f32,
        z_lerp_array: &SimdArray<f32, Z>,
        y_lerp_array: &SimdArray<f32, Y>,
        x_lerp_array: &SimdArray<f32, X>,
        z_start_index: usize,
        y_start_index: usize,
        z_end_index: usize,
        y_end_index: usize,
        weight: f32,
        result: &mut SimdArray<f32, N>,
    ) {
        let weight_vec = ArchSimd::splat(weight);

        let mut tf_base = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut bf_base = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut top_base_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut bottom_base_dif = unsafe { SimdArray::<f32, X>::new_uninit() };

        for x_it in (0..X).step_by(LANES) {
            let x_lerp = x_lerp_array.load_simd_rw(x_it);

            let tlf = gradients.tlf().load_simd_rw(x_it);
            let trf = gradients.trf().load_simd_rw(x_it);
            let blf = gradients.blf().load_simd_rw(x_it);
            let brf = gradients.brf().load_simd_rw(x_it);
            let tlb = gradients.tlb().load_simd_rw(x_it);
            let trb = gradients.trb().load_simd_rw(x_it);
            let blb = gradients.blb().load_simd_rw(x_it);
            let brb = gradients.brb().load_simd_rw(x_it);

            let tf_base_vec = x_lerp.mul_add(trf - tlf, tlf) * weight_vec;
            let bf_base_vec = x_lerp.mul_add(brf - blf, blf) * weight_vec;
            let hi_base_dif_vec = x_lerp
                .mul_add(trb - tlb, tlb)
                .mul_sub(weight_vec, tf_base_vec);
            let lo_base_dif_vec = x_lerp
                .mul_add(brb - blb, blb)
                .mul_sub(weight_vec, bf_base_vec);

            tf_base.store_simd_rw(x_it, tf_base_vec);
            bf_base.store_simd_rw(x_it, bf_base_vec);
            top_base_dif.store_simd_rw(x_it, hi_base_dif_vec);
            bottom_base_dif.store_simd_rw(x_it, lo_base_dif_vec);
        }

        if Self::HAS_BLOCK_HEAD {
            Self::grid_dotted_trilerp_helper::<INITIALIZE, false>(
                z_lerp_array,
                y_lerp_array,
                z_start_index,
                y_start_index,
                z_end_index,
                y_end_index,
                &tf_base,
                &bf_base,
                &top_base_dif,
                &bottom_base_dif,
                result,
            );
        }

        if Self::HAS_BLOCK_TAIL {
            Self::grid_dotted_trilerp_helper::<INITIALIZE, true>(
                z_lerp_array,
                y_lerp_array,
                z_start_index,
                y_start_index,
                z_end_index,
                y_end_index,
                &tf_base,
                &bf_base,
                &top_base_dif,
                &bottom_base_dif,
                result,
            );
        }
    }

    #[inline(always)]
    pub(super) fn grid_dotted_trilerp_helper<const INITIALIZE: bool, const IS_TAIL: bool>(
        z_lerp_array: &SimdArray<f32, Z>,
        y_lerp_array: &SimdArray<f32, Y>,
        z_start_index: usize,
        y_start_index: usize,
        z_end_index: usize,
        y_end_index: usize,
        tf_base: &SimdArray<f32, X>,
        bf_base: &SimdArray<f32, X>,
        top_base_dif: &SimdArray<f32, X>,
        bottom_base_dif: &SimdArray<f32, X>,
        result: &mut SimdArray<f32, N>,
    ) {
        let range = if IS_TAIL {
            Self::BLOCK_TAIL_START..X
        } else {
            0..Self::BLOCK_TAIL_START
        };

        let num_blocks = if IS_TAIL {
            Self::BLOCK_TAIL_SIZE
        } else {
            NUM_BLOCKS
        };

        for z_it in z_start_index..z_end_index {
            let z_lerp = ArchSimd::splat(unsafe { z_lerp_array.get_unchecked(z_it) });

            for x_it in range.clone().step_by(BLOCK_LANES) {
                // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
                let mut base_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
                let mut base_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();

                // These blocked loops will get entirely unrolled by the compiler.
                for block in 0..num_blocks {
                    let index = x_it + LANES * block;

                    let tf = tf_base.load_simd(index);
                    let bf = bf_base.load_simd(index);
                    let top_dif = top_base_dif.load_simd(index);
                    let bottom_dif = bottom_base_dif.load_simd(index);

                    base_top[block] = z_lerp.mul_add(top_dif, tf);
                    let base_bottom = z_lerp.mul_add(bottom_dif, bf);
                    base_dif[block] = base_bottom - base_top[block];
                }

                let mut y_it = y_start_index;
                while y_it < y_end_index {
                    if y_it + 4 > y_end_index {
                        Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
                            &mut base_dif,
                            &mut base_top,
                            z_it,
                            y_it,
                            x_it,
                            y_lerp_array,
                            result,
                            0,
                        );
                        y_it += 1;
                    } else {
                        for i in 0..4 {
                            Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
                                &mut base_dif,
                                &mut base_top,
                                z_it,
                                y_it,
                                x_it,
                                y_lerp_array,
                                result,
                                i,
                            );
                        }
                        y_it += 4;
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn process_lerp_block<const INITIALIZE: bool, const IS_TAIL: bool>(
        base_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
        base_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
        z_it: usize,
        y_it: usize,
        x_it: usize,
        y_lerp_array: &SimdArray<f32, Y>,
        result: &mut SimdArray<f32, N>,
        y_idx: usize,
    ) {
        let y_lerp = ArchSimd::splat(unsafe { y_lerp_array.get_unchecked(y_it + y_idx) });

        let range = if IS_TAIL {
            0..Self::BLOCK_TAIL_SIZE
        } else {
            0..NUM_BLOCKS
        };

        let base_index = z_it * X * Y + y_it * X;
        for block in range {
            let x_index = x_it + block * LANES;
            let index = base_index + x_index + X * y_idx;
            let output = y_lerp.mul_add(base_dif[block], base_top[block]);

            let val = if INITIALIZE {
                output
            } else {
                unsafe { output + result.load_simd_tail_checked(index) }
            };

            unsafe {
                if IS_TAIL && Self::HAS_SIMD_TAIL && x_index >= Self::SIMD_TAIL_START {
                    result.partial_store_simd_unchecked(index, val, Self::SIMD_TAIL_SIZE);
                } else {
                    result.store_simd_unchecked(index, val)
                };
            }
        }
    }
}

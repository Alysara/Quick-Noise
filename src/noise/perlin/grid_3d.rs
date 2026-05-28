use crate::grid_helpers::grid_fill_indices;
use crate::math::vec::{Vec3};
use crate::noise::perlin::constants::*;
use crate::noise::perlin::containers::*;
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
use crate::simd::simd_array::{SimdArray, TailInfo};
use crate::simd::simd_traits::*;

// ————————————————————————————————————————————————————————————————
// ————— 3D Perlin Grid ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

const NUM_BLOCKS: usize = NUM_SIMD_REG / 8;
const LANES: usize = ArchSimd::<f32>::LANES;
const BLOCK_LANES: usize = NUM_BLOCKS * LANES;

pub struct PerlinGridNoise3D<const A: usize, const Y: usize, const X: usize, const N: usize> {}

impl<const X: usize, const Y: usize, const Z: usize, const N: usize> PerlinGridNoise3D<X, Y, Z, N> {
    const HAS_SIMD_TAIL: bool = SimdArray::<f32, X>::HAS_TAIL;
    const HAS_BLOCK_HEAD: bool = X >= BLOCK_LANES;
    const HAS_BLOCK_TAIL: bool = (X % BLOCK_LANES) > 0;

    const BLOCK_TAIL_SIZE: usize = (X % BLOCK_LANES + LANES - 1) / LANES;
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

        // Initialize gradient vectors.
        let mut d_vecs: PerlinContainer3D<X> = unsafe { PerlinContainer3D::new_uninit() };

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
                &x_distances,
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
                    &x_distances,
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
                    z_cur_index as usize,
                    y_cur_index as usize,
                    z_next_index as usize,
                    y_next_index as usize,
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
        lf: &mut Vec3<SimdArray<f32, X>>,
        rf: &mut Vec3<SimdArray<f32, X>>,
        lb: &mut Vec3<SimdArray<f32, X>>,
        rb: &mut Vec3<SimdArray<f32, X>>,
        z_start: i32,
        y_start: i32,
        x_start: i32,
        x_grid_indices: &SimdArray<u32, X>,
        x_num_loops: usize,
        x_distances: &SimdArray<f32, X>,
    ) {
        let x1 = (z_start as u32).wrapping_mul(seed);
        let x2 = x1.wrapping_add(seed);
        let z_vec_front = ArchSimd::splat(x1);
        let z_vec_back = ArchSimd::splat(x2);
        let y_vec = ArchSimd::splat((y_start as u32).wrapping_mul(seed));

        let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(seed);
        let mut x_vec = ArchSimd::splat((x_start as u32).wrapping_mul(seed)) + iota_vec;
        let x_vec_stride = ArchSimd::splat((ArchSimd::<f32>::LANES as u32).wrapping_mul(seed));

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

        let xy_miz_front = z_shuf_front * y_shuf;
        let xy_miz_back = z_shuf_back * y_shuf;

        // Temporary buffer to store indices for gradient values.
        let mut grad_array_front = unsafe { SimdArray::<u32, X>::new_uninit() };
        let mut grad_array_back = unsafe { SimdArray::<u32, X>::new_uninit() };

        // Main vectorized bit mixing loop.
        let end_index = x_num_loops as usize + 1;
        for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
            unsafe {
                grad_array_front.store_simd_tail_checked(i, (xy_miz_front * x_shuf) >> 29);
                grad_array_back.store_simd_tail_checked(i, (xy_miz_back * x_shuf) >> 29);
            }
            x_vec += x_vec_stride;
        }

        Self::grid_gradients_3d_set_loop(&grad_array_front, lf, rf, x_grid_indices, x_num_loops);
        Self::grid_gradients_3d_set_loop(&grad_array_back, lb, rb, x_grid_indices, x_num_loops);

        lf.x *= *x_distances;
        rf.x = rf.x.mul_sub(*x_distances, rf.x);
        lb.x *= *x_distances;
        rb.x = rb.x.mul_sub(*x_distances, rb.x);
    }

    #[inline(always)]
    pub(super) fn grid_gradients_3d_set_loop(
        grad_array: &SimdArray<u32, X>,
        left: &mut Vec3<SimdArray<f32, X>>,
        right: &mut Vec3<SimdArray<f32, X>>,
        x_grid_indices: &SimdArray<u32, X>,
        x_num_loops: usize,
    ) {
        let mut arrays = [
            &mut left.z,
            &mut left.y,
            &mut left.x,
            &mut right.z,
            &mut right.y,
            &mut right.x,
        ];

        let mut x_cur_index = 0;
        for x_it in 0..x_num_loops {
            let x_next_index = x_grid_indices[x_it];
            let set_amount: u32 = x_next_index - x_cur_index;

            unsafe {
                let lf_grad = grad_array.get_unchecked(x_it as usize) as usize;
                let rf_grad = grad_array.get_unchecked(x_it as usize + 1) as usize;
                debug_assert!(lf_grad < 32);
                debug_assert!(rf_grad < 32);
                let values = [
                    GRADIENTS_3D.get_unchecked(lf_grad).z,
                    GRADIENTS_3D.get_unchecked(lf_grad).y,
                    GRADIENTS_3D.get_unchecked(lf_grad).x,
                    GRADIENTS_3D.get_unchecked(rf_grad).z,
                    GRADIENTS_3D.get_unchecked(rf_grad).y,
                    GRADIENTS_3D.get_unchecked(rf_grad).x,
                ];

                SimdArray::multiset_many::<6>(
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
        gradients: &PerlinContainer3D<X>,
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
        let z_weighted_increment_vec = ArchSimd::splat(z_increment * weight);
        let y_weighted_increment_vec = ArchSimd::splat(y_increment * weight);
        let z_upper_increment = ArchSimd::splat(z_frac_start);
        let z_lower_increment = ArchSimd::splat(z_frac_start - 1.0);
        let y_upper_increment = ArchSimd::splat(y_frac_start);
        let y_lower_increment = ArchSimd::splat(y_frac_start - 1.0);

        let mut y_tf_offset = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut y_bf_offset = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut y_top_offset_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut y_bottom_offset_dif = unsafe { SimdArray::<f32, X>::new_uninit() };

        let mut z_tf_offset = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut z_bf_offset = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut z_top_offset_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut z_bottom_offset_dif = unsafe { SimdArray::<f32, X>::new_uninit() };

        let mut tf_base = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut bf_base = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut top_base_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
        let mut bottom_base_dif = unsafe { SimdArray::<f32, X>::new_uninit() };

        for x_it in (0..X).step_by(LANES) {
            let x_lerp = x_lerp_array.load_simd_rw(x_it);

            let z_tlf = gradients.tlf().z.load_simd_rw(x_it);
            let z_trf = gradients.trf().z.load_simd_rw(x_it);
            let z_blf = gradients.blf().z.load_simd_rw(x_it);
            let z_brf = gradients.brf().z.load_simd_rw(x_it);
            let z_tlb = gradients.tlb().z.load_simd_rw(x_it);
            let z_trb = gradients.trb().z.load_simd_rw(x_it);
            let z_blb = gradients.blb().z.load_simd_rw(x_it);
            let z_brb = gradients.brb().z.load_simd_rw(x_it);

            let y_tlf = gradients.tlf().y.load_simd_rw(x_it);
            let y_trf = gradients.trf().y.load_simd_rw(x_it);
            let y_blf = gradients.blf().y.load_simd_rw(x_it);
            let y_brf = gradients.brf().y.load_simd_rw(x_it);
            let y_tlb = gradients.tlb().y.load_simd_rw(x_it);
            let y_trb = gradients.trb().y.load_simd_rw(x_it);
            let y_blb = gradients.blb().y.load_simd_rw(x_it);
            let y_brb = gradients.brb().y.load_simd_rw(x_it);

            let x_tlf = gradients.tlf().x.load_simd_rw(x_it);
            let x_trf = gradients.trf().x.load_simd_rw(x_it);
            let x_blf = gradients.blf().x.load_simd_rw(x_it);
            let x_brf = gradients.brf().x.load_simd_rw(x_it);
            let x_tlb = gradients.tlb().x.load_simd_rw(x_it);
            let x_trb = gradients.trb().x.load_simd_rw(x_it);
            let x_blb = gradients.blb().x.load_simd_rw(x_it);
            let x_brb = gradients.brb().x.load_simd_rw(x_it);

            let sum_prod_tlf =
                z_upper_increment.mul_add(z_tlf, y_upper_increment.mul_add(y_tlf, x_tlf));
            let sum_prod_trf =
                z_upper_increment.mul_add(z_trf, y_upper_increment.mul_add(y_trf, x_trf));
            let sum_prod_blf =
                z_upper_increment.mul_add(z_blf, y_lower_increment.mul_add(y_blf, x_blf));
            let sum_prod_brf =
                z_upper_increment.mul_add(z_brf, y_lower_increment.mul_add(y_brf, x_brf));
            let sum_prod_tlb =
                z_lower_increment.mul_add(z_tlb, y_upper_increment.mul_add(y_tlb, x_tlb));
            let sum_prod_trb =
                z_lower_increment.mul_add(z_trb, y_upper_increment.mul_add(y_trb, x_trb));
            let sum_prod_blb =
                z_lower_increment.mul_add(z_blb, y_lower_increment.mul_add(y_blb, x_blb));
            let sum_prod_brb =
                z_lower_increment.mul_add(z_brb, y_lower_increment.mul_add(y_brb, x_brb));

            let z_tf_offset_vec = x_lerp.mul_add(z_trf - z_tlf, z_tlf) * z_weighted_increment_vec;
            let z_bf_offset_vec = x_lerp.mul_add(z_brf - z_blf, z_blf) * z_weighted_increment_vec;
            let z_tb_offset_vec = x_lerp.mul_add(z_trb - z_tlb, z_tlb) * z_weighted_increment_vec;
            let z_bb_offset_vec = x_lerp.mul_add(z_brb - z_blb, z_blb) * z_weighted_increment_vec;

            let y_tf_offset_vec = x_lerp.mul_add(y_trf - y_tlf, y_tlf) * y_weighted_increment_vec;
            let y_bf_offset_vec = x_lerp.mul_add(y_brf - y_blf, y_blf) * y_weighted_increment_vec;
            let y_hi_offset_dif_vec = x_lerp
                .mul_add(y_trb - y_tlb, y_tlb)
                .mul_sub(y_weighted_increment_vec, y_tf_offset_vec);
            let y_lo_offset_dif_vec = x_lerp
                .mul_add(y_brb - y_blb, y_blb)
                .mul_sub(y_weighted_increment_vec, y_bf_offset_vec);

            let tf_base_vec =
                x_lerp.mul_add(sum_prod_trf - sum_prod_tlf, sum_prod_tlf) * weight_vec;
            let bf_base_vec =
                x_lerp.mul_add(sum_prod_brf - sum_prod_blf, sum_prod_blf) * weight_vec;
            let hi_base_dif_vec = x_lerp
                .mul_add(sum_prod_trb - sum_prod_tlb, sum_prod_tlb)
                .mul_sub(weight_vec, tf_base_vec);
            let lo_base_dif_vec = x_lerp
                .mul_add(sum_prod_brb - sum_prod_blb, sum_prod_blb)
                .mul_sub(weight_vec, bf_base_vec);

            z_tf_offset.store_simd_rw(x_it, z_tf_offset_vec);
            z_bf_offset.store_simd_rw(x_it, z_bf_offset_vec);
            z_top_offset_dif.store_simd_rw(x_it, z_tb_offset_vec - z_tf_offset_vec);
            z_bottom_offset_dif.store_simd_rw(x_it, z_bb_offset_vec - z_bf_offset_vec);

            y_tf_offset.store_simd_rw(x_it, y_tf_offset_vec);
            y_bf_offset.store_simd_rw(x_it, y_bf_offset_vec);
            y_top_offset_dif.store_simd_rw(x_it, y_hi_offset_dif_vec);
            y_bottom_offset_dif.store_simd_rw(x_it, y_lo_offset_dif_vec);

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
                &y_tf_offset,
                &y_bf_offset,
                &y_top_offset_dif,
                &y_bottom_offset_dif,
                &z_tf_offset,
                &z_bf_offset,
                &z_top_offset_dif,
                &z_bottom_offset_dif,
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
                &y_tf_offset,
                &y_bf_offset,
                &y_top_offset_dif,
                &y_bottom_offset_dif,
                &z_tf_offset,
                &z_bf_offset,
                &z_top_offset_dif,
                &z_bottom_offset_dif,
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
        y_tf_offset: &SimdArray<f32, X>,
        y_bf_offset: &SimdArray<f32, X>,
        y_top_offset_dif: &SimdArray<f32, X>,
        y_bottom_offset_dif: &SimdArray<f32, X>,
        z_tf_offset: &SimdArray<f32, X>,
        z_bf_offset: &SimdArray<f32, X>,
        z_top_offset_dif: &SimdArray<f32, X>,
        z_bottom_offset_dif: &SimdArray<f32, X>,
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

        let mut z_counter: f32 = 0.0;
        for z_it in z_start_index..z_end_index {
            let z_lerp = ArchSimd::splat(unsafe { z_lerp_array.get_unchecked(z_it) });
            let z_cur_vec = ArchSimd::splat(z_counter);

            for x_it in range.clone().step_by(BLOCK_LANES) {
                // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
                let mut base_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
                let mut base_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
                let mut y_offset_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
                let mut y_offset_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();

                // These blocked loops will get entirely unrolled by the compiler.
                for block in 0..num_blocks {
                    let index = x_it + LANES * block;
                    let z_tf_offset_vec = z_tf_offset.load_simd(index);
                    let z_bf_offset_vec = z_bf_offset.load_simd(index);
                    let z_top_offset_dif_vec = z_top_offset_dif.load_simd(index);
                    let z_bottom_offset_dif_vec = z_bottom_offset_dif.load_simd(index);

                    let y_tf_offset_vec = y_tf_offset.load_simd(index);
                    let y_bf_offset_vec = y_bf_offset.load_simd(index);
                    let y_top_offset_dif_vec = y_top_offset_dif.load_simd(index);
                    let y_bottom_offset_dif_vec = y_bottom_offset_dif.load_simd(index);

                    let tf_base_vec = tf_base.load_simd(index);
                    let bf_base_vec = bf_base.load_simd(index);
                    let top_base_dif_vec = top_base_dif.load_simd(index);
                    let bottom_base_dif_vec = bottom_base_dif.load_simd(index);

                    let z_top_offset = z_lerp.mul_add(z_top_offset_dif_vec, z_tf_offset_vec);
                    let z_bottom_offset = z_lerp.mul_add(z_bottom_offset_dif_vec, z_bf_offset_vec);

                    base_top[block] = z_cur_vec
                        .mul_add(z_top_offset, z_lerp.mul_add(top_base_dif_vec, tf_base_vec));
                    let bottom_base = z_cur_vec.mul_add(
                        z_bottom_offset,
                        z_lerp.mul_add(bottom_base_dif_vec, bf_base_vec),
                    );
                    base_dif[block] = bottom_base - base_top[block];

                    y_offset_top[block] = z_lerp.mul_add(y_top_offset_dif_vec, y_tf_offset_vec);
                    let y_bottom_offset = z_lerp.mul_add(y_bottom_offset_dif_vec, y_bf_offset_vec);
                    y_offset_dif[block] = y_bottom_offset - y_offset_top[block];
                }

                let mut y_it = y_start_index;
                while y_it < y_end_index {
                    if y_it + 4 > y_end_index {
                        Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
                            &mut base_dif,
                            &mut base_top,
                            &y_offset_dif,
                            &y_offset_top,
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
                                &y_offset_dif,
                                &y_offset_top,
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
            z_counter += 1.0;
        }
    }

    #[inline(always)]
    fn process_lerp_block<const INITIALIZE: bool, const IS_TAIL: bool>(
        base_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
        base_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
        y_offset_dif: &[ArchSimd<f32>; NUM_BLOCKS],
        y_offset_top: &[ArchSimd<f32>; NUM_BLOCKS],
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

            base_dif[block] += y_offset_dif[block];
            base_top[block] += y_offset_top[block];
        }
    }
}

use crate::noise::perlin::Perlin;
use crate::noise::perlin::constants::*;
use crate::noise::perlin::containers::*;
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
use crate::simd::simd_traits::*;

impl Perlin {
    #[inline(never)]
    pub(super) fn uniform_grid_interpolate_2d<const INITIALIZE: bool>(
        gradients: &PerlinContainer2D,
        x_frac_start: f32,
        x_increment: f32,
        interpolations: &PerlinVecPair,
        x_start_index: usize,
        x_end_index: usize,
        weight: f32,
        result: &mut PerlinMap,
    ) {
        let weight_vec = ArchSimd::splat(weight);
        let x_weighted_increment_vec = ArchSimd::splat(x_increment * weight);
        let x_upper_increment = ArchSimd::splat(x_frac_start);
        let x_lower_increment = ArchSimd::splat(x_frac_start - 1.0);

        const NUM_BLOCKS_POSSIBLE: usize = NUM_SIMD_REG / 8;
        const MAX_NUM_BLOCKS: usize = ROW_SIZE / ArchSimd::<f32>::LANES;
        const NUM_BLOCKS: usize = if NUM_BLOCKS_POSSIBLE < MAX_NUM_BLOCKS { NUM_BLOCKS_POSSIBLE } else { MAX_NUM_BLOCKS };

        for y_it in (0..ROW_SIZE).step_by(ArchSimd::<f32>::LANES * NUM_BLOCKS) {
            // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
            let mut base_lerps_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
            let mut base_lerps_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
            let mut x_offset_lerps_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
            let mut x_offset_lerps_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();

            // These blocked loops will get entirely unrolled by the compiler.
            for block in 0..NUM_BLOCKS {
                // Load gradients into registers.
                let y_lerp = interpolations.y.load_simd(y_it + ArchSimd::<f32>::LANES * block);
                let x_tl: ArchSimd<f32> = gradients.tl().x.load_simd(y_it + ArchSimd::<f32>::LANES * block);
                let x_tr: ArchSimd<f32> = gradients.tr().x.load_simd(y_it + ArchSimd::<f32>::LANES * block);
                let x_bl: ArchSimd<f32> = gradients.bl().x.load_simd(y_it + ArchSimd::<f32>::LANES * block);
                let x_br: ArchSimd<f32> = gradients.br().x.load_simd(y_it + ArchSimd::<f32>::LANES * block);
                let y_tl: ArchSimd<f32> = gradients.tl().y.load_simd(y_it + ArchSimd::<f32>::LANES * block);
                let y_tr: ArchSimd<f32> = gradients.tr().y.load_simd(y_it + ArchSimd::<f32>::LANES * block);
                let y_bl: ArchSimd<f32> = gradients.bl().y.load_simd(y_it + ArchSimd::<f32>::LANES * block);
                let y_br: ArchSimd<f32> = gradients.br().y.load_simd(y_it + ArchSimd::<f32>::LANES * block);

                // Compute base dot products.
                let prod_sum_tl = x_tl.mul_add(x_upper_increment, y_tl);
                let prod_sum_tr = x_tr.mul_add(x_upper_increment, y_tr);
                let prod_sum_bl = x_bl.mul_add(x_lower_increment, y_bl);
                let prod_sum_br = x_br.mul_add(x_lower_increment, y_br);

                // Base interpolation.
                base_lerps_top[block] =
                    y_lerp.mul_add(prod_sum_tr - prod_sum_tl, prod_sum_tl) * weight_vec;
                let base_lerp_bottom =
                    y_lerp.mul_add(prod_sum_br - prod_sum_bl, prod_sum_bl) * weight_vec;
                base_lerps_dif[block] = base_lerp_bottom - base_lerps_top[block];

                // Offset interpolation.
                x_offset_lerps_top[block] =
                    y_lerp.mul_add(x_tr - x_tl, x_tl) * x_weighted_increment_vec;
                let x_offset_lerp_bottom =
                    y_lerp.mul_add(x_br - x_bl, x_bl) * x_weighted_increment_vec;
                x_offset_lerps_dif[block] = x_offset_lerp_bottom - x_offset_lerps_top[block];
            }

            // for x_it in x_start_index..x_start_index + x_chunk_size {
            //     let x_lerp = ArchSimd::splat(unsafe { interpolations.x.get_unchecked(x_it) } );

            //     for block in 0..NUM_BLOCKS {
            //         let index: usize = x_it * ROW_SIZE + y_it + block * ArchSimd::<f32>::LANES;

            //         // Final interpolation.
            //         let output = x_lerp.mul_add(base_lerps_dif[block], base_lerps_top[block]);

            //         // Store if Initialize, Add to and store if not.
            //         let val = if INITIALIZE { output } else { output + result.load_simd(index) };
            //         result.store_simd(index, val);

            //         // Accumulate interpolation variables.
            //         base_lerps_dif[block] += x_offset_lerps_dif[block];
            //         base_lerps_top[block] += x_offset_lerps_top[block];
            //     }
            // }

            let mut x_it = x_start_index;
            while x_it < x_end_index {
                if x_it + 4 > x_end_index {
                    let x_lerp = ArchSimd::splat(unsafe { interpolations.x.get_unchecked(x_it) });
                    for block in 0..NUM_BLOCKS {
                        let index: usize = x_it * ROW_SIZE + y_it + block * ArchSimd::<f32>::LANES;
                        let output = x_lerp.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                        result.store_simd(index, val);
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    x_it += 1;
                } else {
                    let x_lerp_1 = ArchSimd::splat(unsafe { interpolations.x.get_unchecked(x_it) });
                    let x_lerp_2 = ArchSimd::splat(unsafe { interpolations.x.get_unchecked(x_it + 1) });
                    let x_lerp_3 = ArchSimd::splat(unsafe { interpolations.x.get_unchecked(x_it + 2) });
                    let x_lerp_4 = ArchSimd::splat(unsafe { interpolations.x.get_unchecked(x_it + 3) });

                    for block in 0..NUM_BLOCKS {
                        let index: usize = x_it * ROW_SIZE + y_it + block * ArchSimd::<f32>::LANES;
                        let output = x_lerp_1.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                        result.store_simd(index, val);
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    for block in 0..NUM_BLOCKS {
                        let index: usize = x_it * ROW_SIZE + y_it + block * ArchSimd::<f32>::LANES + ROW_SIZE;
                        let output = x_lerp_2.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                        result.store_simd(index, val);
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    for block in 0..NUM_BLOCKS {
                        let index: usize = x_it * ROW_SIZE + y_it + block * ArchSimd::<f32>::LANES + ROW_SIZE * 2;
                        let output = x_lerp_3.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                        result.store_simd(index, val);
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    for block in 0..NUM_BLOCKS {
                        let index: usize = x_it * ROW_SIZE + y_it + block * ArchSimd::<f32>::LANES + ROW_SIZE * 3;
                        let output = x_lerp_4.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                        result.store_simd(index, val);
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    x_it += 4;
                }
            }
        }
    }

    #[inline(never)]
    pub(super) fn uniform_grid_interpolate_3d<const INITIALIZE: bool>(
        gradients: &PerlinContainer3D,
        x_frac_start: f32,
        y_frac_start: f32,
        x_increment: f32,
        y_increment: f32,
        interpolations: &PerlinVecTriple,
        x_start_index: usize,
        y_start_index: usize,
        x_end_index: usize,
        y_end_index: usize,
        weight: f32,
        result: &mut PerlinVol,
    ) {
        let weight_vec = ArchSimd::splat(weight);
        let x_weighted_increment_vec = ArchSimd::splat(x_increment * weight);
        let y_weighted_increment_vec = ArchSimd::splat(y_increment * weight);
        let x_upper_increment = ArchSimd::splat(x_frac_start);
        let x_lower_increment = ArchSimd::splat(x_frac_start - 1.0);
        let y_upper_increment = ArchSimd::splat(y_frac_start);
        let y_lower_increment = ArchSimd::splat(y_frac_start - 1.0);

        let mut y_tf_offset = unsafe { PerlinVec::new_uninit() };
        let mut y_bf_offset = unsafe { PerlinVec::new_uninit() };
        let mut y_top_offset_dif = unsafe { PerlinVec::new_uninit() };
        let mut y_bottom_offset_dif = unsafe { PerlinVec::new_uninit() };

        let mut x_tf_offset = unsafe { PerlinVec::new_uninit() };
        let mut x_bf_offset = unsafe { PerlinVec::new_uninit() };
        let mut x_top_offset_dif = unsafe { PerlinVec::new_uninit() };
        let mut x_bottom_offset_dif = unsafe { PerlinVec::new_uninit() };

        let mut tf_base = unsafe { PerlinVec::new_uninit() };
        let mut bf_base = unsafe { PerlinVec::new_uninit() };
        let mut top_base_dif = unsafe { PerlinVec::new_uninit() };
        let mut bottom_base_dif = unsafe { PerlinVec::new_uninit() };

        for z_it in (0..ROW_SIZE).step_by(ArchSimd::<f32>::LANES) {
            let z_lerp = interpolations.z.load_simd(z_it);

            let x_tlf = gradients.tlf().x.load_simd(z_it);
            let x_trf = gradients.trf().x.load_simd(z_it);
            let x_blf = gradients.blf().x.load_simd(z_it);
            let x_brf = gradients.brf().x.load_simd(z_it);
            let x_tlb = gradients.tlb().x.load_simd(z_it);
            let x_trb = gradients.trb().x.load_simd(z_it);
            let x_blb = gradients.blb().x.load_simd(z_it);
            let x_brb = gradients.brb().x.load_simd(z_it);

            let y_tlf = gradients.tlf().y.load_simd(z_it);
            let y_trf = gradients.trf().y.load_simd(z_it);
            let y_blf = gradients.blf().y.load_simd(z_it);
            let y_brf = gradients.brf().y.load_simd(z_it);
            let y_tlb = gradients.tlb().y.load_simd(z_it);
            let y_trb = gradients.trb().y.load_simd(z_it);
            let y_blb = gradients.blb().y.load_simd(z_it);
            let y_brb = gradients.brb().y.load_simd(z_it);

            let z_tlf = gradients.tlf().z.load_simd(z_it);
            let z_trf = gradients.trf().z.load_simd(z_it);
            let z_blf = gradients.blf().z.load_simd(z_it);
            let z_brf = gradients.brf().z.load_simd(z_it);
            let z_tlb = gradients.tlb().z.load_simd(z_it);
            let z_trb = gradients.trb().z.load_simd(z_it);
            let z_blb = gradients.blb().z.load_simd(z_it);
            let z_brb = gradients.brb().z.load_simd(z_it);

            let sum_prod_tlf = x_upper_increment.mul_add(x_tlf, y_upper_increment.mul_add(y_tlf, z_tlf));
            let sum_prod_trf = x_upper_increment.mul_add(x_trf, y_upper_increment.mul_add(y_trf, z_trf));
            let sum_prod_blf = x_upper_increment.mul_add(x_blf, y_lower_increment.mul_add(y_blf, z_blf));
            let sum_prod_brf = x_upper_increment.mul_add(x_brf, y_lower_increment.mul_add(y_brf, z_brf));
            let sum_prod_tlb = x_lower_increment.mul_add(x_tlb, y_upper_increment.mul_add(y_tlb, z_tlb));
            let sum_prod_trb = x_lower_increment.mul_add(x_trb, y_upper_increment.mul_add(y_trb, z_trb));
            let sum_prod_blb = x_lower_increment.mul_add(x_blb, y_lower_increment.mul_add(y_blb, z_blb));
            let sum_prod_brb = x_lower_increment.mul_add(x_brb, y_lower_increment.mul_add(y_brb, z_brb));

            let x_tf_offset_vec = z_lerp.mul_add(x_trf - x_tlf, x_tlf) * x_weighted_increment_vec;
            let x_bf_offset_vec = z_lerp.mul_add(x_brf - x_blf, x_blf) * x_weighted_increment_vec;
            let x_tb_offset_vec = z_lerp.mul_add(x_trb - x_tlb, x_tlb) * x_weighted_increment_vec;
            let x_bb_offset_vec = z_lerp.mul_add(x_brb - x_blb, x_blb) * x_weighted_increment_vec;

            let y_tf_offset_vec = z_lerp.mul_add(y_trf - y_tlf, y_tlf) * y_weighted_increment_vec;
            let y_bf_offset_vec = z_lerp.mul_add(y_brf - y_blf, y_blf) * y_weighted_increment_vec;
            let y_hi_offset_dif_vec = z_lerp.mul_add(y_trb - y_tlb, y_tlb).mul_sub(y_weighted_increment_vec, y_tf_offset_vec);
            let y_lo_offset_dif_vec = z_lerp.mul_add(y_brb - y_blb, y_blb).mul_sub(y_weighted_increment_vec, y_bf_offset_vec);

            let tf_base_vec = z_lerp.mul_add(sum_prod_trf - sum_prod_tlf, sum_prod_tlf) * weight_vec;
            let bf_base_vec = z_lerp.mul_add(sum_prod_brf - sum_prod_blf, sum_prod_blf) * weight_vec;
            let hi_base_dif_vec = z_lerp.mul_add(sum_prod_trb - sum_prod_tlb, sum_prod_tlb).mul_sub(weight_vec, tf_base_vec);
            let lo_base_dif_vec = z_lerp.mul_add(sum_prod_brb - sum_prod_blb, sum_prod_blb).mul_sub(weight_vec, bf_base_vec);

            x_tf_offset.store_simd(z_it, x_tf_offset_vec);
            x_bf_offset.store_simd(z_it, x_bf_offset_vec);
            x_top_offset_dif.store_simd(z_it, x_tb_offset_vec - x_tf_offset_vec);
            x_bottom_offset_dif.store_simd(z_it, x_bb_offset_vec - x_bf_offset_vec);

            y_tf_offset.store_simd(z_it, y_tf_offset_vec);
            y_bf_offset.store_simd(z_it, y_bf_offset_vec);
            y_top_offset_dif.store_simd(z_it, y_hi_offset_dif_vec);
            y_bottom_offset_dif.store_simd(z_it, y_lo_offset_dif_vec);

            tf_base.store_simd(z_it, tf_base_vec);
            bf_base.store_simd(z_it, bf_base_vec);
            top_base_dif.store_simd(z_it, hi_base_dif_vec);
            bottom_base_dif.store_simd(z_it, lo_base_dif_vec);
        }

        const NUM_BLOCKS_POSSIBLE: usize = NUM_SIMD_REG / 8;
        const MAX_NUM_BLOCKS: usize = ROW_SIZE / ArchSimd::<f32>::LANES;
        const NUM_BLOCKS: usize = if NUM_BLOCKS_POSSIBLE < MAX_NUM_BLOCKS { NUM_BLOCKS_POSSIBLE } else { MAX_NUM_BLOCKS };

        let mut x_counter: f32 = 0.0;
        for x_it in x_start_index..x_end_index {
            let x_lerp = ArchSimd::splat(unsafe { interpolations.x.get_unchecked(x_it) });
            let x_cur_vec = ArchSimd::splat(x_counter);

            for z_it in (0..ROW_SIZE).step_by(ArchSimd::<f32>::LANES * NUM_BLOCKS) {
                // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
                let mut base_lerps_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
                let mut base_lerps_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
                let mut y_offset_lerps_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
                let mut y_offset_lerps_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();

                // These blocked loops will get entirely unrolled by the compiler.
                for block in 0..NUM_BLOCKS {
                    let x_tf_offset_vec = x_tf_offset.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let x_bf_offset_vec = x_bf_offset.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let x_top_offset_dif_vec = x_top_offset_dif.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let x_bottom_offset_dif_vec = x_bottom_offset_dif.load_simd(z_it + ArchSimd::<f32>::LANES * block);

                    let y_tf_offset_vec = y_tf_offset.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let y_bf_offset_vec = y_bf_offset.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let y_top_offset_dif_vec = y_top_offset_dif.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let y_bottom_offset_dif_vec = y_bottom_offset_dif.load_simd(z_it + ArchSimd::<f32>::LANES * block);

                    let tf_base_vec = tf_base.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let bf_base_vec = bf_base.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let top_base_dif_vec = top_base_dif.load_simd(z_it + ArchSimd::<f32>::LANES * block);
                    let bottom_base_dif_vec = bottom_base_dif.load_simd(z_it + ArchSimd::<f32>::LANES * block);

                    let x_top_offset = x_lerp.mul_add(x_top_offset_dif_vec, x_tf_offset_vec);
                    let x_bottom_offset = x_lerp.mul_add(x_bottom_offset_dif_vec, x_bf_offset_vec);

                    base_lerps_top[block] = x_cur_vec.mul_add(x_top_offset, x_lerp.mul_add(top_base_dif_vec, tf_base_vec));
                    let bottom_base = x_cur_vec.mul_add(x_bottom_offset, x_lerp.mul_add(bottom_base_dif_vec, bf_base_vec));
                    base_lerps_dif[block] = bottom_base - base_lerps_top[block];

                    y_offset_lerps_top[block] = x_lerp.mul_add(y_top_offset_dif_vec, y_tf_offset_vec);
                    let y_bottom_offset = x_lerp.mul_add(y_bottom_offset_dif_vec, y_bf_offset_vec);
                    y_offset_lerps_dif[block] = y_bottom_offset - y_offset_lerps_top[block];
                }

                // for y_it in y_start_index..y_start_index + y_chunk_size {
                //     let y_lerp = ArchSimd::splat(unsafe { interpolations.x.get_unchecked(y_it) } );

                //     for block in 0..NUM_BLOCKS {
                //         let index: usize = x_it * MAP_SIZE + y_it * ROW_SIZE + z_it + block * ArchSimd::<f32>::LANES;
                //         let output = y_lerp.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                //         let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                //         result.store_simd(index, val);
                //         base_lerps_dif[block] += y_offset_lerps_dif[block];
                //         base_lerps_top[block] += y_offset_lerps_top[block];
                //     }
                // }

                let mut y_it = y_start_index;
                while y_it < y_end_index {
                    if y_it + 4 <= y_end_index {
                        let y_lerp_1 = ArchSimd::splat(unsafe { interpolations.y.get_unchecked(y_it) });
                        let y_lerp_2 = ArchSimd::splat(unsafe { interpolations.y.get_unchecked(y_it + 1) });
                        let y_lerp_3 = ArchSimd::splat(unsafe { interpolations.y.get_unchecked(y_it + 2) });
                        let y_lerp_4 = ArchSimd::splat(unsafe { interpolations.y.get_unchecked(y_it + 3) });

                        let base_index: usize = x_it * MAP_SIZE + y_it * ROW_SIZE + z_it;
                        for block in 0..NUM_BLOCKS {
                            let index: usize = base_index + block * ArchSimd::<f32>::LANES;
                            let output = y_lerp_1.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                            let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                            result.store_simd(index, val);
                            base_lerps_dif[block] += y_offset_lerps_dif[block];
                            base_lerps_top[block] += y_offset_lerps_top[block];
                        }
                        for block in 0..NUM_BLOCKS {
                            let index: usize = base_index + block * ArchSimd::<f32>::LANES + ROW_SIZE;
                            let output = y_lerp_2.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                            let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                            result.store_simd(index, val);
                            base_lerps_dif[block] += y_offset_lerps_dif[block];
                            base_lerps_top[block] += y_offset_lerps_top[block];
                        }
                        for block in 0..NUM_BLOCKS {
                            let index: usize = base_index + block * ArchSimd::<f32>::LANES + ROW_SIZE * 2;
                            let output = y_lerp_3.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                            let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                            result.store_simd(index, val);
                            base_lerps_dif[block] += y_offset_lerps_dif[block];
                            base_lerps_top[block] += y_offset_lerps_top[block];
                        }
                        for block in 0..NUM_BLOCKS {
                            let index: usize = base_index + block * ArchSimd::<f32>::LANES + ROW_SIZE * 3;
                            let output = y_lerp_4.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                            let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                            result.store_simd(index, val);
                            base_lerps_dif[block] += y_offset_lerps_dif[block];
                            base_lerps_top[block] += y_offset_lerps_top[block];
                        }
                        y_it += 4;
                    } else {
                        let y_lerp = ArchSimd::splat(unsafe { interpolations.y.get_unchecked(y_it) });

                        for block in 0..NUM_BLOCKS {
                            let index: usize = x_it * MAP_SIZE + y_it * ROW_SIZE + z_it + block * ArchSimd::<f32>::LANES;
                            let output = y_lerp.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                            let val = if INITIALIZE { output } else { output + result.load_simd(index) };
                            result.store_simd(index, val);
                            base_lerps_dif[block] += y_offset_lerps_dif[block];
                            base_lerps_top[block] += y_offset_lerps_top[block];
                        }
                        y_it += 1;
                    }
                }
            }
            x_counter += 1.0;
        }
    }
}

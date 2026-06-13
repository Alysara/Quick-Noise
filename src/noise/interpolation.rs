// use crate::grid_helpers::grid_fill_indices;
// use crate::math::vec::Vec2;
// use crate::noise::perlin::constants::*;
// use crate::noise::perlin::containers::*;
// use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
// use crate::simd::simd_array::{SimdArray, TailInfo};
// use crate::simd::simd_traits::*;
//
// // ————————————————————————————————————————————————————————————————
// // ————— 2D Perlin Grid ———————————————————————————————————————————
// // ————————————————————————————————————————————————————————————————
//
// #[derive(Default)]
// pub struct LerpAccumulator {
//     base_dif: [ArchSimd<f32>; NUM_BLOCKS],
//     base_top: [ArchSimd<f32>; NUM_BLOCKS],
//     offset_dif: [ArchSimd<f32>; NUM_BLOCKS],
//     offset_top: [ArchSimd<f32>; NUM_BLOCKS],
// }
//
// impl LerpAccumulator {
//     fn accumulate(&mut self, block: usize) {
//         self.base_dif[block] += self.offset_dif[block];
//         self.base_top[block] += self.offset_top[block];
//     }
// }
//
// pub struct StaticLerp<const X: usize, const Y: usize, const Z: usize, const N: usize> {}
//
// const NUM_BLOCKS: usize = NUM_SIMD_REG / 8;
// const LANES: usize = ArchSimd::<f32>::LANES;
// const BLOCK_LANES: usize = NUM_BLOCKS * LANES;
//
// impl<const X: usize, const Y: usize, const Z: usize, const N: usize> StaticLerp<X, Y, Z, N> {
//     const HAS_SIMD_TAIL: bool = SimdArray::<f32, X>::HAS_TAIL;
//     const HAS_BLOCK_HEAD: bool = X >= BLOCK_LANES;
//     const HAS_BLOCK_TAIL: bool = (X % BLOCK_LANES) > 0;
//
//     const BLOCK_TAIL_SIZE: usize = (X % BLOCK_LANES + LANES - 1) / LANES;
//     const SIMD_TAIL_SIZE: usize = SimdArray::<f32, X>::TAIL_SIZE;
//
//     const BLOCK_TAIL_START: usize = (X / BLOCK_LANES) * BLOCK_LANES;
//     const SIMD_TAIL_START: usize = SimdArray::<f32, X>::TAIL_START;
//
//     #[inline(always)]
//     pub(super) fn grid_dotted_bilerp<const INITIALIZE: bool>(
//         gradients: &PerlinContainer2D<X>,
//         y_frac_start: f32,
//         y_increment: f32,
//         x_lerp_array: &SimdArray<f32, X>,
//         y_lerp_array: &SimdArray<f32, Y>,
//         y_start_index: usize,
//         y_end_index: usize,
//         weight: f32,
//         result: &mut SimdArray<f32, N>,
//     ) {
//         let weight_vec = ArchSimd::splat(weight);
//         let y_weighted_increment = ArchSimd::splat(y_increment * weight);
//         let y_upper_increment = ArchSimd::splat(y_frac_start);
//         let y_lower_increment = ArchSimd::splat(y_frac_start - 1.0);
//
//         // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
//         let mut base_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
//         let mut base_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
//         let mut y_offset_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
//         let mut y_offset_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
//
//         if Self::HAS_BLOCK_HEAD {
//             Self::grid_dotted_bilerp_helper::<INITIALIZE, false>(
//                 gradients,
//                 &mut base_dif,
//                 &mut base_top,
//                 &mut y_offset_dif,
//                 &mut y_offset_top,
//                 x_lerp_array,
//                 y_lerp_array,
//                 y_upper_increment,
//                 y_lower_increment,
//                 y_start_index,
//                 y_end_index,
//                 weight_vec,
//                 y_weighted_increment,
//                 result,
//             );
//         }
//
//         if Self::HAS_BLOCK_TAIL {
//             Self::grid_dotted_bilerp_helper::<INITIALIZE, true>(
//                 gradients,
//                 &mut base_dif,
//                 &mut base_top,
//                 &mut y_offset_dif,
//                 &mut y_offset_top,
//                 x_lerp_array,
//                 y_lerp_array,
//                 y_upper_increment,
//                 y_lower_increment,
//                 y_start_index,
//                 y_end_index,
//                 weight_vec,
//                 y_weighted_increment,
//                 result,
//             );
//         }
//     }
//
//     #[inline(always)]
//     fn grid_dotted_bilerp_helper<const INITIALIZE: bool, const IS_TAIL: bool>(
//         gradients: &PerlinContainer2D<X>,
//         base_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
//         base_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
//         y_offset_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
//         y_offset_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
//         x_lerp_array: &SimdArray<f32, X>,
//         y_lerp_array: &SimdArray<f32, Y>,
//         y_upper_increment: ArchSimd<f32>,
//         y_lower_increment: ArchSimd<f32>,
//         y_start_index: usize,
//         y_end_index: usize,
//         weight_vec: ArchSimd<f32>,
//         y_weighted_increment: ArchSimd<f32>,
//         result: &mut SimdArray<f32, N>,
//     ) {
//         let range = if IS_TAIL {
//             Self::BLOCK_TAIL_START..X
//         } else {
//             0..Self::BLOCK_TAIL_START
//         };
//
//         let num_blocks = if IS_TAIL {
//             Self::BLOCK_TAIL_SIZE
//         } else {
//             NUM_BLOCKS
//         };
//
//         for x_it in range.step_by(BLOCK_LANES) {
//             // These blocked loops will get entirely unrolled by the compiler.
//             for block in 0..num_blocks {
//                 // Load gradients into registers.
//                 let index = x_it + LANES * block;
//                 let x_lerp = x_lerp_array.load_simd_chunked::<IS_TAIL>(index);
//                 let x_tl = gradients.tl().x.load_simd_chunked::<IS_TAIL>(index);
//                 let x_tr = gradients.tr().x.load_simd_chunked::<IS_TAIL>(index);
//                 let x_bl = gradients.bl().x.load_simd_chunked::<IS_TAIL>(index);
//                 let x_br = gradients.br().x.load_simd_chunked::<IS_TAIL>(index);
//                 let y_tl = gradients.tl().y.load_simd_chunked::<IS_TAIL>(index);
//                 let y_tr = gradients.tr().y.load_simd_chunked::<IS_TAIL>(index);
//                 let y_bl = gradients.bl().y.load_simd_chunked::<IS_TAIL>(index);
//                 let y_br = gradients.br().y.load_simd_chunked::<IS_TAIL>(index);
//
//                 // Compute base dot products.
//                 let prod_sum_tl = y_tl.mul_add(y_upper_increment, x_tl);
//                 let prod_sum_tr = y_tr.mul_add(y_upper_increment, x_tr);
//                 let prod_sum_bl = y_bl.mul_add(y_lower_increment, x_bl);
//                 let prod_sum_br = y_br.mul_add(y_lower_increment, x_br);
//
//                 // Base interpolation.
//                 let prod_sum_top_dif = prod_sum_tr - prod_sum_tl;
//                 let prod_sum_low_dif = prod_sum_br - prod_sum_bl;
//                 base_top[block] = x_lerp.mul_add(prod_sum_top_dif, prod_sum_tl) * weight_vec;
//                 let base_lerp_bottom = x_lerp.mul_add(prod_sum_low_dif, prod_sum_bl) * weight_vec;
//                 base_dif[block] = base_lerp_bottom - base_top[block];
//
//                 // Offset interpolation.
//                 y_offset_top[block] = x_lerp.mul_add(y_tr - y_tl, y_tl) * y_weighted_increment;
//                 let y_offset_lerp_bottom = x_lerp.mul_add(y_br - y_bl, y_bl) * y_weighted_increment;
//                 y_offset_dif[block] = y_offset_lerp_bottom - y_offset_top[block];
//             }
//
//             let mut y_it = y_start_index;
//             while y_it < y_end_index {
//                 if y_it + 4 > y_end_index {
//                     Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
//                         base_dif,
//                         base_top,
//                         y_offset_dif,
//                         y_offset_top,
//                         x_it,
//                         y_it,
//                         &y_lerp_array,
//                         result,
//                         0,
//                     );
//                     y_it += 1;
//                 } else {
//                     for i in 0..4 {
//                         Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
//                             base_dif,
//                             base_top,
//                             y_offset_dif,
//                             y_offset_top,
//                             x_it,
//                             y_it,
//                             &y_lerp_array,
//                             result,
//                             i,
//                         );
//                     }
//                     y_it += 4;
//                 }
//             }
//         }
//     }
//
//     #[inline(always)]
//     pub(super) fn grid_dotted_trilerp<const INITIALIZE: bool>(
//         gradients: &PerlinContainer3D<X>,
//         z_frac_start: f32,
//         y_frac_start: f32,
//         z_increment: f32,
//         y_increment: f32,
//         z_lerp_array: &SimdArray<f32, Z>,
//         y_lerp_array: &SimdArray<f32, Y>,
//         x_lerp_array: &SimdArray<f32, X>,
//         z_start_index: usize,
//         y_start_index: usize,
//         z_end_index: usize,
//         y_end_index: usize,
//         weight: f32,
//         result: &mut SimdArray<f32, N>,
//     ) {
//         let weight_vec = ArchSimd::splat(weight);
//         let z_weighted_increment_vec = ArchSimd::splat(z_increment * weight);
//         let y_weighted_increment_vec = ArchSimd::splat(y_increment * weight);
//         let z_upper_increment = ArchSimd::splat(z_frac_start);
//         let z_lower_increment = ArchSimd::splat(z_frac_start - 1.0);
//         let y_upper_increment = ArchSimd::splat(y_frac_start);
//         let y_lower_increment = ArchSimd::splat(y_frac_start - 1.0);
//
//         let mut y_tf_offset = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut y_bf_offset = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut y_top_offset_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut y_bottom_offset_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
//
//         let mut z_tf_offset = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut z_bf_offset = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut z_top_offset_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut z_bottom_offset_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
//
//         let mut tf_base = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut bf_base = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut top_base_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
//         let mut bottom_base_dif = unsafe { SimdArray::<f32, X>::new_uninit() };
//
//         for x_it in (0..X).step_by(LANES) {
//             let x_lerp = x_lerp_array.load_simd_rw(x_it);
//
//             let z_tlf = gradients.tlf().z.load_simd_rw(x_it);
//             let z_trf = gradients.trf().z.load_simd_rw(x_it);
//             let z_blf = gradients.blf().z.load_simd_rw(x_it);
//             let z_brf = gradients.brf().z.load_simd_rw(x_it);
//             let z_tlb = gradients.tlb().z.load_simd_rw(x_it);
//             let z_trb = gradients.trb().z.load_simd_rw(x_it);
//             let z_blb = gradients.blb().z.load_simd_rw(x_it);
//             let z_brb = gradients.brb().z.load_simd_rw(x_it);
//
//             let y_tlf = gradients.tlf().y.load_simd_rw(x_it);
//             let y_trf = gradients.trf().y.load_simd_rw(x_it);
//             let y_blf = gradients.blf().y.load_simd_rw(x_it);
//             let y_brf = gradients.brf().y.load_simd_rw(x_it);
//             let y_tlb = gradients.tlb().y.load_simd_rw(x_it);
//             let y_trb = gradients.trb().y.load_simd_rw(x_it);
//             let y_blb = gradients.blb().y.load_simd_rw(x_it);
//             let y_brb = gradients.brb().y.load_simd_rw(x_it);
//
//             let x_tlf = gradients.tlf().x.load_simd_rw(x_it);
//             let x_trf = gradients.trf().x.load_simd_rw(x_it);
//             let x_blf = gradients.blf().x.load_simd_rw(x_it);
//             let x_brf = gradients.brf().x.load_simd_rw(x_it);
//             let x_tlb = gradients.tlb().x.load_simd_rw(x_it);
//             let x_trb = gradients.trb().x.load_simd_rw(x_it);
//             let x_blb = gradients.blb().x.load_simd_rw(x_it);
//             let x_brb = gradients.brb().x.load_simd_rw(x_it);
//
//             let sum_prod_tlf =
//                 z_upper_increment.mul_add(z_tlf, y_upper_increment.mul_add(y_tlf, x_tlf));
//             let sum_prod_trf =
//                 z_upper_increment.mul_add(z_trf, y_upper_increment.mul_add(y_trf, x_trf));
//             let sum_prod_blf =
//                 z_upper_increment.mul_add(z_blf, y_lower_increment.mul_add(y_blf, x_blf));
//             let sum_prod_brf =
//                 z_upper_increment.mul_add(z_brf, y_lower_increment.mul_add(y_brf, x_brf));
//             let sum_prod_tlb =
//                 z_lower_increment.mul_add(z_tlb, y_upper_increment.mul_add(y_tlb, x_tlb));
//             let sum_prod_trb =
//                 z_lower_increment.mul_add(z_trb, y_upper_increment.mul_add(y_trb, x_trb));
//             let sum_prod_blb =
//                 z_lower_increment.mul_add(z_blb, y_lower_increment.mul_add(y_blb, x_blb));
//             let sum_prod_brb =
//                 z_lower_increment.mul_add(z_brb, y_lower_increment.mul_add(y_brb, x_brb));
//
//             let z_tf_offset_vec = x_lerp.mul_add(z_trf - z_tlf, z_tlf) * z_weighted_increment_vec;
//             let z_bf_offset_vec = x_lerp.mul_add(z_brf - z_blf, z_blf) * z_weighted_increment_vec;
//             let z_tb_offset_vec = x_lerp.mul_add(z_trb - z_tlb, z_tlb) * z_weighted_increment_vec;
//             let z_bb_offset_vec = x_lerp.mul_add(z_brb - z_blb, z_blb) * z_weighted_increment_vec;
//
//             let y_tf_offset_vec = x_lerp.mul_add(y_trf - y_tlf, y_tlf) * y_weighted_increment_vec;
//             let y_bf_offset_vec = x_lerp.mul_add(y_brf - y_blf, y_blf) * y_weighted_increment_vec;
//             let y_hi_offset_dif_vec = x_lerp
//                 .mul_add(y_trb - y_tlb, y_tlb)
//                 .mul_sub(y_weighted_increment_vec, y_tf_offset_vec);
//             let y_lo_offset_dif_vec = x_lerp
//                 .mul_add(y_brb - y_blb, y_blb)
//                 .mul_sub(y_weighted_increment_vec, y_bf_offset_vec);
//
//             let tf_base_vec =
//                 x_lerp.mul_add(sum_prod_trf - sum_prod_tlf, sum_prod_tlf) * weight_vec;
//             let bf_base_vec =
//                 x_lerp.mul_add(sum_prod_brf - sum_prod_blf, sum_prod_blf) * weight_vec;
//             let hi_base_dif_vec = x_lerp
//                 .mul_add(sum_prod_trb - sum_prod_tlb, sum_prod_tlb)
//                 .mul_sub(weight_vec, tf_base_vec);
//             let lo_base_dif_vec = x_lerp
//                 .mul_add(sum_prod_brb - sum_prod_blb, sum_prod_blb)
//                 .mul_sub(weight_vec, bf_base_vec);
//
//             z_tf_offset.store_simd_rw(x_it, z_tf_offset_vec);
//             z_bf_offset.store_simd_rw(x_it, z_bf_offset_vec);
//             z_top_offset_dif.store_simd_rw(x_it, z_tb_offset_vec - z_tf_offset_vec);
//             z_bottom_offset_dif.store_simd_rw(x_it, z_bb_offset_vec - z_bf_offset_vec);
//
//             y_tf_offset.store_simd_rw(x_it, y_tf_offset_vec);
//             y_bf_offset.store_simd_rw(x_it, y_bf_offset_vec);
//             y_top_offset_dif.store_simd_rw(x_it, y_hi_offset_dif_vec);
//             y_bottom_offset_dif.store_simd_rw(x_it, y_lo_offset_dif_vec);
//
//             tf_base.store_simd_rw(x_it, tf_base_vec);
//             bf_base.store_simd_rw(x_it, bf_base_vec);
//             top_base_dif.store_simd_rw(x_it, hi_base_dif_vec);
//             bottom_base_dif.store_simd_rw(x_it, lo_base_dif_vec);
//         }
//
//         if Self::HAS_BLOCK_HEAD {
//             Self::grid_dotted_trilerp_helper::<INITIALIZE, false>(
//                 z_lerp_array,
//                 y_lerp_array,
//                 z_start_index,
//                 y_start_index,
//                 z_end_index,
//                 y_end_index,
//                 &y_tf_offset,
//                 &y_bf_offset,
//                 &y_top_offset_dif,
//                 &y_bottom_offset_dif,
//                 &z_tf_offset,
//                 &z_bf_offset,
//                 &z_top_offset_dif,
//                 &z_bottom_offset_dif,
//                 &tf_base,
//                 &bf_base,
//                 &top_base_dif,
//                 &bottom_base_dif,
//                 result,
//             );
//         }
//
//         if Self::HAS_BLOCK_TAIL {
//             Self::grid_dotted_trilerp_helper::<INITIALIZE, true>(
//                 z_lerp_array,
//                 y_lerp_array,
//                 z_start_index,
//                 y_start_index,
//                 z_end_index,
//                 y_end_index,
//                 &y_tf_offset,
//                 &y_bf_offset,
//                 &y_top_offset_dif,
//                 &y_bottom_offset_dif,
//                 &z_tf_offset,
//                 &z_bf_offset,
//                 &z_top_offset_dif,
//                 &z_bottom_offset_dif,
//                 &tf_base,
//                 &bf_base,
//                 &top_base_dif,
//                 &bottom_base_dif,
//                 result,
//             );
//         }
//     }
//
//     #[inline(always)]
//     pub(super) fn grid_dotted_trilerp_helper<const INITIALIZE: bool, const IS_TAIL: bool>(
//         z_lerp_array: &SimdArray<f32, Z>,
//         y_lerp_array: &SimdArray<f32, Y>,
//         z_start_index: usize,
//         y_start_index: usize,
//         z_end_index: usize,
//         y_end_index: usize,
//         y_tf_offset: &SimdArray<f32, X>,
//         y_bf_offset: &SimdArray<f32, X>,
//         y_top_offset_dif: &SimdArray<f32, X>,
//         y_bottom_offset_dif: &SimdArray<f32, X>,
//         z_tf_offset: &SimdArray<f32, X>,
//         z_bf_offset: &SimdArray<f32, X>,
//         z_top_offset_dif: &SimdArray<f32, X>,
//         z_bottom_offset_dif: &SimdArray<f32, X>,
//         tf_base: &SimdArray<f32, X>,
//         bf_base: &SimdArray<f32, X>,
//         top_base_dif: &SimdArray<f32, X>,
//         bottom_base_dif: &SimdArray<f32, X>,
//         result: &mut SimdArray<f32, N>,
//     ) {
//         let range = if IS_TAIL {
//             Self::BLOCK_TAIL_START..X
//         } else {
//             0..Self::BLOCK_TAIL_START
//         };
//
//         let num_blocks = if IS_TAIL {
//             Self::BLOCK_TAIL_SIZE
//         } else {
//             NUM_BLOCKS
//         };
//
//         let mut z_counter: f32 = 0.0;
//         for z_it in z_start_index..z_end_index {
//             let z_lerp = ArchSimd::splat(unsafe { z_lerp_array.get_unchecked(z_it) });
//             let z_cur_vec = ArchSimd::splat(z_counter);
//
//             for x_it in range.clone().step_by(BLOCK_LANES) {
//                 // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
//                 let mut accumulator = LerpAccumulator::default();
//
//                 // These blocked loops will get entirely unrolled by the compiler.
//                 for block in 0..num_blocks {
//                     let index = x_it + LANES * block;
//                     let z_tf_offset_vec = z_tf_offset.load_simd(index);
//                     let z_bf_offset_vec = z_bf_offset.load_simd(index);
//                     let z_top_offset_dif_vec = z_top_offset_dif.load_simd(index);
//                     let z_bottom_offset_dif_vec = z_bottom_offset_dif.load_simd(index);
//
//                     let y_tf_offset_vec = y_tf_offset.load_simd(index);
//                     let y_bf_offset_vec = y_bf_offset.load_simd(index);
//                     let y_top_offset_dif_vec = y_top_offset_dif.load_simd(index);
//                     let y_bottom_offset_dif_vec = y_bottom_offset_dif.load_simd(index);
//
//                     let tf_base_vec = tf_base.load_simd(index);
//                     let bf_base_vec = bf_base.load_simd(index);
//                     let top_base_dif_vec = top_base_dif.load_simd(index);
//                     let bottom_base_dif_vec = bottom_base_dif.load_simd(index);
//
//                     let z_top_offset = z_lerp.mul_add(z_top_offset_dif_vec, z_tf_offset_vec);
//                     let z_bottom_offset = z_lerp.mul_add(z_bottom_offset_dif_vec, z_bf_offset_vec);
//
//                     accumulator.base_top[block] = z_cur_vec
//                         .mul_add(z_top_offset, z_lerp.mul_add(top_base_dif_vec, tf_base_vec));
//                     let bottom_base = z_cur_vec.mul_add(
//                         z_bottom_offset,
//                         z_lerp.mul_add(bottom_base_dif_vec, bf_base_vec),
//                     );
//                     accumulator.base_dif[block] = bottom_base - accumulator.base_top[block];
//
//                     accumulator.offset_top[block] =
//                         z_lerp.mul_add(y_top_offset_dif_vec, y_tf_offset_vec);
//                     let y_bottom_offset = z_lerp.mul_add(y_bottom_offset_dif_vec, y_bf_offset_vec);
//                     accumulator.offset_dif[block] = y_bottom_offset - accumulator.offset_top[block];
//                 }
//
//                 let mut y_it = y_start_index;
//                 while y_it < y_end_index {
//                     if y_it + 4 > y_end_index {
//                         Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
//                             &mut accumulator,
//                             z_it,
//                             y_it,
//                             x_it,
//                             y_lerp_array,
//                             result,
//                             0,
//                         );
//                         y_it += 1;
//                     } else {
//                         for i in 0..4 {
//                             Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
//                                 &mut accumulator,
//                                 z_it,
//                                 y_it,
//                                 x_it,
//                                 y_lerp_array,
//                                 result,
//                                 i,
//                             );
//                         }
//                         y_it += 4;
//                     }
//                 }
//             }
//             z_counter += 1.0;
//         }
//     }
//
//     #[inline(always)]
//     fn process_lerp_block<const INITIALIZE: bool, const IS_TAIL: bool>(
//         accumulator: &mut LerpAccumulator,
//         z_it: usize,
//         y_it: usize,
//         x_it: usize,
//         y_lerp_array: &SimdArray<f32, Y>,
//         result: &mut SimdArray<f32, N>,
//         y_idx: usize,
//     ) {
//         let y_lerp = ArchSimd::splat(unsafe { y_lerp_array.get_unchecked(y_it + y_idx) });
//
//         let range = if IS_TAIL {
//             0..Self::BLOCK_TAIL_SIZE
//         } else {
//             0..NUM_BLOCKS
//         };
//
//         let base_index = z_it * X * Y + y_it * X;
//         for block in range {
//             let x_index = x_it + block * LANES;
//             let index = base_index + x_index + X * y_idx;
//             let output = y_lerp.mul_add(accumulator.base_dif[block], accumulator.base_top[block]);
//
//             let val = if INITIALIZE {
//                 output
//             } else {
//                 unsafe { output + result.load_simd_tail_checked(index) }
//             };
//
//             unsafe {
//                 if IS_TAIL && Self::HAS_SIMD_TAIL && x_index >= Self::SIMD_TAIL_START {
//                     result.partial_store_simd_unchecked(index, val, Self::SIMD_TAIL_SIZE);
//                 } else {
//                     result.store_simd_unchecked(index, val)
//                 };
//             }
//
//             accumulator.accumulate(block);
//         }
//     }
//
//     // #[inline(always)]
//     // fn process_lerp_block<const INITIALIZE: bool, const IS_TAIL: bool>(
//     //     base_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
//     //     base_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
//     //     y_offset_dif: &[ArchSimd<f32>; NUM_BLOCKS],
//     //     y_offset_top: &[ArchSimd<f32>; NUM_BLOCKS],
//     //     x_it: usize,
//     //     y_it: usize,
//     //     y_lerp_array: &SimdArray<f32, Y>,
//     //     result: &mut SimdArray<f32, N>,
//     //     y_idx: usize,
//     // ) {
//     //     let y_lerp = ArchSimd::splat(unsafe { y_lerp_array.get_unchecked(y_it + y_idx) });
//     //
//     //     let range = if IS_TAIL {
//     //         0..Self::BLOCK_TAIL_SIZE
//     //     } else {
//     //         0..NUM_BLOCKS
//     //     };
//     //
//     //     for block in range {
//     //         let x_index = x_it + LANES * block;
//     //         let index = x_index + y_it * X + X * y_idx;
//     //         let output = y_lerp.mul_add(base_dif[block], base_top[block]);
//     //
//     //         let val = if INITIALIZE {
//     //             output
//     //         } else {
//     //             unsafe { output + result.load_simd_tail_checked(index) }
//     //         };
//     //
//     //         unsafe {
//     //             if IS_TAIL && Self::HAS_SIMD_TAIL && x_index >= Self::SIMD_TAIL_START {
//     //                 result.partial_store_simd_unchecked(index, val, Self::SIMD_TAIL_SIZE);
//     //             } else {
//     //                 result.store_simd_unchecked(index, val)
//     //             };
//     //         }
//     //
//     //         base_dif[block] += y_offset_dif[block];
//     //         base_top[block] += y_offset_top[block];
//     //     }
//     // }
// }

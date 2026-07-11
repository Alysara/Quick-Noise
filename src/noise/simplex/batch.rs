use std::f32::consts::SQRT_2;

use crate::api::batch::interface::BatchNoiseImpl;
use crate::simd::arch_simd::ArchSimd;
use crate::simplex::Simplex;

const SQRT_3: f32 = 1.7320508075688772935274;
const SKEW_2D: f32 = (SQRT_3 - 1.0) / 2.0;
const UNSKEW_2D: f32 = (3.0 - SQRT_3) / 6.0;
const SKEW_3D: f32 = 1.0 / 3.0;
const UNSKEW_3D: f32 = 1.0 / 6.0;

const SCALE: f32 = 80.0;
const SCALED_SQRT: f32 = (SQRT_2 / 2.0) * SCALE;

const A: f32 = SCALE;
const B: f32 = SCALED_SQRT;
const C: f32 = 0.0;
pub const X_GRADIENTS_2D: [f32; 8] = [A, B, C, -B, -A, -B, C, B];
pub const Y_GRADIENTS_2D: [f32; 8] = [C, B, A, B, C, -B, -A, -B];

// pub const X_GRADIENTS_2D: [f32; 8] = [
//     1.4142135623730951,
//     1.0000000000000000,
//     0.0000000000000000,
//    -1.0000000000000000,
//    -1.4142135623730951,
//    -1.0000000000000000,
//     0.0000000000000000,
//     1.0000000000000000,
// ];

// pub const Y_GRADIENTS_2D: [f32; 8] = [
//     0.0000000000000000,
//     1.0000000000000000,
//     1.4142135623730951,
//     1.0000000000000000,
//     0.0000000000000000,
//    -1.0000000000000000,
//    -1.4142135623730951,
//    -1.0000000000000000,
// ];

impl BatchNoiseImpl<2> for Simplex {
    fn sample_batch(
        seed: u32,
        input: [ArchSimd<f32>; 2],
        freq: [ArchSimd<f32>; 2],
    ) -> ArchSimd<f32> {
        // Constants.
        let skew: ArchSimd<f32> = ArchSimd::splat(SKEW_2D);
        let unskew: ArchSimd<f32> = ArchSimd::splat(UNSKEW_2D);
        let subbed_unskew: ArchSimd<f32> = ArchSimd::splat(UNSKEW_2D - 1.0);
        let hi_skew_offset: ArchSimd<f32> = ArchSimd::splat(2.0 * UNSKEW_2D - 1.0);
        let half: ArchSimd<f32> = ArchSimd::splat(0.5);
        let zero: ArchSimd<f32> = ArchSimd::splat(0.0);
        let t_hi_coef = ArchSimd::splat(2.0 * SQRT_3 / 3.0);
        let neg_two_thirds = ArchSimd::splat(-2.0 / 3.0);

        // Hash constants.
        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
        let channel_seed = ArchSimd::splat(seed as u32);
        let prime = ArchSimd::splat(0x85ebca6b_u32 as u32);

        let x_scaled = input[0] * freq[0];
        let y_scaled = input[1] * freq[1];

        // Gridpoints and distances: 19
        let s = (x_scaled + y_scaled) * skew;
        let x_grid = (x_scaled + s).floor();
        let y_grid = (y_scaled + s).floor();

        let unskew_sub = (x_grid + y_grid) * unskew;
        let x_dist_lo = x_scaled - x_grid + unskew_sub;
        let y_dist_lo = y_scaled - y_grid + unskew_sub;
        let triangle_mask = x_dist_lo.simd_gt(y_dist_lo);

        let x_dist_mi_offset = triangle_mask.select(subbed_unskew, unskew);
        let y_dist_mi_offset = triangle_mask.select(unskew, subbed_unskew);
        let x_dist_mi = x_dist_lo + x_dist_mi_offset;
        let y_dist_mi = y_dist_lo + y_dist_mi_offset;

        let x_dist_hi = x_dist_lo + hi_skew_offset;
        let y_dist_hi = y_dist_lo + hi_skew_offset;

        // Hash: 22
        let x1: ArchSimd<u32> = x_grid.cast_int_trunc().raw_cast() * channel_seed;
        let y1: ArchSimd<u32> = y_grid.cast_int_trunc().raw_cast() * channel_seed;
        let x2 = x1 + channel_seed;
        let y2 = y1 + channel_seed;

        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;

        let mix_lo = (x1_shuf * y1_shuf) ^ x1_shuf;
        let mix_hi = (x2_shuf * y2_shuf) ^ x2_shuf;

        let x_shuf_mi = triangle_mask.raw_cast().select(x2_shuf, x1_shuf);
        let y_shuf_mi = triangle_mask.raw_cast().select(y1_shuf, y2_shuf);
        let mix_mi = (x_shuf_mi * y_shuf_mi) ^ x_shuf_mi;

        // Gradient lookup: 9
        let indices_lo = mix_lo >> 29;
        let indices_mi = mix_mi >> 29;
        let indices_hi = mix_hi >> 29;

        let x_grads_lo = indices_lo.gather(&X_GRADIENTS_2D);
        let y_grads_lo = indices_lo.gather(&Y_GRADIENTS_2D);
        let x_grads_mi = indices_mi.gather(&X_GRADIENTS_2D);
        let y_grads_mi = indices_mi.gather(&Y_GRADIENTS_2D);
        let x_grads_hi = indices_hi.gather(&X_GRADIENTS_2D);
        let y_grads_hi = indices_hi.gather(&Y_GRADIENTS_2D);

        // Sum of products: 27
        let t_lo_pre = half - x_dist_lo.mul_add(x_dist_lo, y_dist_lo * y_dist_lo);
        let t_mi_pre = half - x_dist_mi.mul_add(x_dist_mi, y_dist_mi * y_dist_mi);
        let t_hi_pre = t_lo_pre + t_hi_coef.mul_add(x_dist_lo + y_dist_lo, neg_two_thirds);

        let t_lo = t_lo_pre.max(zero);
        let t_mi = t_mi_pre.max(zero);
        let t_hi = t_hi_pre.max(zero);

        let t2_lo = t_lo * t_lo;
        let t2_mi = t_mi * t_mi;
        let t2_hi = t_hi * t_hi;

        let t4_lo = t2_lo * t2_lo;
        let t4_mi = t2_mi * t2_mi;
        let t4_hi = t2_hi * t2_hi;

        let dot_lo = x_grads_lo.mul_add(x_dist_lo, y_grads_lo * y_dist_lo);
        let dot_mi = x_grads_mi.mul_add(x_dist_mi, y_grads_mi * y_dist_mi);
        let dot_hi = x_grads_hi.mul_add(x_dist_hi, y_grads_hi * y_dist_hi);

        let result = t4_lo.mul_add(dot_lo, t4_mi.mul_add(dot_mi, t4_hi * dot_hi));

        result
    }
}

impl BatchNoiseImpl<3> for Simplex {
    fn sample_batch(
        seed: u32,
        input: [ArchSimd<f32>; 3],
        freq: [ArchSimd<f32>; 3],
    ) -> ArchSimd<f32> {
        // Constants.
        let skew: ArchSimd<f32> = ArchSimd::splat(SKEW_3D);
        let unskew: ArchSimd<f32> = ArchSimd::splat(UNSKEW_3D);
        let subbed_unskew: ArchSimd<f32> = ArchSimd::splat(UNSKEW_3D - 1.0);
        let hi_skew_offset: ArchSimd<f32> = ArchSimd::splat(3.0 * UNSKEW_3D - 1.0);
        let two_unskew: ArchSimd<f32> = ArchSimd::splat(2.0 * UNSKEW_3D);
        let mi2_skew_offset: ArchSimd<f32> = ArchSimd::splat(2.0 * UNSKEW_3D - 1.0);
        let half: ArchSimd<f32> = ArchSimd::splat(0.5);
        let zero: ArchSimd<f32> = ArchSimd::splat(0.0);
        let three_int: ArchSimd<u32> = ArchSimd::splat(3);

        let c1: ArchSimd<u32> = ArchSimd::splat(0x09009999);
        let c2: ArchSimd<u32> = ArchSimd::splat(0xA59900A5);
        let c3: ArchSimd<u32> = ArchSimd::splat(0x90A5A500);

        // Hash constants.
        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        // TODO: Figure out what this needs to be.
        const S: f32 = 100.0;
        const GRAD_TABLE: [f32; 4] = [0.0, S, -S, 0.0];

        let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
        let channel_seed = ArchSimd::splat(seed as u32);
        let prime = ArchSimd::splat(0x85ebca6b_u32 as u32);

        // Scale: 3
        let x_scaled = input[0] * freq[0];
        let y_scaled = input[1] * freq[1];
        let z_scaled = input[2] * freq[2];

        // Gridpoints and distances: 39
        let s = (x_scaled + y_scaled + z_scaled) * skew;
        let x_grid = (x_scaled + s).floor();
        let y_grid = (y_scaled + s).floor();
        let z_grid = (z_scaled + s).floor();

        let unskew_sub = (x_grid + y_grid + z_grid) * unskew;
        let x_dist_lo = x_scaled - x_grid + unskew_sub;
        let y_dist_lo = y_scaled - y_grid + unskew_sub;
        let z_dist_lo = z_scaled - z_grid + unskew_sub;

        let x_gt_y = x_dist_lo.simd_gt(y_dist_lo);
        let x_gt_z = x_dist_lo.simd_gt(z_dist_lo);
        let ny_gt_z = y_dist_lo.simd_le(z_dist_lo);

        let nx_gt_y = x_dist_lo.simd_le(y_dist_lo);
        let nx_gt_z = x_dist_lo.simd_le(z_dist_lo);
        let y_gt_z = y_dist_lo.simd_gt(z_dist_lo);

        let i1 = x_gt_y & x_gt_z;
        let j1 = nx_gt_y & y_gt_z;
        let k1 = nx_gt_z & ny_gt_z;

        let i2 = x_gt_y | x_gt_z;
        let j2 = nx_gt_y | y_gt_z;
        let k2 = nx_gt_z | ny_gt_z;

        let x_dist_mi1 = x_dist_lo + i1.select(subbed_unskew, unskew);
        let y_dist_mi1 = y_dist_lo + j1.select(subbed_unskew, unskew);
        let z_dist_mi1 = z_dist_lo + k1.select(subbed_unskew, unskew);

        let x_dist_mi2 = x_dist_lo + i2.select(mi2_skew_offset, two_unskew);
        let y_dist_mi2 = y_dist_lo + j2.select(mi2_skew_offset, two_unskew);
        let z_dist_mi2 = z_dist_lo + k2.select(mi2_skew_offset, two_unskew);

        let x_dist_hi = x_dist_lo + hi_skew_offset;
        let y_dist_hi = y_dist_lo + hi_skew_offset;
        let z_dist_hi = z_dist_lo + hi_skew_offset;

        // Hash: 35
        let x1: ArchSimd<u32> = x_grid.cast_int_trunc().raw_cast() * channel_seed;
        let y1: ArchSimd<u32> = y_grid.cast_int_trunc().raw_cast() * channel_seed;
        let z1: ArchSimd<u32> = z_grid.cast_int_trunc().raw_cast() * channel_seed;
        let x2 = x1 + channel_seed;
        let y2 = y1 + channel_seed;
        let z2 = z1 + channel_seed;

        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
        let z1_shuf = z1.permute_8(shuffle_indices) ^ prime;

        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
        let z2_shuf = z2.permute_8(shuffle_indices) ^ prime;

        let x_mi1_shuf = i1.raw_cast().select(x2_shuf, x1_shuf);
        let y_mi1_shuf = j1.raw_cast().select(y2_shuf, y1_shuf);
        let z_mi1_shuf = k1.raw_cast().select(z2_shuf, z1_shuf);

        let x_mi2_shuf = i2.raw_cast().select(x2_shuf, x1_shuf);
        let y_mi2_shuf = j2.raw_cast().select(y2_shuf, y1_shuf);
        let z_mi2_shuf = k2.raw_cast().select(z2_shuf, z1_shuf);

        let mix_lo = x1_shuf * y1_shuf * z1_shuf;
        let mix_hi = x2_shuf * y2_shuf * z2_shuf;
        let mix_mi1 = x_mi1_shuf * y_mi1_shuf * z_mi1_shuf;
        let mix_mi2 = x_mi2_shuf * y_mi2_shuf * z_mi2_shuf;

        // Gradient lookup: 44
        let indices_lo = (mix_lo >> 28) << 1;
        let indices_mi1 = (mix_mi1 >> 28) << 1;
        let indices_mi2 = (mix_mi2 >> 28) << 1;
        let indices_hi = (mix_hi >> 28) << 1;

        let x_grads_lo = ((c1 >> indices_lo) & three_int).gather(&GRAD_TABLE);
        let y_grads_lo = ((c2 >> indices_lo) & three_int).gather(&GRAD_TABLE);
        let z_grads_lo = ((c3 >> indices_lo) & three_int).gather(&GRAD_TABLE);
        let x_grads_mi1 = ((c1 >> indices_mi1) & three_int).gather(&GRAD_TABLE);
        let y_grads_mi1 = ((c2 >> indices_mi1) & three_int).gather(&GRAD_TABLE);
        let z_grads_mi1 = ((c3 >> indices_mi1) & three_int).gather(&GRAD_TABLE);
        let x_grads_mi2 = ((c1 >> indices_mi2) & three_int).gather(&GRAD_TABLE);
        let y_grads_mi2 = ((c2 >> indices_mi2) & three_int).gather(&GRAD_TABLE);
        let z_grads_mi2 = ((c3 >> indices_mi2) & three_int).gather(&GRAD_TABLE);
        let x_grads_hi = ((c1 >> indices_hi) & three_int).gather(&GRAD_TABLE);
        let y_grads_hi = ((c2 >> indices_hi) & three_int).gather(&GRAD_TABLE);
        let z_grads_hi = ((c3 >> indices_hi) & three_int).gather(&GRAD_TABLE);

        // Sum of products: 44
        let t_lo = (half
            - x_dist_lo.mul_add(
                x_dist_lo,
                y_dist_lo.mul_add(y_dist_lo, z_dist_lo * z_dist_lo),
            ))
        .max(zero);
        let t_mi1 = (half
            - x_dist_mi1.mul_add(
                x_dist_mi1,
                y_dist_mi1.mul_add(y_dist_mi1, z_dist_mi1 * z_dist_mi1),
            ))
        .max(zero);
        let t_mi2 = (half
            - x_dist_mi2.mul_add(
                x_dist_mi2,
                y_dist_mi2.mul_add(y_dist_mi2, z_dist_mi2 * z_dist_mi2),
            ))
        .max(zero);
        let t_hi = (half
            - x_dist_hi.mul_add(
                x_dist_hi,
                y_dist_hi.mul_add(y_dist_hi, z_dist_hi * z_dist_hi),
            ))
        .max(zero);

        let t2_lo = t_lo * t_lo;
        let t2_mi1 = t_mi1 * t_mi1;
        let t2_mi2 = t_mi2 * t_mi2;
        let t2_hi = t_hi * t_hi;

        let t4_lo = t2_lo * t2_lo;
        let t4_mi1 = t2_mi1 * t2_mi1;
        let t4_mi2 = t2_mi2 * t2_mi2;
        let t4_hi = t2_hi * t2_hi;

        let dot_lo = x_grads_lo.mul_add(
            x_dist_lo,
            y_grads_lo.mul_add(y_dist_lo, z_dist_lo * z_grads_lo),
        );
        let dot_mi1 = x_grads_mi1.mul_add(
            x_dist_mi1,
            y_grads_mi1.mul_add(y_dist_mi1, z_dist_mi1 * z_grads_mi1),
        );
        let dot_mi2 = x_grads_mi2.mul_add(
            x_dist_mi2,
            y_grads_mi2.mul_add(y_dist_mi2, z_dist_mi2 * z_grads_mi2),
        );
        let dot_hi = x_grads_hi.mul_add(
            x_dist_hi,
            y_grads_hi.mul_add(y_dist_hi, z_dist_hi * z_grads_hi),
        );

        let result = t4_lo.mul_add(
            dot_lo,
            t4_mi1.mul_add(dot_mi1, t4_mi2.mul_add(dot_mi2, t4_hi * dot_hi)),
        );

        result
    }
}

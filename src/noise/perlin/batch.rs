use crate::api::batch::interface::BatchNoise;
use crate::perlin::Perlin;
use crate::perlin::constants::{X_GRADIENTS_2D, Y_GRADIENTS_2D};
use crate::simd::arch_simd::ArchSimd;

impl BatchNoise<2> for Perlin {
    fn sample_batch(seed: u32, input: [ArchSimd<f32>; 2], freq: [ArchSimd<f32>; 2]) -> ArchSimd<f32> {
        // Constants.
        let six: ArchSimd<f32> = ArchSimd::splat(6.0);
        let ten: ArchSimd<f32> = ArchSimd::splat(10.0);
        let fifteen: ArchSimd<f32> = ArchSimd::splat(15.0);
        let one: ArchSimd<f32> = ArchSimd::splat(1.0);

        // Hash constants.
        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
        let channel_seed = ArchSimd::splat(seed);
        let prime = ArchSimd::splat(0x85ebca6b_u32);

        // Scale: 2
        let x_scaled = input[0] * freq[0];
        let y_scaled = input[1] * freq[1];

        // Gridpoints and distances: 8
        let x_scaled_floored = x_scaled.floor();
        let y_scaled_floored = y_scaled.floor();

        let x_grid_lo = x_scaled_floored.cast_int_trunc();
        let y_grid_lo = y_scaled_floored.cast_int_trunc();

        let x_dist_lo = x_scaled - x_scaled_floored;
        let y_dist_lo = y_scaled - y_scaled_floored;
        let x_dist_hi = x_dist_lo - one;
        let y_dist_hi = y_dist_lo - one;

        // Lerp fade calculation: 10
        let t = x_dist_lo;
        let s = y_dist_lo;
        let x_lerp = t * t * t * t.mul_add(t.mul_sub(six, fifteen), ten);
        let y_lerp = s * s * s * s.mul_add(s.mul_sub(six, fifteen), ten);

        // Hash: 16
        let x1: ArchSimd<u32> = x_grid_lo.raw_cast() * channel_seed;
        let y1: ArchSimd<u32> = y_grid_lo.raw_cast() * channel_seed;
        let x2 = x1 + channel_seed;
        let y2 = y1 + channel_seed;

        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;

        let mix_tl = x1_shuf * y1_shuf;
        let mix_tr = x1_shuf * y2_shuf;
        let mix_bl = x2_shuf * y1_shuf;
        let mix_br = x2_shuf * y2_shuf;

        // Permute Gather: 12
        let indices_tl = mix_tl >> 29;
        let indices_tr = mix_tr >> 29;
        let indices_bl = mix_bl >> 29;
        let indices_br = mix_br >> 29;

        let x_grads_tl = indices_tl.gather(&X_GRADIENTS_2D);
        let y_grads_tl = indices_tl.gather(&Y_GRADIENTS_2D);
        let x_grads_tr = indices_tr.gather(&X_GRADIENTS_2D);
        let y_grads_tr = indices_tr.gather(&Y_GRADIENTS_2D);
        let x_grads_bl = indices_bl.gather(&X_GRADIENTS_2D);
        let y_grads_bl = indices_bl.gather(&Y_GRADIENTS_2D);
        let x_grads_br = indices_br.gather(&X_GRADIENTS_2D);
        let y_grads_br = indices_br.gather(&Y_GRADIENTS_2D);

        // Interpolation: 14
        let prod_tl = x_grads_tl.mul_add(x_dist_lo, y_grads_tl * y_dist_lo);
        let prod_tr = x_grads_tr.mul_add(x_dist_lo, y_grads_tr * y_dist_hi);
        let top_lerp = y_lerp.mul_add(prod_tr - prod_tl, prod_tl);

        let prod_bl = x_grads_bl.mul_add(x_dist_hi, y_grads_bl * y_dist_lo);
        let prod_br = x_grads_br.mul_add(x_dist_hi, y_grads_br * y_dist_hi);
        let bottom_lerp = y_lerp.mul_add(prod_br - prod_bl, prod_bl);

        x_lerp.mul_add(bottom_lerp - top_lerp, top_lerp)
    }
}

impl BatchNoise<3> for Perlin {
    fn sample_batch(seed: u32, input: [ArchSimd<f32>; 3], freq: [ArchSimd<f32>; 3]) -> ArchSimd<f32> {
        // Constants.
        let six: ArchSimd<f32> = ArchSimd::splat(6.0);
        let ten: ArchSimd<f32> = ArchSimd::splat(10.0);
        let fifteen: ArchSimd<f32> = ArchSimd::splat(15.0);
        let one: ArchSimd<f32> = ArchSimd::splat(1.0);
        let three_int: ArchSimd<u32> = ArchSimd::splat(3);

        let c1: ArchSimd<u32> = ArchSimd::splat(0x90A5A500);
        let c2: ArchSimd<u32> = ArchSimd::splat(0xA59900A5);
        let c3: ArchSimd<u32> = ArchSimd::splat(0x09009999);

        // Hash constants.
        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        const GRAD_TABLE: [f32; 4] = [0.0, 1.0, -1.0, 0.0];

        // X: 0000 1001 0000 0000 1001 1001 1001 1001
        // Y: 1010 0101 1001 1001 0000 0000 1010 0101
        // Z: 1001 0000 1010 0101 1010 0101 0000 0000
        //
        // X: 09009999
        // Y: A59900A5
        // Z: 90A5A500

        let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
        let channel_seed = ArchSimd::splat(seed);
        let prime = ArchSimd::splat(0x85ebca6b_u32);

        // Scale: 3
        let x_scaled = input[0] * freq[0];
        let y_scaled = input[1] * freq[1];
        let z_scaled = input[2] * freq[2];

        // Gridpoints and distances: 12
        let x_scaled_floored = x_scaled.floor();
        let y_scaled_floored = y_scaled.floor();
        let z_scaled_floored = z_scaled.floor();

        let x_grid_lo = x_scaled_floored.cast_int_trunc();
        let y_grid_lo = y_scaled_floored.cast_int_trunc();
        let z_grid_lo = z_scaled_floored.cast_int_trunc();

        let x_dist_lo = x_scaled - x_scaled_floored;
        let y_dist_lo = y_scaled - y_scaled_floored;
        let z_dist_lo = z_scaled - z_scaled_floored;
        let x_dist_hi = x_dist_lo - one;
        let y_dist_hi = y_dist_lo - one;
        let z_dist_hi = z_dist_lo - one;

        // Lerp fade calculation: 15
        let t = x_dist_lo;
        let s = y_dist_lo;
        let u = z_dist_lo;
        let x_lerp = t * t * t * t.mul_add(t.mul_sub(six, fifteen), ten);
        let y_lerp = s * s * s * s.mul_add(s.mul_sub(six, fifteen), ten);
        let z_lerp = u * u * u * u.mul_add(u.mul_sub(six, fifteen), ten);

        // Hash: 26
        let x1: ArchSimd<u32> = x_grid_lo.raw_cast() * channel_seed;
        let y1: ArchSimd<u32> = y_grid_lo.raw_cast() * channel_seed;
        let z1: ArchSimd<u32> = z_grid_lo.raw_cast() * channel_seed;
        let x2 = x1 + channel_seed;
        let y2 = y1 + channel_seed;
        let z2 = z1 + channel_seed;

        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
        let z1_shuf = z1.permute_8(shuffle_indices) ^ prime;
        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;
        let z2_shuf = z2.permute_8(shuffle_indices) ^ prime;

        let mix_tlf = x1_shuf * y1_shuf * z1_shuf;
        let mix_trf = x1_shuf * y1_shuf * z2_shuf;
        let mix_blf = x1_shuf * y2_shuf * z1_shuf;
        let mix_brf = x1_shuf * y2_shuf * z2_shuf;
        let mix_tlb = x2_shuf * y1_shuf * z1_shuf;
        let mix_trb = x2_shuf * y1_shuf * z2_shuf;
        let mix_blb = x2_shuf * y2_shuf * z1_shuf;
        let mix_brb = x2_shuf * y2_shuf * z2_shuf;

        // Products: 88
        let indices_tlf = (mix_tlf >> 28) << 1;
        let indices_trf = (mix_trf >> 28) << 1;
        let indices_blf = (mix_blf >> 28) << 1;
        let indices_brf = (mix_brf >> 28) << 1;
        let indices_tlb = (mix_tlb >> 28) << 1;
        let indices_trb = (mix_trb >> 28) << 1;
        let indices_blb = (mix_blb >> 28) << 1;
        let indices_brb = (mix_brb >> 28) << 1;

        let x_grads_tlf = ((c1 >> indices_tlf) & three_int).gather(&GRAD_TABLE);
        let x_grads_trf = ((c1 >> indices_trf) & three_int).gather(&GRAD_TABLE);
        let x_grads_blf = ((c1 >> indices_blf) & three_int).gather(&GRAD_TABLE);
        let x_grads_brf = ((c1 >> indices_brf) & three_int).gather(&GRAD_TABLE);
        let x_grads_tlb = ((c1 >> indices_tlb) & three_int).gather(&GRAD_TABLE);
        let x_grads_trb = ((c1 >> indices_trb) & three_int).gather(&GRAD_TABLE);
        let x_grads_blb = ((c1 >> indices_blb) & three_int).gather(&GRAD_TABLE);
        let x_grads_brb = ((c1 >> indices_brb) & three_int).gather(&GRAD_TABLE);
        let y_grads_tlf = ((c2 >> indices_tlf) & three_int).gather(&GRAD_TABLE);
        let y_grads_trf = ((c2 >> indices_trf) & three_int).gather(&GRAD_TABLE);
        let y_grads_blf = ((c2 >> indices_blf) & three_int).gather(&GRAD_TABLE);
        let y_grads_brf = ((c2 >> indices_brf) & three_int).gather(&GRAD_TABLE);
        let y_grads_tlb = ((c2 >> indices_tlb) & three_int).gather(&GRAD_TABLE);
        let y_grads_trb = ((c2 >> indices_trb) & three_int).gather(&GRAD_TABLE);
        let y_grads_blb = ((c2 >> indices_blb) & three_int).gather(&GRAD_TABLE);
        let y_grads_brb = ((c2 >> indices_brb) & three_int).gather(&GRAD_TABLE);
        let z_grads_tlf = ((c3 >> indices_tlf) & three_int).gather(&GRAD_TABLE);
        let z_grads_trf = ((c3 >> indices_trf) & three_int).gather(&GRAD_TABLE);
        let z_grads_blf = ((c3 >> indices_blf) & three_int).gather(&GRAD_TABLE);
        let z_grads_brf = ((c3 >> indices_brf) & three_int).gather(&GRAD_TABLE);
        let z_grads_tlb = ((c3 >> indices_tlb) & three_int).gather(&GRAD_TABLE);
        let z_grads_trb = ((c3 >> indices_trb) & three_int).gather(&GRAD_TABLE);
        let z_grads_blb = ((c3 >> indices_blb) & three_int).gather(&GRAD_TABLE);
        let z_grads_brb = ((c3 >> indices_brb) & three_int).gather(&GRAD_TABLE);

        // Interpolation: 38
        let prod_tlf = x_grads_tlf.mul_add(
            x_dist_lo,
            y_grads_tlf.mul_add(y_dist_lo, z_grads_tlf * z_dist_lo),
        );
        let prod_trf = x_grads_trf.mul_add(
            x_dist_lo,
            y_grads_trf.mul_add(y_dist_lo, z_grads_trf * z_dist_hi),
        );
        let prod_blf = x_grads_blf.mul_add(
            x_dist_lo,
            y_grads_blf.mul_add(y_dist_hi, z_grads_blf * z_dist_lo),
        );
        let prod_brf = x_grads_brf.mul_add(
            x_dist_lo,
            y_grads_brf.mul_add(y_dist_hi, z_grads_brf * z_dist_hi),
        );
        let prod_tlb = x_grads_tlb.mul_add(
            x_dist_hi,
            y_grads_tlb.mul_add(y_dist_lo, z_grads_tlb * z_dist_lo),
        );
        let prod_trb = x_grads_trb.mul_add(
            x_dist_hi,
            y_grads_trb.mul_add(y_dist_lo, z_grads_trb * z_dist_hi),
        );
        let prod_blb = x_grads_blb.mul_add(
            x_dist_hi,
            y_grads_blb.mul_add(y_dist_hi, z_grads_blb * z_dist_lo),
        );
        let prod_brb = x_grads_brb.mul_add(
            x_dist_hi,
            y_grads_brb.mul_add(y_dist_hi, z_grads_brb * z_dist_hi),
        );

        let lerp_tf = z_lerp.mul_add(prod_trf - prod_tlf, prod_tlf);
        let lerp_bf = z_lerp.mul_add(prod_brf - prod_blf, prod_blf);
        let lerp_tb = z_lerp.mul_add(prod_trb - prod_tlb, prod_tlb);
        let lerp_bb = z_lerp.mul_add(prod_brb - prod_blb, prod_blb);

        let lerp_front = y_lerp.mul_add(lerp_bf - lerp_tf, lerp_tf);
        let lerp_back = y_lerp.mul_add(lerp_bb - lerp_tb, lerp_tb);

        x_lerp.mul_add(lerp_back - lerp_front, lerp_front)
    }
}

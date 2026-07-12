use crate::api::batch::interface::BatchNoise;
use crate::simd::arch_simd::ArchSimd;
use crate::value::Value;

impl BatchNoise<2> for Value {
    fn sample_batch(seed: u32, input: [ArchSimd<f32>; 2], freq: [ArchSimd<f32>; 2]) -> ArchSimd<f32> {
        // Constants.
        let neg_two: ArchSimd<f32> = ArchSimd::splat(-2.0);
        let three: ArchSimd<f32> = ArchSimd::splat(3.0);

        let hash_mask: ArchSimd<u32> = ArchSimd::splat(0x007FFFFF);
        let exp_bits: ArchSimd<u32> = ArchSimd::splat(0x40000000);

        // Hash constants.
        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
        let channel_seed = ArchSimd::splat(seed);
        let prime = ArchSimd::splat(0x85ebca6b);

        // Scale: 4
        let x_scaled = input[0] * freq[0];
        let y_scaled = input[1] * freq[1];

        // Gridpoints and distances: 6
        let x_scaled_floored = x_scaled.floor();
        let y_scaled_floored = y_scaled.floor();

        let x_grid_lo = x_scaled_floored.cast_int_trunc();
        let y_grid_lo = y_scaled_floored.cast_int_trunc();

        let x_dist_lo = x_scaled - x_scaled_floored;
        let y_dist_lo = y_scaled - y_scaled_floored;

        // Lerp fade calculation: 6
        let t = x_dist_lo;
        let s = y_dist_lo;
        let x_lerp = t * t * t.mul_add(neg_two, three);
        let y_lerp = s * s * s.mul_add(neg_two, three);

        // Hash: 20
        let x1: ArchSimd<u32> = x_grid_lo.raw_cast() * channel_seed;
        let y1: ArchSimd<u32> = y_grid_lo.raw_cast() * channel_seed;
        let x2 = x1 + channel_seed;
        let y2 = y1 + channel_seed;

        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;

        let hash_tl = x1_shuf * y1_shuf * y1_shuf;
        let hash_tr = x1_shuf * y2_shuf * y2_shuf;
        let hash_bl = x2_shuf * y1_shuf * y1_shuf;
        let hash_br = x2_shuf * y2_shuf * y2_shuf;

        // Values: 12
        let val_tl = ((hash_tl & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_tr = ((hash_tr & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_bl = ((hash_bl & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_br = ((hash_br & hash_mask) | exp_bits).raw_cast::<f32>() - three;

        // Interpolation: 6
        let top_lerp = y_lerp.mul_add(val_tr - val_tl, val_tl);
        let bottom_lerp = y_lerp.mul_add(val_br - val_bl, val_bl);

        x_lerp.mul_add(bottom_lerp - top_lerp, top_lerp)
    }
}

impl BatchNoise<3> for Value {
    fn sample_batch(seed: u32, input: [ArchSimd<f32>; 3], freq: [ArchSimd<f32>; 3]) -> ArchSimd<f32> {
        // Constants.
        let neg_two: ArchSimd<f32> = ArchSimd::splat(-2.0);
        let three: ArchSimd<f32> = ArchSimd::splat(3.0);

        let hash_mask: ArchSimd<u32> = ArchSimd::splat(0x007FFFFF);
        let exp_bits: ArchSimd<u32> = ArchSimd::splat(0x40000000);

        // Hash constants.
        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
        let channel_seed = ArchSimd::splat(seed);
        let prime = ArchSimd::splat(0x85ebca6b);

        // Scale: 3
        let x_scaled = input[0] * freq[0];
        let y_scaled = input[1] * freq[1];
        let z_scaled = input[2] * freq[2];

        // Gridpoints and distances: 9
        let x_scaled_floored = x_scaled.floor();
        let y_scaled_floored = y_scaled.floor();
        let z_scaled_floored = z_scaled.floor();

        let x_grid_lo = x_scaled_floored.cast_int_trunc();
        let y_grid_lo = y_scaled_floored.cast_int_trunc();
        let z_grid_lo = z_scaled_floored.cast_int_trunc();

        let x_dist_lo = x_scaled - x_scaled_floored;
        let y_dist_lo = y_scaled - y_scaled_floored;
        let z_dist_lo = z_scaled - z_scaled_floored;

        // Lerp fade calculation: 9
        let t = x_dist_lo;
        let s = y_dist_lo;
        let u = z_dist_lo;
        let x_lerp = t * t * t.mul_add(neg_two, three);
        let y_lerp = s * s * s.mul_add(neg_two, three);
        let z_lerp = u * u * u.mul_add(neg_two, three);

        // Hash: 42
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

        let hash_tlf = x1_shuf * y1_shuf + z1_shuf * y1_shuf;
        let hash_trf = x1_shuf * y1_shuf + z2_shuf * y1_shuf;
        let hash_blf = x1_shuf * y2_shuf + z1_shuf * y2_shuf;
        let hash_brf = x1_shuf * y2_shuf + z2_shuf * y2_shuf;
        let hash_tlb = x2_shuf * y1_shuf + z1_shuf * y1_shuf;
        let hash_trb = x2_shuf * y1_shuf + z2_shuf * y1_shuf;
        let hash_blb = x2_shuf * y2_shuf + z1_shuf * y2_shuf;
        let hash_brb = x2_shuf * y2_shuf + z2_shuf * y2_shuf;

        // Values: 24
        let val_tlf = ((hash_tlf & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_trf = ((hash_trf & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_blf = ((hash_blf & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_brf = ((hash_brf & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_tlb = ((hash_tlb & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_trb = ((hash_trb & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_blb = ((hash_blb & hash_mask) | exp_bits).raw_cast::<f32>() - three;
        let val_brb = ((hash_brb & hash_mask) | exp_bits).raw_cast::<f32>() - three;

        // Interpolation: 14
        let lerp_tf = z_lerp.mul_add(val_trf - val_tlf, val_tlf);
        let lerp_bf = z_lerp.mul_add(val_brf - val_blf, val_blf);
        let lerp_tb = z_lerp.mul_add(val_trb - val_tlb, val_tlb);
        let lerp_bb = z_lerp.mul_add(val_brb - val_blb, val_blb);

        let lerp_front = y_lerp.mul_add(lerp_bf - lerp_tf, lerp_tf);
        let lerp_back = y_lerp.mul_add(lerp_bb - lerp_tb, lerp_tb);

        x_lerp.mul_add(lerp_back - lerp_front, lerp_front)
    }
}

use crate::api::batch::interface::BatchGenerator;
use crate::simd::arch_simd::ArchSimd;
use crate::noise::generators::Value;

impl BatchGenerator<2> for Value {
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


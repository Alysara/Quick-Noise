// Primitive but fast module for generating random-looking outputs.

use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_traits::*;

pub struct Random {
    core_seed: u64,
    pub channel_seed: u64,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        let init_core_seed: u64 = Self::static_mix_u64(seed);
        let init_channel_seed: u64 = Self::static_mix_u64(init_core_seed);

        Self {
            core_seed: init_core_seed,
            channel_seed: init_channel_seed,
        }
    }

    pub fn set_channel(&mut self, data: u64) {
        self.channel_seed = Self::static_mix_u64(self.core_seed ^ data);
    }

    // === Raw Mixers ===

    pub fn mix_u64(&self, data: u64) -> u64 {
        Self::mix_u64_impl(data ^ self.channel_seed)
    }

    pub fn mix_u64_pair(&self, data1: u64, data2: u64) -> u64 {
        Self::mix_u64_pair_impl(data1 ^ self.channel_seed, data2)
    }

    pub fn mix_i32_simd_pair(&self, data1: ArchSimd<i32>, data2: ArchSimd<i32>) -> ArchSimd<u32> {
        let seed_vec = ArchSimd::<u32>::splat(self.channel_seed as u32);
        Self::mix_u32_pair_simd_impl(data1.raw_cast() ^ seed_vec, data2.raw_cast())
    }

    pub fn mix_i32_simd_pair_fast(
        &self,
        data1: ArchSimd<i32>,
        data2: ArchSimd<i32>,
    ) -> ArchSimd<u32> {
        let seed_vec = ArchSimd::<u32>::splat(self.channel_seed as u32);
        Self::mix_u32_pair_simd_fast_impl(data1.raw_cast(), data2.raw_cast(), seed_vec)
    }

    pub fn mix_i32_simd_triple(
        &self,
        data1: ArchSimd<i32>,
        data2: ArchSimd<i32>,
        data3: ArchSimd<i32>,
    ) -> ArchSimd<u32> {
        let seed_vec = ArchSimd::<u32>::splat(self.channel_seed as u32);

        Self::mix_u32_triple_simd_impl(
            data1.raw_cast() ^ seed_vec,
            data2.raw_cast(),
            data3.raw_cast(),
        )
    }

    pub fn mix_u32(&self, data: u32) -> u32 {
        Self::mix_bits_32_impl(data ^ self.core_seed as u32)
    }

    // === Mix Variants ===

    pub fn mix_i32_pair(&self, data1: i32, data2: i32) -> u64 {
        self.mix_u64(Self::combine_i32_pair(data1, data2))
    }

    pub fn mix_i32_triple(&self, data1: i32, data2: i32, data3: i32) -> u64 {
        let concat_data: u64 = Self::combine_i32_pair(data1, data2);
        self.mix_u64_pair(concat_data, (data3 as u64) << 32)
    }

    // === Static Mixers ===

    pub fn static_mix_u64(data: u64) -> u64 {
        Self::mix_u64_impl(data ^ 0x9e3779b97f4a7c15)
    }

    pub fn static_mix_u64_pair(data1: u64, data2: u64) -> u64 {
        Self::mix_u64_pair_impl(data1 ^ 0x9e3779b97f4a7c15, data2)
    }

    pub fn static_mix_u32(data: u32) -> u32 {
        Self::mix_bits_32_impl(data ^ 0x7f4a7c15)
    }

    // === Private Helpers ===

    fn combine_i32_pair(data1: i32, data2: i32) -> u64 {
        (data1 as u64) | ((data2 as u64) << 32)
    }

    // === Bit mixing implementations | From MurmurHash ===

    pub fn mix_u64_impl(mut data: u64) -> u64 {
        data ^= data >> 33;
        data = data.wrapping_mul(0xff51afd7ed558ccd);
        data ^= data >> 33;
        data = data.wrapping_mul(0xc4ceb9fe1a85ec53);
        data ^= data >> 33;
        data
    }

    fn mix_u64_pair_impl(mut data1: u64, data2: u64) -> u64 {
        data1 ^= data1 >> 33;
        data1 = data1.wrapping_mul(0xff51afd7ed558ccd ^ data2);
        data1 ^= data1 >> 33;
        data1 = data1.wrapping_mul(0xc4ceb9fe1a85ec53 ^ data2);
        data1 ^= data1 >> 33;
        data1
    }

    pub fn mix_u64_triple(mut data1: u64, data2: u64, data3: u64) -> u64 {
        data1 ^= data1 >> 33;
        data1 = data1.wrapping_mul(0xff51afd7ed558ccd ^ data2);
        data1 ^= data1 >> 33;
        data1 = data1.wrapping_mul(0xc4ceb9fe1a85ec53 ^ data3);
        data1 ^= data1 >> 33;
        data1 = data1.wrapping_mul(0xff51afd7ed558ccd ^ data2);
        data1 ^= data1 >> 33;
        data1
    }

    fn mix_bits_32_impl(mut data: u32) -> u32 {
        data ^= data >> 16;
        data = data.wrapping_mul(0x85ebca6b);
        data ^= data >> 13;
        data = data.wrapping_mul(0xc2b2ae35);
        data ^= data >> 16;
        data
    }

    fn mix_u32_pair_simd_impl(mut data1: ArchSimd<u32>, data2: ArchSimd<u32>) -> ArchSimd<u32> {
        data1 ^= data1 >> 16;
        data1 *= ArchSimd::splat(0x85ebca6b) ^ data2;
        data1 ^= data1 >> 13;
        data1 *= ArchSimd::splat(0xc2b2ae35);
        data1 ^= data1 >> 16;
        data1
    }

    fn mix_u32_pair_simd_fast_impl(
        mut data1: ArchSimd<u32>,
        data2: ArchSimd<u32>,
        seed: ArchSimd<u32>,
    ) -> ArchSimd<u32> {
        data1 ^= data1 >> 16;
        data1 *= ArchSimd::splat(0x85ebca6b) ^ data2;
        data1 ^= data1 >> 16;
        data1
    }

    // pub fn mix_u32_four_group(
    //     &self, mut x1: ArchSimd<u32>, mut y1: ArchSimd<u32>) -> (ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>) {
    //     // let core_seed = ArchSimd::splat(self.core_seed as u32);
    //     let channel_seed = ArchSimd::splat(self.channel_seed as u32);
    //     let prime = ArchSimd::splat(0x85ebca6b);

    //     x1 *= channel_seed;
    //     let mut x2 = x1 + channel_seed;
    //     y1 *= channel_seed;
    //     let mut y2 = y1 + channel_seed;

    //     x1 ^= (x1 ^ prime) >> 16;
    //     x2 ^= (x2 ^ prime) >> 16;
    //     y1 ^= (y1 ^ prime) >> 16;
    //     y2 ^= (y2 ^ prime) >> 16;

    //     let tl = x1 * y1;
    //     let tr = x1 * y2;
    //     let bl = x2 * y1;
    //     let br = x2 * y2;

    //     (tl, tr, bl, br)
    // }

    // pub fn mix_u32_four_group(
    //     &self, mut x1: ArchSimd<u32>, mut y1: ArchSimd<u32>)
    //         -> (ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>)
    //     {

    //     // let core_seed = Simd::splat(self.core_seed as i32);
    //     let channel_seed = Simd::splat(self.channel_seed as u32);
    //     let prime = Simd::splat(0x85ebca6b_u32 as u32);

    //     x1 *= channel_seed;
    //     let mut x2 = x1 + channel_seed;
    //     y1 *= channel_seed;
    //     let mut y2 = y1 + channel_seed;

    //     x1 ^= (x1 ^ prime) >> 16;
    //     x2 ^= (x2 ^ prime) >> 16;
    //     y1 ^= (y1 ^ prime) >> 16;
    //     y2 ^= (y2 ^ prime) >> 16;

    //     (x1 * y1, x1 * y2, x2 * y1, x2 * y2)
    // }

    pub fn mix_u32_four_group(
        &self,
        mut x1: ArchSimd<u32>,
        mut y1: ArchSimd<u32>,
    ) -> (ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>) {
        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3,
            0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0, 2, 1, 3, 0,
            2, 1, 3, 0, 2, 1,
        ];

        let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
        let channel_seed = ArchSimd::splat(self.channel_seed as u32);
        let prime = ArchSimd::splat(0x85ebca6b_u32 as u32);

        x1 *= channel_seed;
        y1 *= channel_seed;

        let x2 = x1 + channel_seed;
        let y2 = y1 + channel_seed;

        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;

        let tl = x1_shuf * y1_shuf;
        let tr = x1_shuf * y2_shuf;
        let bl = x2_shuf * y1_shuf;
        let br = x2_shuf * y2_shuf;

        (tl, tr, bl, br)
    }

    // pub fn mix_u32_two_group(&self, mut x: ArchSimd<u32>, mut y: ArchSimd<u32>) -> (ArchSimd<u32>, ArchSimd<u32>)

    // pub fn mix_u32_four_group(
    //     &self, mut x1: ArchSimd<u32>, mut y1: ArchSimd<u32>) -> (ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>) {

    //     let channel_seed = ArchSimd::splat(self.channel_seed as u32);
    //     let prime = ArchSimd::splat(0x85ebca6b_u32 as u32);

    //     x1 *= channel_seed;
    //     y1 *= channel_seed;
    //     let x2 = x1 + channel_seed;
    //     let y2 = y1 + channel_seed;

    //     const BYTE_SHUFFLE: [u8; 4] = [3, 0, 2, 1];
    //     let x1_shuf = x1.permute_8_pattern_32(BYTE_SHUFFLE) ^ prime;
    //     let y1_shuf = y1.permute_8_pattern_32(BYTE_SHUFFLE) ^ prime;
    //     let x2_shuf = x2.permute_8_pattern_32(BYTE_SHUFFLE) ^ prime;
    //     let y2_shuf = y2.permute_8_pattern_32(BYTE_SHUFFLE) ^ prime;

    //     let tl = x1_shuf * y1_shuf;
    //     let tr = x1_shuf * y2_shuf;
    //     let bl = x2_shuf * y1_shuf;
    //     let br = x2_shuf * y2_shuf;

    //     (tl, tr, bl, br)
    // }

    // pub fn mix_u32_four_group(
    //     &self,
    //     x1: ArchSimd<u32>, x2: ArchSimd<u32>,
    //     y1: ArchSimd<u32>, y2: ArchSimd<u32>,
    // ) -> (ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>) {
    //     let cs = ArchSimd::splat(self.core_seed as u32);
    //     let ch = ArchSimd::splat(self.channel_seed as u32);
    //     let prime = ArchSimd::splat(0x9e3779b9u32);
    //     let x1s = x1 * cs;
    //     let x2s = x2 * cs;
    //     let y1s = y1 * ch;
    //     let y2s = y2 * ch;
    //     (
    //         (x1s ^ y1s) * prime,
    //         (x1s ^ y2s) * prime,
    //         (x2s ^ y1s) * prime,
    //         (x2s ^ y2s) * prime,
    //     )
    // }

    //     pub fn mix_f32_four_group(
    //         &self, x1: ArchSimd<f32>, x2: ArchSimd<f32>, y1: ArchSimd<f32>, y2: ArchSimd<f32>
    //     ) -> (ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>) {
    //         let core_seed: ArchSimd<f32> = unsafe { std::mem::transmute(ArchSimd::splat(self.core_seed)) };
    //         let channel_seed: ArchSimd<f32> = unsafe { std::mem::transmute(ArchSimd::splat(self.channel_seed)) };

    //         let x1s = x1 * core_seed;
    //         let x2s = x2 * core_seed;
    //         let y1s = y1 * channel_seed;
    //         let y2s = y2 * channel_seed;

    //         let tl: ArchSimd<u32> = unsafe { std::mem::transmute(x1s + y1s) };
    //         let tr: ArchSimd<u32> = unsafe { std::mem::transmute(x1s + y2s) };
    //         let bl: ArchSimd<u32> = unsafe { std::mem::transmute(x2s + y1s) };
    //         let br: ArchSimd<u32> = unsafe { std::mem::transmute(x2s + y2s) };

    //         (tl, tr, bl, br)
    //     }

    fn mix_u32_triple_simd_impl(
        mut data1: ArchSimd<u32>,
        data2: ArchSimd<u32>,
        data3: ArchSimd<u32>,
    ) -> ArchSimd<u32> {
        data1 ^= data1 >> 16;
        data1 *= ArchSimd::splat(0x85ebca6b) ^ data2;
        data1 ^= data1 >> 13;
        data1 *= ArchSimd::splat(0xc2b2ae35) ^ data3;
        data1 ^= data1 >> 16;
        data1
    }

    #[inline(always)]
    pub fn cartesian_quad_hi_bit_shuf(
        &self,
        x_lo: ArchSimd<u32>,
        y_lo: ArchSimd<u32>,
    ) -> (ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>, ArchSimd<u32>) {
        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        let seed = ArchSimd::splat(self.channel_seed as u32);
        let prime = ArchSimd::splat(0x85ebca6b_u32);
        let shuffle_indices = ArchSimd::from_slice(&BYTE_SHUFFLE[..]);

        let x1 = x_lo * seed;
        let y1 = y_lo * seed;
        let x2 = x1 + seed;
        let y2 = y1 + seed;

        let x1_shuf = x1.permute_8(shuffle_indices) ^ prime;
        let y1_shuf = y1.permute_8(shuffle_indices) ^ prime;
        let x2_shuf = x2.permute_8(shuffle_indices) ^ prime;
        let y2_shuf = y2.permute_8(shuffle_indices) ^ prime;

        let mix_tl = x1_shuf * y1_shuf;
        let mix_tr = x1_shuf * y2_shuf;
        let mix_bl = x2_shuf * y1_shuf;
        let mix_br = x2_shuf * y2_shuf;

        (mix_tl, mix_tr, mix_bl, mix_br)
    }
}

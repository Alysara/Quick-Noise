use crate::noise::perlin::Perlin;
use crate::noise::perlin::constants::*;
use crate::simd::arch_simd::{ArchSimd, ArchMask};
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;

impl Perlin {
    #[inline(always)]
    pub(super) fn set_uniform_grid_gradients_2d (
        &mut self,
        left: &mut PerlinVecPair,
        right: &mut PerlinVecPair,
        x_start: i32,
        y_start: i32,
        y_grid_indices: u32,
        y_num_loops: u32,
        y_distances: &PerlinVec,
    ) {
        let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(self.random_gen.channel_seed as u32);
        let x_vec = ArchSimd::splat((x_start as u32).wrapping_mul(self.random_gen.channel_seed as u32));
        let mut y_vec = ArchSimd::splat((y_start as u32).wrapping_mul(self.random_gen.channel_seed as u32)) + iota_vec;
        let y_vec_stride = ArchSimd::splat((ArchSimd::<f32>::LANES as u32).wrapping_mul(self.random_gen.channel_seed as u32));

        const BYTE_SHUFFLE: [u8; 64] = [
            3,0,2,1, 7,4,6,5, 11,8,10,9, 15,12,14,13,
            3,0,2,1, 7,4,6,5, 11,8,10,9, 15,12,14,13,
            3,0,2,1, 7,4,6,5, 11,8,10,9, 15,12,14,13,
            3,0,2,1, 7,4,6,5, 11,8,10,9, 15,12,14,13,
        ];

        let shuffle_indices = ArchSimd::<u8>::load(&BYTE_SHUFFLE[..]);

        let prime = ArchSimd::splat(0x85ebca6b_u32);
        let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;

        // Temporary buffer to store indices for gradient values.
        let mut grad_array = SimdArray::<u32, 64>::new_uninit();

        // Main vectorized bit mixing loop.
        let end_index = y_num_loops as usize + 1;
        for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
            let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (x_shuf * y_shuf) >> 29;
            grad_array.store_simd(i, indices);
            y_vec += y_vec_stride;
        }

        let mut arrays = [
            &mut left.x, &mut left.y,
            &mut right.x, &mut right.y,
        ];

        // Loop through the y chunks.
        let mut y_cur_index: u32 = 0;
        let mut indices = y_grid_indices;
        for y_it in 0..y_num_loops {
            let y_next_index: u32 = indices.trailing_zeros();

            // Find range of gradients to set.
            let set_amount: u32 = y_next_index - y_cur_index;

            unsafe {
                let l = grad_array.get_unchecked(y_it as usize) as usize;
                let r = grad_array.get_unchecked    (y_it as usize + 1) as usize;

                debug_assert!(l < 32);
                debug_assert!(r < 32);
                let values = [
                    GRADIENTS_2D.get_unchecked(l).x, GRADIENTS_2D.get_unchecked(l).y,
                    GRADIENTS_2D.get_unchecked(r).x, GRADIENTS_2D.get_unchecked(r).y,
                ];

                PerlinVec::multiset_many::<4>(&mut arrays, &values, y_cur_index as usize, set_amount as isize);
            }

            indices ^= 1 << y_next_index;
            y_cur_index = y_next_index;
        }

        // Compute y dot products (Better to do here since these dot products get reused and operate per element).
        left.y *= *y_distances;
        right.y = right.y.mul_sub(*y_distances, right.y); // equivalent to -> right.y *= y_distances - 1.0
    }

    // #[inline(never)]
    pub(super) fn set_uniform_grid_gradients_3d (
        &mut self,
        lf: &mut PerlinVecTriple,
        rf: &mut PerlinVecTriple,
        lb: &mut PerlinVecTriple,
        rb: &mut PerlinVecTriple,
        x_start: i32,
        y_start: i32,
        z_start: i32,
        z_grid_indices: &SimdArray<u32, ROW_SIZE>,
        z_scale: f32,
        z_num_loops: u32,
        z_distances: &PerlinVec,
    ) {
        let iota_vec = ArchSimd::iota(0);

        let x_front_vec = ArchSimd::splat(x_start);
        let x_back_vec = ArchSimd::splat(x_start + 1);
        let y_vec = ArchSimd::splat(y_start);
        let lane_increment = ArchSimd::splat(ArchSimd::<f32>::LANES as i32);

        let mut front_grad_array = SimdArray::<u32, 64>::new_uninit();
        let mut z_vec = ArchSimd::splat(z_start) + iota_vec;
        let grad: ArchSimd<u32> = self.random_gen.mix_i32_simd_triple(x_front_vec, y_vec, z_vec) & ArchSimd::splat(15);
        front_grad_array.store_simd(0, grad);
        for i in (ArchSimd::<f32>::LANES..z_num_loops as usize + 1).step_by(ArchSimd::<f32>::LANES) {
            z_vec += lane_increment;
            let grad: ArchSimd<u32> = self.random_gen.mix_i32_simd_triple(x_front_vec, y_vec, z_vec) & ArchSimd::splat(15);
            front_grad_array.store_simd(i, grad);
        }

        let mut front_arrays = [
            &mut lf.x, &mut lf.y, &mut lf.z,
            &mut rf.x, &mut rf.y, &mut rf.z,
        ];

        let mut z_cur_index: u32 = 0;
        for z_it in 0..z_num_loops {
            let z_next_index: u32 = z_grid_indices[z_it as usize];
            let set_amount: u32 = z_next_index - z_cur_index;
            unsafe {
                let lf_grad = front_grad_array.get_unchecked(z_it as usize) as usize;
                let rf_grad = front_grad_array.get_unchecked(z_it as usize + 1) as usize;
                debug_assert!(lf_grad < 32);
                debug_assert!(rf_grad < 32);
                let values = [
                    GRADIENTS_3D.get_unchecked(lf_grad).x, GRADIENTS_3D.get_unchecked(lf_grad).y, GRADIENTS_3D.get_unchecked(lf_grad).z,
                    GRADIENTS_3D.get_unchecked(rf_grad).x, GRADIENTS_3D.get_unchecked(rf_grad).y, GRADIENTS_3D.get_unchecked(rf_grad).z,
                ];
                PerlinVec::multiset_many::<6>(&mut front_arrays, &values, z_cur_index as usize, set_amount as isize);
            }
            z_cur_index = z_next_index;
        }

        let mut back_grad_array = SimdArray::<u32, 64>::new_uninit();
        let mut z_vec = ArchSimd::splat(z_start) + iota_vec;
        let grad: ArchSimd<u32> = self.random_gen.mix_i32_simd_triple(x_back_vec, y_vec, z_vec) & ArchSimd::splat(15);
        back_grad_array.store_simd(0, grad);
        for i in (ArchSimd::<f32>::LANES..z_num_loops as usize + 1).step_by(ArchSimd::<f32>::LANES) {
            z_vec += lane_increment;
            let grad: ArchSimd<u32> = self.random_gen.mix_i32_simd_triple(x_back_vec, y_vec, z_vec) & ArchSimd::splat(15);
            back_grad_array.store_simd(i, grad);
        }

        let mut back_arrays = [
            &mut lb.x, &mut lb.y, &mut lb.z,
            &mut rb.x, &mut rb.y, &mut rb.z,
        ];

        let mut z_cur_index: u32 = 0;
        for z_it in 0..z_num_loops {
            let z_next_index: u32 = z_grid_indices[z_it as usize];
            let set_amount: u32 = z_next_index - z_cur_index;
            unsafe {
                let lb_grad = back_grad_array.get_unchecked(z_it as usize) as usize;
                let rb_grad = back_grad_array.get_unchecked(z_it as usize + 1) as usize;
                debug_assert!(lb_grad < 32);
                debug_assert!(rb_grad < 32);
                let values = [
                    GRADIENTS_3D.get_unchecked(lb_grad).x, GRADIENTS_3D.get_unchecked(lb_grad).y, GRADIENTS_3D.get_unchecked(lb_grad).z,
                    GRADIENTS_3D.get_unchecked(rb_grad).x, GRADIENTS_3D.get_unchecked(rb_grad).y, GRADIENTS_3D.get_unchecked(rb_grad).z,
                ];
                PerlinVec::multiset_many::<6>(&mut back_arrays, &values, z_cur_index as usize, set_amount as isize);
            }
            if z_next_index == 32 { break; }
            z_cur_index = z_next_index;
        }

        lf.z *= *z_distances;
        rf.z = rf.z.mul_sub(*z_distances, rf.z);
        lb.z *= *z_distances;
        rb.z = rb.z.mul_sub(*z_distances, rb.z);
    }
}

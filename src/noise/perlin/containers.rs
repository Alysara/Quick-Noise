use crate::math::vec::{BasicVec, Vec2, Vec3};
use crate::simd::simd_array::SimdArray;

#[derive(Copy, Clone)]
pub struct Octave2D {
    pub frequency: Vec2<f32>,
    pub weight: f32,
}

impl Octave2D {
    pub const fn new(frequency: Vec2<f32>, weight: f32) -> Self {
        Self { frequency, weight }
    }

    pub const fn splat(frequency: f32, weight: f32) -> Self {
        Self {
            frequency: Vec2::<f32>::new(frequency, frequency),
            weight,
        }
    }
}

impl From<(f32, f32)> for Octave2D {
    fn from((frequency, weight): (f32, f32)) -> Self {
        Octave2D::new((frequency, frequency).into(), weight)
    }
}

impl From<((f32, f32), f32)> for Octave2D {
    fn from(((x_frequency, y_frequency), weight): ((f32, f32), f32)) -> Self {
        Octave2D::new((x_frequency, y_frequency).into(), weight)
    }
}

impl From<&Octave2D> for Octave2D {
    fn from(octave: &Octave2D) -> Self {
        octave.clone()
    }
}

#[derive(Copy, Clone)]
pub struct Octave3D {
    pub frequency: Vec3<f32>,
    pub weight: f32,
}

impl Octave3D {
    pub const fn new(frequency: Vec3<f32>, weight: f32) -> Self {
        Self { frequency, weight }
    }

    pub const fn splat(frequency: f32, weight: f32) -> Self {
        Self {
            frequency: Vec3::<f32>::new(frequency, frequency, frequency),
            weight,
        }
    }
}

impl From<(f32, f32)> for Octave3D {
    fn from((frequency, weight): (f32, f32)) -> Self {
        Octave3D::new((frequency, frequency, frequency).into(), weight)
    }
}

impl From<((f32, f32, f32), f32)> for Octave3D {
    fn from(((x_frequency, y_frequency, z_frequency), weight): ((f32, f32, f32), f32)) -> Self {
        Octave3D::new((x_frequency, y_frequency, z_frequency).into(), weight)
    }
}

impl From<&Octave3D> for Octave3D {
    fn from(octave: &Octave3D) -> Self {
        octave.clone()
    }
}

pub struct PerlinContainer2D<const N: usize> {
    vecs: [Vec2<SimdArray<f32, N>>; 4],
    tl: usize, // Top left.
    tr: usize, // Top right.
    bl: usize, // Bottom left.
    br: usize, // Bottom right.
}

impl<const N: usize> PerlinContainer2D<N> {
    pub unsafe fn new_uninit() -> Self {
        unsafe {
            Self {
                vecs: [
                    Vec2::splat(SimdArray::new_uninit()),
                    Vec2::splat(SimdArray::new_uninit()),
                    Vec2::splat(SimdArray::new_uninit()),
                    Vec2::splat(SimdArray::new_uninit()),
                ],
                tl: 0,
                tr: 1,
                bl: 2,
                br: 3,
            }
        }
    }

    pub fn tl(&self) -> &Vec2<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.tl) }
    }
    pub fn tr(&self) -> &Vec2<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.tr) }
    }
    pub fn bl(&self) -> &Vec2<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.bl) }
    }
    pub fn br(&self) -> &Vec2<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.br) }
    }

    pub fn tl_tr_mut(&mut self) -> (&mut Vec2<SimdArray<f32, N>>, &mut Vec2<SimdArray<f32, N>>) {
        debug_assert!(self.tl < self.tr);
        debug_assert!(self.tr < self.vecs.len());
        unsafe {
            let ptr = self.vecs.as_mut_ptr();
            (&mut *ptr.add(self.tl), &mut *ptr.add(self.tr))
        }
    }

    pub fn bl_br_mut(&mut self) -> (&mut Vec2<SimdArray<f32, N>>, &mut Vec2<SimdArray<f32, N>>) {
        debug_assert!(self.bl < self.br);
        debug_assert!(self.br < self.vecs.len());
        unsafe {
            let ptr = self.vecs.as_mut_ptr();
            (&mut *ptr.add(self.bl), &mut *ptr.add(self.br))
        }
    }

    pub fn swap_top_bottom(&mut self) {
        std::mem::swap(&mut self.tl, &mut self.bl);
        std::mem::swap(&mut self.tr, &mut self.br);
    }
}
pub struct PerlinContainer3D<const N: usize> {
    vecs: [Vec3<SimdArray<f32, N>>; 8],
    tlf: usize, // Top left front.
    trf: usize, // Top right front.
    tlb: usize, // Top left back.
    trb: usize, // Top right back.
    blf: usize, // Bottom left front.
    brf: usize, // Bottom right front.
    blb: usize, // Bottom left back.
    brb: usize, // Bottom right back.
}

impl<const N: usize> PerlinContainer3D<N> {
    pub unsafe fn new_uninit() -> Self {
        unsafe {
            PerlinContainer3D {
                vecs: [
                    Vec3::splat(SimdArray::new_uninit()),
                    Vec3::splat(SimdArray::new_uninit()),
                    Vec3::splat(SimdArray::new_uninit()),
                    Vec3::splat(SimdArray::new_uninit()),
                    Vec3::splat(SimdArray::new_uninit()),
                    Vec3::splat(SimdArray::new_uninit()),
                    Vec3::splat(SimdArray::new_uninit()),
                    Vec3::splat(SimdArray::new_uninit()),
                ],
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

    pub fn tlf(&self) -> &Vec3<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.tlf) }
    }
    pub fn trf(&self) -> &Vec3<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.trf) }
    }
    pub fn blf(&self) -> &Vec3<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.blf) }
    }
    pub fn brf(&self) -> &Vec3<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.brf) }
    }
    pub fn tlb(&self) -> &Vec3<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.tlb) }
    }
    pub fn trb(&self) -> &Vec3<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.trb) }
    }
    pub fn blb(&self) -> &Vec3<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.blb) }
    }
    pub fn brb(&self) -> &Vec3<SimdArray<f32, N>> {
        unsafe { &self.vecs.get_unchecked(self.brb) }
    }

    pub fn tlf_trf_tlb_trb_mut(
        &mut self,
    ) -> (
        &mut Vec3<SimdArray<f32, N>>,
        &mut Vec3<SimdArray<f32, N>>,
        &mut Vec3<SimdArray<f32, N>>,
        &mut Vec3<SimdArray<f32, N>>,
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
        &mut Vec3<SimdArray<f32, N>>,
        &mut Vec3<SimdArray<f32, N>>,
        &mut Vec3<SimdArray<f32, N>>,
        &mut Vec3<SimdArray<f32, N>>,
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

// pub struct PerlinContainer3D {
//     tlf: SimdArray<f32, N>, // Top left front.
//     trf: SimdArray<f32, N>, // Top right front.
//     blf: SimdArray<f32, N>, // Bottom left front.
//     brf: SimdArray<f32, N>, // Bottom right front.
//     tlb: SimdArray<f32, N>, // Top left back.
//     trb: SimdArray<f32, N>, // Top right back.
//     blb: SimdArray<f32, N>, // Bottom left back.
//     brb: SimdArray<f32, N>, // Bottom right back.
// }

// impl PerlinContainer3D {
//     pub fn new_uninit() -> Self {
//         PerlinContainer3D {
//             tlf: SimdArray<f32, N>::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             trf: SimdArray<f32, N>::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             blf: SimdArray<f32, N>::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             brf: SimdArray<f32, N>::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             tlb: SimdArray<f32, N>::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             trb: SimdArray<f32, N>::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             blb: SimdArray<f32, N>::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             brb: SimdArray<f32, N>::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//         }
//     }
// }

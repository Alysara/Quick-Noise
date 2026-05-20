use crate::math::vec::{Vec2, Vec3};
use crate::noise::perlin::constants::{PerlinVec, PerlinVecPair, PerlinVecTriple};

#[derive(Copy, Clone)]
pub struct Octave2D {
    pub scale: Vec2<f32>,
    pub weight: f32,
}

impl Octave2D {
    pub fn new(scale: Vec2<f32>, weight: f32) -> Self {
        Self { scale, weight }
    }

    pub fn splat(scale: f32, weight: f32) -> Self {
        Self { scale: Vec2::<f32>::new(scale, scale), weight }
    }
}

impl From<(f32, f32)> for Octave2D {
    fn from((scale, weight): (f32, f32)) -> Self {
        Octave2D::new((scale, scale).into(), weight)
    }
}

impl From<((f32, f32), f32)> for Octave2D {
    fn from(((x_scale, y_scale), weight): ((f32, f32), f32)) -> Self {
        Octave2D::new((x_scale, y_scale).into(), weight)
    }
}

impl From<&Octave2D> for Octave2D {
    fn from(octave: &Octave2D) -> Self {
        octave.clone()
    }
}

#[derive(Copy, Clone)]
pub struct Octave3D {
    pub scale: Vec3<f32>,
    pub weight: f32,
}

impl Octave3D {
    pub fn new(scale: Vec3<f32>, weight: f32) -> Self {
        Self { scale, weight }
    }

    pub fn splat(scale: f32, weight: f32) -> Self {
        Self { scale: Vec3::<f32>::new(scale, scale, scale), weight }
    }
}

impl From<(f32, f32)> for Octave3D {
    fn from((scale, weight): (f32, f32)) -> Self {
        Octave3D::new((scale, scale, scale).into(), weight)
    }
}

impl From<((f32, f32, f32), f32)> for Octave3D {
    fn from(((x_scale, y_scale, z_scale), weight): ((f32, f32, f32), f32)) -> Self {
        Octave3D::new((x_scale, y_scale, z_scale).into(), weight)
    }
}

impl From<&Octave3D> for Octave3D {
    fn from(octave: &Octave3D) -> Self {
        octave.clone()
    }
}

pub struct PerlinContainer2D {
    vecs: [PerlinVecPair; 4],
    tl: usize, // Top left.
    tr: usize, // Top right.
    bl: usize, // Bottom left.
    br: usize, // Bottom right.
}

impl PerlinContainer2D {
    pub unsafe fn new_uninit() -> Self {
        unsafe {
            PerlinContainer2D {
                vecs: [
                    PerlinVecPair::new(PerlinVec::new_uninit(), PerlinVec::new_uninit()),
                    PerlinVecPair::new(PerlinVec::new_uninit(), PerlinVec::new_uninit()),
                    PerlinVecPair::new(PerlinVec::new_uninit(), PerlinVec::new_uninit()),
                    PerlinVecPair::new(PerlinVec::new_uninit(), PerlinVec::new_uninit()),
                ],
                tl: 0,
                tr: 1,
                bl: 2,
                br: 3,
            }
        }
    }

    pub fn tl(&self) -> &PerlinVecPair { unsafe { &self.vecs.get_unchecked(self.tl) } }
    pub fn tr(&self) -> &PerlinVecPair { unsafe { &self.vecs.get_unchecked(self.tr) } }
    pub fn bl(&self) -> &PerlinVecPair { unsafe { &self.vecs.get_unchecked(self.bl) } }
    pub fn br(&self) -> &PerlinVecPair { unsafe { &self.vecs.get_unchecked(self.br) } }

    pub fn tl_tr_mut(&mut self) -> (&mut PerlinVecPair, &mut PerlinVecPair) {
        debug_assert!(self.tl < self.tr);
        debug_assert!(self.tr < self.vecs.len());
        unsafe {
            let ptr = self.vecs.as_mut_ptr();
            (&mut *ptr.add(self.tl), &mut *ptr.add(self.tr))
        }
    }

    pub fn bl_br_mut(&mut self) -> (&mut PerlinVecPair, &mut PerlinVecPair) {
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
pub struct PerlinContainer3D {
    vecs: [PerlinVecTriple; 8],
    tlf: usize, // Top left front.
    trf: usize, // Top right front.
    tlb: usize, // Top left back.
    trb: usize, // Top right back.
    blf: usize, // Bottom left front.
    brf: usize, // Bottom right front.
    blb: usize, // Bottom left back.
    brb: usize, // Bottom right back.
}

impl PerlinContainer3D {
    pub unsafe fn new_uninit() -> Self {
        unsafe {
            PerlinContainer3D {
                vecs: [
                    PerlinVecTriple::splat(PerlinVec::new_uninit()),
                    PerlinVecTriple::splat(PerlinVec::new_uninit()),
                    PerlinVecTriple::splat(PerlinVec::new_uninit()),
                    PerlinVecTriple::splat(PerlinVec::new_uninit()),
                    PerlinVecTriple::splat(PerlinVec::new_uninit()),
                    PerlinVecTriple::splat(PerlinVec::new_uninit()),
                    PerlinVecTriple::splat(PerlinVec::new_uninit()),
                    PerlinVecTriple::splat(PerlinVec::new_uninit()),
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

    pub fn tlf(&self) -> &PerlinVecTriple { unsafe { &self.vecs.get_unchecked(self.tlf) } }
    pub fn trf(&self) -> &PerlinVecTriple { unsafe { &self.vecs.get_unchecked(self.trf) } }
    pub fn blf(&self) -> &PerlinVecTriple { unsafe { &self.vecs.get_unchecked(self.blf) } }
    pub fn brf(&self) -> &PerlinVecTriple { unsafe { &self.vecs.get_unchecked(self.brf) } }
    pub fn tlb(&self) -> &PerlinVecTriple { unsafe { &self.vecs.get_unchecked(self.tlb) } }
    pub fn trb(&self) -> &PerlinVecTriple { unsafe { &self.vecs.get_unchecked(self.trb) } }
    pub fn blb(&self) -> &PerlinVecTriple { unsafe { &self.vecs.get_unchecked(self.blb) } }
    pub fn brb(&self) -> &PerlinVecTriple { unsafe { &self.vecs.get_unchecked(self.brb) } }

    pub fn tlf_trf_tlb_trb_mut(&mut self) -> (&mut PerlinVecTriple, &mut PerlinVecTriple, &mut PerlinVecTriple, &mut PerlinVecTriple) {
        debug_assert!(self.tlf < self.trf);
        debug_assert!(self.trf < self.tlb);
        debug_assert!(self.tlb < self.trb);
        debug_assert!(self.trb < self.vecs.len());
        unsafe {
            let ptr = self.vecs.as_mut_ptr();
            (
                &mut *ptr.add(self.tlf), &mut *ptr.add(self.trf),
                &mut *ptr.add(self.tlb), &mut *ptr.add(self.trb),
            )
        }
    }

    pub fn blf_brf_blb_brb_mut(&mut self) -> (&mut PerlinVecTriple, &mut PerlinVecTriple, &mut PerlinVecTriple, &mut PerlinVecTriple) {
        debug_assert!(self.blf < self.brf);
        debug_assert!(self.brf < self.blb);
        debug_assert!(self.blb < self.brb);
        debug_assert!(self.brb < self.vecs.len());
        unsafe {
            let ptr = self.vecs.as_mut_ptr();
            (
                &mut *ptr.add(self.blf), &mut *ptr.add(self.brf),
                &mut *ptr.add(self.blb), &mut *ptr.add(self.brb),
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
//     tlf: PerlinVecTriple, // Top left front.
//     trf: PerlinVecTriple, // Top right front.
//     blf: PerlinVecTriple, // Bottom left front.
//     brf: PerlinVecTriple, // Bottom right front.
//     tlb: PerlinVecTriple, // Top left back.
//     trb: PerlinVecTriple, // Top right back.
//     blb: PerlinVecTriple, // Bottom left back.
//     brb: PerlinVecTriple, // Bottom right back.
// }

// impl PerlinContainer3D {
//     pub fn new_uninit() -> Self {
//         PerlinContainer3D {
//             tlf: PerlinVecTriple::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             trf: PerlinVecTriple::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             blf: PerlinVecTriple::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             brf: PerlinVecTriple::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             tlb: PerlinVecTriple::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             trb: PerlinVecTriple::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             blb: PerlinVecTriple::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//             brb: PerlinVecTriple::new(PerlinVec::new_uninit(), PerlinVec::new_uninit(), PerlinVec::new_uninit()),
//         }
//     }
// }

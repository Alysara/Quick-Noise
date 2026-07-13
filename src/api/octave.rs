#[derive(Copy, Clone)]
pub struct Octave<const D: usize> {
    pub weight: f32,
    pub frequency: [f32; D],
}

impl<const D: usize> Octave<D> {
    pub fn new(frequency: [f32; D], weight: f32) -> Self {
        Self { frequency, weight }
    }

    pub fn splat(frequency: f32, weight: f32) -> Self {
        let frequency = [frequency; D];
        Self { frequency, weight }
    }
}

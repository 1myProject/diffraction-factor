const SPEED_OF_LIGHT: f32 = 29.9792458;

pub fn converter_freq(freq: f32) -> f32 {
    //ГГц <-> см
    SPEED_OF_LIGHT / freq
}

// elements of difraction-factor
#[derive(Debug)]
pub struct DifrPoint {
    u: f64,
    abs: f64,
    phi: f64,
}

impl DifrPoint {
    #[inline]
    pub fn new(u: f64, abs: f64, phi: f64) -> Self {
        Self {u,abs,phi}
    }

    // angel
    #[inline]
    pub fn p_arg(&self) -> [f64;2] {
        [self.u, self.phi]
    }

    // abs
    #[inline]
    pub fn p_norm(&self) -> [f64;2] {
        [self.u, self.abs]
    }
}

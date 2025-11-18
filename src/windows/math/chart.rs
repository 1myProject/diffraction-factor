use std::f32::consts::FRAC_PI_4;

pub struct ChartParams {
    pub pitch: f32,
    pub yaw: f32,
    pub pitch_vel: f32,
    pub yaw_vel: f32,
}

impl Default for ChartParams {
    fn default() -> Self {
        Self {
            pitch:  0.3,
            yaw: FRAC_PI_4,
            pitch_vel: 0.,
            yaw_vel: 0.,
        }
    }
}


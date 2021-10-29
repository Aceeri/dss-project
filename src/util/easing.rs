#[derive(Debug, Copy, Clone)]
pub enum EaseMethod {
    Linear,
    EaseInOutCubic,
}

impl EaseMethod {
    pub fn ease(&self, start: f32, end: f32, percent: f32) -> f32 {
        start + (end - start) * self.progress(percent)
    }

    pub fn progress(&self, x: f32) -> f32 {
        match self {
            EaseMethod::Linear => x,
            EaseMethod::EaseInOutCubic => {
                if x < 0.5 {
                    // in
                    4.0 * x * x * x
                } else {
                    // out
                    let inner = -2.0 * x + 2.0;
                    1.0 - (inner * inner * inner) / 2.0
                }
            }
        }
    }
}

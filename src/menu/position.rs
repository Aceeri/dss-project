use glam::Vec3;

#[derive(Debug, Clone)]
pub struct Position {
    parent_position: Vec3, // Cumulative position of parents.
    local_position: Vec3,  // Local position relative to parent.
}

impl Position {
    pub fn new() -> Self {
        Self {
            parent_position: Vec3::ZERO,
            local_position: Vec3::ZERO,
        }
    }

    #[inline]
    pub fn local_position(&self) -> Vec3 {
        self.local_position
    }

    #[inline]
    pub fn parent_position(&self) -> Vec3 {
        self.parent_position
    }

    #[inline]
    pub fn absolute_position(&self) -> Vec3 {
        self.parent_position() + self.local_position()
    }
}

pub trait PositionHierarchy {
    fn position(&self) -> &Position;
    fn position_mut(&mut self) -> &mut Position;
    fn absolute_position(&self) -> Vec3 { self.position().absolute_position() }
    fn local_position(&self) -> Vec3 { self.position().local_position() }
    fn parent_position(&self) -> Vec3 { self.position().parent_position() }
    fn set_position(&mut self, local_position: &Vec3) {
        self.position_mut().local_position = *local_position;
        self.set_child_positions();
    }
    fn set_child_positions(&mut self) {}
    fn set_parent_position(&mut self, parent_position: &Vec3) {
        self.position_mut().parent_position = *parent_position;
        self.set_child_positions();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EaseMethod {
    Linear,
}

impl EaseMethod {
    pub fn ease(&self, start: f32, end: f32, percent: f32) -> f32 {
        match self {
            EaseMethod::Linear => start + (end - start) * percent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InterpPosition {
    position: Position,
    origin_local: Vec3,
    wanted_local: Vec3,
    counter: f64,
    duration: f64, // in seconds
    easing_methods: [EaseMethod; 3],
}

impl PositionHierarchy for InterpPosition {
    fn position(&self) -> &Position {
        &self.position
    }
    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }
    fn set_position(&mut self, local_position: &Vec3) {
        self.origin_local = self.position.local_position();
        self.wanted_local = *local_position;
    }
}

impl InterpPosition {
    pub fn new() -> Self {
        Self::from_position(Position::new())
    }

    pub fn from_position(position: Position) -> Self {
        let local = position.local_position.clone();

        Self {
            position,
            origin_local: local,
            wanted_local: local,
            counter: 0.0,
            duration: 0.0,
            easing_methods: [EaseMethod::Linear, EaseMethod::Linear, EaseMethod::Linear],
        }
    }

    pub fn interp_position(&mut self, position: Vec3, duration: f64,) {
        self.counter = 0.0;
        self.duration = duration;
        self.origin_local = self.position.local_position();
        self.wanted_local = position;
    }

    pub fn wanted_position(&self) -> Vec3 {
        self.wanted_local
    }

    pub fn update(&mut self, delta: f64) {
        if self.counter < self.duration {
            self.counter += delta;

            let percent = self.counter / self.duration;

            let x = (self.easing_methods[0]).ease(self.origin_local.x, self.wanted_local.x, percent as f32);
            let y = (self.easing_methods[1]).ease(self.origin_local.y, self.wanted_local.y, percent as f32);
            let z = (self.easing_methods[2]).ease(self.origin_local.z, self.wanted_local.z, percent as f32);

            self.position.local_position = Vec3::new(x, y, z);
        }
    }

    pub fn set_easing_method(&mut self, ease: EaseMethod) {
        self.set_easing_method_x(ease);
        self.set_easing_method_y(ease);
        self.set_easing_method_z(ease);
    }

    pub fn set_easing_method_x(&mut self, ease: EaseMethod) {
        self.easing_methods[0] = ease;
    }

    pub fn set_easing_method_y(&mut self, ease: EaseMethod) {
        self.easing_methods[1] = ease;
    }

    pub fn set_easing_method_z(&mut self, ease: EaseMethod) {
        self.easing_methods[2] = ease;
    }
}
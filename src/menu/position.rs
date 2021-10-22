use glam::Vec3;

#[derive(Debug, Clone)]
pub struct Position {
    parent_position: Vec3, // Cumulative position of parents.
    local_position: Vec3,  // Local position relative to parent.
}

impl Position {
    pub fn new() -> Position {
        Position {
            parent_position: Vec3::ZERO,
            local_position: Vec3::ZERO,
        }
    }

    #[inline]
    pub fn absolute_position(&self) -> Vec3 {
        self.parent_position + self.local_position
    }
}

pub trait PositionHierarchy {
    fn position(&self) -> &Position;
    fn position_mut(&mut self) -> &mut Position;
    fn absolute_position(&self) -> Vec3 {
        self.position().absolute_position()
    }
    fn set_position(&mut self, local_position: &Vec3) {
        self.position_mut().local_position = *local_position;
        self.set_child_positions();
    }
    fn set_child_positions(&mut self);
    fn set_parent_position(&mut self, parent_position: &Vec3) {
        self.position_mut().parent_position = *parent_position;
        self.set_child_positions();
    }
}

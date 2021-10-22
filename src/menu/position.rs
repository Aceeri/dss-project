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
}

// Maybe it would be better to just use Rc/Arc and have the children reference the parent's position?
pub trait PositionHierarchy {
    fn position(&self) -> &Position;
    fn position_mut(&mut self) -> &mut Position;
    fn absolute_position(&self) -> Vec3 {
        let position = self.position();
        position.parent_position + position.local_position
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

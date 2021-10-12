
use glam::Vec2;

pub struct Position {
    parent_position: Vec2, // Cumulative position of parents.
    local_position: Vec2, // Local position relative to parent.
}

impl Position {
    fn new() -> Position {
        Position {
            parent_position: Vec2::ZERO,
            local_position: Vec2::ZERO,
        }
    }
}

// Maybe it would be better to just use Rc/Arc and have the children reference the parent's position?
trait PositionHierarchy {
    fn position(&self) -> &Position;
    fn position_mut(&mut self) -> &mut Position;
    fn absolute_position(&self) -> Vec2 {
        let position = self.position();
        position.parent_position + position.local_position
    }
    fn set_position(&mut self, local_position: &Vec2) {
        self.position_mut().local_position = *local_position;
        self.set_child_positions();
    }
    fn set_child_positions(&mut self);
    fn set_parent_position(&mut self, parent_position: &Vec2) {
        self.position_mut().parent_position = *parent_position;
        self.set_child_positions();
    }
}

pub struct Menu {
    position: Position,
    // Vertical list of collections, each collection being a group of tiles.
    collections: Vec<Collection>,
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            position: Position::new(),
            collections: Vec::new(),
        }
    }
}

impl PositionHierarchy for Menu {
    fn position(&self) -> &Position { &self.position }
    fn position_mut(&mut self) -> &mut Position { &mut self.position }
    fn set_child_positions(&mut self) {
        let absolute = self.absolute_position();
        for collection in &mut self.collections {
            collection.set_parent_position(&absolute);
        }
    }
}

pub struct Collection {
    position: Position,
    tiles: Vec<Tile>,
}

impl PositionHierarchy for Collection {
    fn position(&self) -> &Position { &self.position }
    fn position_mut(&mut self) -> &mut Position { &mut self.position }
    fn set_child_positions(&mut self) {
        let absolute = self.absolute_position();
        for tile in &mut self.tiles {
            tile.set_parent_position(&absolute);
        }
    }
}

pub struct Tile {
    position: Position,
    //image_instance: ImageInstanceHandle,
}

impl PositionHierarchy for Tile {
    fn position(&self) -> &Position { &self.position }
    fn position_mut(&mut self) -> &mut Position { &mut self.position }
    fn set_child_positions(&mut self) { }
}

#[cfg(test)]
mod test {
    use crate::menu::{Position, PositionHierarchy, Menu, Collection, Tile};
    use glam::Vec2;

    #[test]
    fn hierarchy_test() {
        let mut menu = Menu {
            position: Position::new(),
            collections: vec![
                Collection {
                    position: Position::new(),
                    tiles: vec![
                        Tile {
                            position: Position::new(),
                        }
                    ]
                }
            ]
        };

        assert_eq!(menu.absolute_position(), Vec2::ZERO);
        assert_eq!(menu.collections[0].absolute_position(), Vec2::ZERO);
        assert_eq!(menu.collections[0].tiles[0].absolute_position(), Vec2::ZERO);

        let vec_10_10 = Vec2::new(10.0, 10.0);
        menu.set_position(&vec_10_10);

        assert_eq!(menu.absolute_position(), vec_10_10);
        assert_eq!(menu.collections[0].absolute_position(), vec_10_10);
        assert_eq!(menu.collections[0].tiles[0].absolute_position(), vec_10_10);

        menu.collections[0].tiles[0].set_position(&vec_10_10);
        assert_eq!(menu.collections[0].tiles[0].absolute_position(), Vec2::new(20.0, 20.0));

        menu.collections[0].set_position(&vec_10_10);
        assert_eq!(menu.collections[0].absolute_position(), Vec2::new(20.0, 20.0));
        assert_eq!(menu.collections[0].tiles[0].absolute_position(), Vec2::new(30.0, 30.0));
    }
}
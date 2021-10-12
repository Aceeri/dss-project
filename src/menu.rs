

use glam::Vec2;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

#[derive(Debug, Clone)]
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
pub trait PositionHierarchy {
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

pub trait EventGrab {
    // Pass along events to the UI elements.
    //
    // Return true to consume the event.
    fn input(&mut self, _event: &WindowEvent) -> bool { false }
}

#[derive(Debug, Clone)]
pub struct Menu {
    position: Position,

    // Vertical list of collections, each collection being a group of tiles.
    collections: Vec<Collection>,
    focused_collection: usize,
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            position: Position::new(),
            collections: Vec::new(),
            focused_collection: 0,
        }
    }

    pub fn push_collection(&mut self, mut collection: Collection) {
        collection.set_parent_position(&self.absolute_position());
        //collection.set_position(Vec2::new())
        self.collections.push(collection);
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

impl EventGrab for Menu {
    fn input(&mut self, event: &WindowEvent) -> bool {
        // Take up/down requests so we cycle through collections.
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(
                        direction @ VirtualKeyCode::Down |
                        direction @ VirtualKeyCode::Up
                    ),
                    ..
                },
                ..
            } => {
                println!("menu {:?}", direction);
                let mut new_focused_collection = match direction {
                    VirtualKeyCode::Up => self.focused_collection.saturating_sub(1),
                    VirtualKeyCode::Down => self.focused_collection.saturating_add(1),
                    _ => self.focused_collection,
                };


                if self.collections.len() > 0 {
                    if new_focused_collection > self.collections.len() - 1 {
                        new_focused_collection = self.collections.len() - 1;
                    }
                } else {
                    new_focused_collection = 0;
                }

                self.focused_collection = new_focused_collection;

                println!("new focused {:?}", self.focused_collection);
                return true;
            },
            _ => {},
        }

        if let Some(collection) = self.collections.get_mut(self.focused_collection) {
            collection.input(event)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Collection {
    position: Position,
    tiles: Vec<Tile>,
    focused_tile: usize,
}

impl Collection {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
            tiles: Vec::new(),
            focused_tile: 0,
        }
    }

    pub fn push_tile(&mut self, mut tile: Tile) {
        tile.set_parent_position(&self.absolute_position());
        //collection.set_position(Vec2::new())
        self.tiles.push(tile);
    }
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

impl EventGrab for Collection {
    fn input(&mut self, event: &WindowEvent) -> bool {
        // Take up/down requests so we cycle through collections.
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(
                        direction @ VirtualKeyCode::Left |
                        direction @ VirtualKeyCode::Right
                    ),
                    ..
                },
                ..
            } => {
                println!("collection {:?}", direction);
                let mut new_focused_tile = match direction {
                    VirtualKeyCode::Left => self.focused_tile.saturating_sub(1),
                    VirtualKeyCode::Right => self.focused_tile.saturating_add(1),
                    _ => self.focused_tile,
                };


                println!("{:?}", self.tiles);
                if self.tiles.len() > 0 {
                    if new_focused_tile > self.tiles.len() - 1 {
                        new_focused_tile = self.tiles.len() - 1;
                    }
                } else {
                    new_focused_tile = 0;
                }

                self.focused_tile = new_focused_tile;
                println!("new focused {:?}", self.focused_tile);
                return true;
            },
            _ => {},
        }

        if let Some(tile) = self.tiles.get_mut(self.focused_tile) {
            tile.input(event)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    position: Position,
    //image_instance: ImageInstanceHandle,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
        }
    }
}

impl PositionHierarchy for Tile {
    fn position(&self) -> &Position { &self.position }
    fn position_mut(&mut self) -> &mut Position { &mut self.position }
    fn set_child_positions(&mut self) { }
}

impl EventGrab for Tile {
    fn input(&mut self, _event: &WindowEvent) -> bool {
        // Do nothing for now.
        false
    }
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
                    ],
                    focused_tile: 0,
                }
            ],
            focused_collection: 0,
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
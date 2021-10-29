use anyhow::Result;
use glam::{Vec2, Vec3};
use image::EncodableLayout;
use std::task::Poll;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{
    grabber::HttpGrabber,
    home::Home,
    renderer::Renderer,

};

use super::{
    prelude::*,
    Collection,
    Tile,
};

pub static HOME_URL: &'static str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
pub static ASPECT_RATIO_STRING: &'static str = "1.78";
pub const COLLECTION_SPACING: f32 = 0.5 * SCALE;

#[derive(Debug, Clone)]
pub struct Menu {
    position: InterpPosition,

    // Vertical list of collections, each collection being a group of tiles.
    collections: Vec<Collection>,
    focused_collection: usize,
    focused_tile: usize,

    // Indices of collections and tiles to iterate through slowly for rendering.
    partial_collection: usize,
    partial_tile: usize,

    // List of tiles that need to be re-rendered immediately.
    dirty_list: Vec<usize>,

    home: Option<Home>,
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            position: InterpPosition::new(),
            collections: Vec::new(),
            focused_collection: 0,
            focused_tile: 0,
            home: None,

            partial_collection: 0,
            partial_tile: 0,
            dirty_list: Vec::new(),
        }
    }

    pub fn update(&mut self, delta: f64) {
        self.position.update(delta);
        self.set_child_positions();
    }

    pub fn push_collection(&mut self, mut collection: Collection) {
        collection.set_parent_position(&self.absolute_position());
        collection.set_position(&Vec3::new(
            0.0,
            (1.0 * SCALE + COLLECTION_SPACING) * self.collections.len() as f32,
            0.0,
        ));
        self.collections.push(collection);
    }

    pub fn construct_home(&mut self) {
        let mut new_collections = Vec::new();

        if let Some(home) = &self.home {
            for container in &home.data.standard_collection.containers {
                let text_details = container.set.text.title.full.details();
                let mut collection = Collection::new(text_details.content.clone());

                if let Some(items) = &container.set.items {
                    for item in items {
                        // Get images with the aspect ratio we want.
                        if let Some(image) = item.image.tile.get(ASPECT_RATIO_STRING) {
                            let details = image.details();
                            let mut tile = Tile::new(details.clone());
                            tile.set_size(Vec2::new(1.78 * SCALE, 1.0 * SCALE));
                            collection.push_tile(tile);
                        }
                    }
                }

                new_collections.push(collection);
            }
        }

        for new_collection in new_collections {
            self.push_collection(new_collection);
        }
    }

    pub fn focus_tile(&mut self, collection_index: usize, tile_index: usize) {
        if let Some(collection) = self.collections.get_mut(self.focused_collection) {
            collection.focus_tile(self.focused_tile, false);
        }

        self.focused_collection = collection_index;
        self.focused_tile = tile_index;

        if let Some(collection) = self.collections.get_mut(self.focused_collection) {
            collection.focus_tile(self.focused_tile, true);
        }
    }
}

impl PositionHierarchy for Menu {
    fn position(&self) -> &Position {
        self.position.position()
    }
    fn position_mut(&mut self) -> &mut Position {
        self.position.position_mut()
    }
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
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::LShift),
                        ..
                    },
                ..
            } => {
                match self.collections.get(self.focused_collection) {
                    Some(collection) => {
                        println!("{:?}", collection.absolute_position());
                    }
                    None => {}
                }
                return true;
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::RShift),
                        ..
                    },
                ..
            } => {
                let new_position = self.position.wanted_position() - Vec3::new(0.0, 100.0, 0.0);
                self.position.interp_position(new_position, 0.2);
                return true;
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode:
                            Some(
                                direction @ VirtualKeyCode::Down
                                | direction @ VirtualKeyCode::Up
                                | direction @ VirtualKeyCode::Left
                                | direction @ VirtualKeyCode::Right,
                            ),
                        ..
                    },
                ..
            } => {
                let mut new_focused_tile = self.focused_tile;
                let mut new_focused_collection = self.focused_collection;

                match direction {
                    VirtualKeyCode::Up => {
                        new_focused_collection = new_focused_collection.saturating_sub(1)
                    }
                    VirtualKeyCode::Down => {
                        new_focused_collection = new_focused_collection.saturating_add(1)
                    }
                    VirtualKeyCode::Left => new_focused_tile = new_focused_tile.saturating_sub(1),
                    VirtualKeyCode::Right => new_focused_tile = new_focused_tile.saturating_add(1),
                    _ => {}
                };

                if self.collections.len() > 0 {
                    if new_focused_collection > self.collections.len() - 1 {
                        new_focused_collection = self.collections.len() - 1;
                    }

                    let focused_tiles = self.collections[new_focused_collection].tiles.len();
                    if focused_tiles > 0 && new_focused_tile > focused_tiles - 1 {
                        new_focused_tile = focused_tiles - 1;
                    }
                } else {
                    new_focused_collection = 0;
                    new_focused_tile = 0;
                }

                self.focus_tile(new_focused_collection, new_focused_tile);
                return true;
            }
            _ => {}
        }

        if let Some(collection) = self.collections.get_mut(self.focused_collection) {
            if !collection.input(event) {
                if let Some(tile) = collection.tiles.get_mut(self.focused_tile) {
                    tile.input(event)
                } else {
                    false
                }
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl Pollable for Menu {
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool> {
        match &self.home {
            Some(_) => {
                let mut done = true;
                for collection in &mut self.collections {
                    done = done && collection.poll(grabber)?;
                }

                Ok(done)
            }
            None => {
                match grabber.poll_request(HOME_URL.to_owned())? {
                    Poll::Pending => Ok(false),
                    Poll::Ready(home) => {
                        println!("got homepage, rendering page now");
                        // Construct initial homepage.
                        let home = home?;
                        self.home = Some(serde_json::from_slice(home.as_bytes())?);
                        self.construct_home();
                        Ok(false)
                    }
                }
            }
        }
    }
}

impl SetRenderDetails for Menu {
    fn set_render_details(&mut self, renderer: &mut Renderer) {
        for collection in &mut self.collections {
            collection.set_render_details(renderer);
        }
    }

    fn partial_set_render_details(&mut self, renderer: &mut Renderer) {
        if let Some(collection) = self.collections.get_mut(self.partial_collection) {
            collection.partial_set_render_details(renderer);

            if let Some(tile) = collection.tiles.get_mut(self.partial_tile) {
                tile.set_render_details(renderer);
                self.partial_tile += 1;
            }

            if self.partial_tile >= collection.tiles.len() {
                self.partial_tile = 0;
                self.partial_collection += 1;

                if self.partial_collection >= self.collections.len() {
                    self.partial_collection = 0;
                }
            }
        } else {
            self.partial_tile = 0;
            self.partial_collection = 0;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::home::ImageDetails;
    use crate::menu::{Collection, Menu, PositionHierarchy, Tile};
    use glam::Vec3;

    #[test]
    fn hierarchy_test() {
        let dummy_details: ImageDetails = ImageDetails {
            master_width: 0,
            master_height: 0,
            url: "dummy".to_owned(),
        };

        let mut menu = Menu::new();

        let mut collection = Collection::new("dummy".to_owned());
        let tile = Tile::new(dummy_details.clone());
        collection.push_tile(tile);

        menu.push_collection(collection);

        assert_eq!(menu.absolute_position(), Vec3::ZERO);
        assert_eq!(menu.collections[0].absolute_position(), Vec3::ZERO);
        assert_eq!(menu.collections[0].tiles[0].absolute_position(), Vec3::ZERO);

        let vec_10_10 = Vec3::new(10.0, 10.0, 10.0);
        menu.set_position(&vec_10_10);

        assert_eq!(menu.absolute_position(), vec_10_10);
        assert_eq!(menu.collections[0].absolute_position(), vec_10_10);
        assert_eq!(menu.collections[0].tiles[0].absolute_position(), vec_10_10);

        menu.collections[0].tiles[0].set_position(&vec_10_10);
        assert_eq!(
            menu.collections[0].tiles[0].absolute_position(),
            Vec3::new(20.0, 20.0, 20.0)
        );

        menu.collections[0].set_position(&vec_10_10);
        assert_eq!(
            menu.collections[0].absolute_position(),
            Vec3::new(20.0, 20.0, 20.0)
        );
        assert_eq!(
            menu.collections[0].tiles[0].absolute_position(),
            Vec3::new(30.0, 30.0, 30.0)
        );

        let mut new_collection = Collection::new("dummy".to_owned());
        let new_tile = Tile::new(dummy_details);
        new_collection.push_tile(new_tile);
        menu.push_collection(new_collection);
        println!("{:?}", menu.absolute_position());
        println!("{:?}", menu.collections[1].absolute_position());
        println!("{:?}", menu.collections[1].tiles[0].absolute_position());
    }
}

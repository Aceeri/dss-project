
use glam::Vec2;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use std::task::{Poll};
use anyhow::Result;
use image::EncodableLayout;

use crate::{
    grabber::HttpGrabber,
    renderer::{Renderer, ImageInstanceHandle, Instance},
    home::{ImageDetails, Home},
};

pub static HOME_URL: &'static str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
pub static ASPECT_RATIO: &'static str = "1.78";

#[derive(Debug, Clone)]
pub struct Position {
    parent_position: Vec2, // Cumulative position of parents.
    local_position: Vec2,  // Local position relative to parent.
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
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
}

pub trait Pollable {
    // Might be better to genericize it past just http grabbing for polling, but this is fine for now.
    //
    // Poll responses for http responses, return Ok(true) if done polling.
    fn poll(&mut self, grabber: &HttpGrabber) -> Result<bool>;
}

pub trait SetRenderDetails {
    fn set_render_details(&mut self, renderer: &mut crate::renderer::Renderer);
}

#[derive(Debug, Clone)]
pub struct Menu {
    position: Position,

    // Vertical list of collections, each collection being a group of tiles.
    collections: Vec<Collection>,
    focused_collection: usize,

    home: Option<Home>,
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            position: Position::new(),
            collections: Vec::new(),
            focused_collection: 0,
            home: None,
        }
    }

    pub fn push_collection(&mut self, mut collection: Collection) {
        collection.set_parent_position(&self.absolute_position());
        collection.set_position(&Vec2::new(0.0, 0.2 * self.collections.len() as f32));
        self.collections.push(collection);
    }

    pub fn construct_home(&mut self) {
        println!("constructing home");

        let mut new_collections = Vec::new();

        if let Some(home) = &self.home {
            for container in &home.data.standard_collection.containers {
                let mut collection = Collection::new();
                println!("new collection");
                
                if let Some(items) = &container.set.items {
                    for item in items {
                        if let Some(image) = item.image.tile.get(ASPECT_RATIO) {
                            if let Some(series) = &image.series {
                                if let Some(details) = series.get("default") {
                                    let mut tile = Tile::new(details.clone());
                                    tile.size = Vec2::new(0.2, 0.2);
                                    println!("new tile");
                                    collection.push_tile(tile);
                                }
                            }
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
}

impl PositionHierarchy for Menu {
    fn position(&self) -> &Position {
        &self.position
    }
    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
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
                        virtual_keycode:
                            Some(direction @ VirtualKeyCode::Down | direction @ VirtualKeyCode::Up),
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
            }
            _ => {}
        }

        if let Some(collection) = self.collections.get_mut(self.focused_collection) {
            collection.input(event)
        } else {
            false
        }
    }
}

impl Pollable for Menu {
    fn poll(&mut self, grabber: &HttpGrabber) -> Result<bool> {
        match &self.home {
            Some(home) => {
                let mut done = false;
                for collection in &mut self.collections {
                    done = done || collection.poll(grabber)?;
                }

                Ok(done)
            },
            None => {
                match grabber.poll(HOME_URL.to_owned())? {
                    Poll::Pending => Ok(false),
                    Poll::Ready(home) => {
                        // Construct initial homepage.
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
        tile.set_position(&Vec2::new(0.2 * self.tiles.len() as f32, 0.0));
        println!("{:?}", tile.absolute_position());
        self.tiles.push(tile);
    }
}

impl PositionHierarchy for Collection {
    fn position(&self) -> &Position {
        &self.position
    }
    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }
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
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode:
                            Some(
                                direction @ VirtualKeyCode::Left
                                | direction @ VirtualKeyCode::Right,
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
            }
            _ => {}
        }

        if let Some(tile) = self.tiles.get_mut(self.focused_tile) {
            tile.input(event)
        } else {
            false
        }
    }
}

impl Pollable for Collection {
    fn poll(&mut self, grabber: &HttpGrabber) -> Result<bool> {
        let mut done = false;
        for tile in &mut self.tiles {
            done = done || tile.poll(grabber)?;
        }

        Ok(done)
    }
}

impl SetRenderDetails for Collection {
    fn set_render_details(&mut self, renderer: &mut Renderer) {
        // TODO: add text rendering to this.

        for tile in &mut self.tiles {
            tile.set_render_details(renderer);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    position: Position,
    size: Vec2,

    image_instance: Option<ImageInstanceHandle>,
    image_bytes: Option<bytes::Bytes>,
    details: ImageDetails,
}

impl Tile {
    pub fn new(details: ImageDetails) -> Self {
        Self {
            position: Position::new(),
            size: Vec2::new(0.1, 0.1),

            image_instance: None,
            image_bytes: None,
            details: details,
        }
    }
}

impl Pollable for Tile {
    fn poll(&mut self, grabber: &HttpGrabber) -> Result<bool> {
        match &self.image_bytes {
            Some(_image_bytes) => Ok(true),
            None => {
                if let Poll::Ready(bytes) = grabber.poll(self.details.url.clone())? {
                    self.image_bytes = Some(bytes);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}

impl PositionHierarchy for Tile {
    fn position(&self) -> &Position {
        &self.position
    }
    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }
    fn set_child_positions(&mut self) {}
}

impl EventGrab for Tile {
    fn input(&mut self, _event: &WindowEvent) -> bool {
        // Do nothing for now.
        false
    }
}

impl SetRenderDetails for Tile {
    fn set_render_details(&mut self, renderer: &mut Renderer) {
        match self.image_instance {
            Some(image_instance) => {
                renderer.set_image_instance_position(image_instance, Instance {
                    position: self.absolute_position().into(),
                    size: self.size.into(),
                });
            },
            None => {
                let texture_bytes = include_bytes!("renderer/test.png");
                let texture = crate::renderer::Texture::from_bytes(&renderer.device, &renderer.queue, texture_bytes, "test.png").expect("created texture");

                let image_handle = renderer.create_image(texture);
                let instance_handle = renderer.create_instance(Instance {
                    position: self.absolute_position().into(),
                    size: self.size.into(),
                });

                self.image_instance = Some(renderer.create_image_instance(image_handle, instance_handle));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::menu::{Collection, Menu, Position, PositionHierarchy, Tile};
    use crate::home::{ImageDetails};
    use glam::Vec2;

    #[test]
    fn hierarchy_test() {
        let mut menu = Menu {
            home: None,
            position: Position::new(),
            collections: vec![Collection {
                position: Position::new(),
                tiles: vec![Tile {
                    position: Position::new(),
                    size: Vec2::ZERO,
                    image_bytes: None,
                    image_instance: None,
                    details: ImageDetails {
                        master_width: 0,
                        master_height: 0,
                        url: "dummy".to_owned(),
                    },
                }],
                focused_tile: 0,
            }],
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
        assert_eq!(
            menu.collections[0].tiles[0].absolute_position(),
            Vec2::new(20.0, 20.0)
        );

        menu.collections[0].set_position(&vec_10_10);
        assert_eq!(
            menu.collections[0].absolute_position(),
            Vec2::new(20.0, 20.0)
        );
        assert_eq!(
            menu.collections[0].tiles[0].absolute_position(),
            Vec2::new(30.0, 30.0)
        );
    }
}

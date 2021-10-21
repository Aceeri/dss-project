
use glam::{Vec2, Vec3};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use std::task::{Poll};
use anyhow::Result;
use image::EncodableLayout;

use crate::{
    grabber::HttpGrabber,
    renderer::{Renderer, ImageInstanceHandle, Instance, Texture},
    home::{ImageDetails, Home},
};

pub static HOME_URL: &'static str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
pub static ASPECT_RATIO_STRING: &'static str = "1.78";
pub const ASPECT_RATIO: f32 = 1.78;
pub const TILE_SPACING: f32 = 0.2;
pub const COLLECTION_SPACING: f32 = 0.5;

#[derive(Debug, Clone)]
pub struct Position {
    parent_position: Vec3, // Cumulative position of parents.
    local_position: Vec3,  // Local position relative to parent.
}

impl Position {
    fn new() -> Position {
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
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool>;
}

pub trait SetRenderDetails {
    fn set_render_details(&mut self, renderer: &mut crate::renderer::Renderer);

    // Only set portions of the renderer every frame instead of all at once.
    fn partial_set_render_details(&mut self, renderer: &mut crate::renderer::Renderer) { self.set_render_details(renderer); }
}

#[derive(Debug, Clone)]
pub struct Menu {
    position: Position,

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
            position: Position::new(),
            collections: Vec::new(),
            focused_collection: 0,
            focused_tile: 0,
            home: None,

            partial_collection: 0,
            partial_tile: 0,
            dirty_list: Vec::new(),
        }
    }

    pub fn push_collection(&mut self, mut collection: Collection) {
        collection.set_parent_position(&self.absolute_position());
        collection.set_position(&Vec3::new(0.0, (1.0 + COLLECTION_SPACING) * self.collections.len() as f32, 0.0));
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
                        // Get images with the aspect ratio we want.
                        if let Some(image) = item.image.tile.get(ASPECT_RATIO_STRING) {
                            let details = image.details();
                            let mut tile = Tile::new(details.clone());
                            tile.size = Vec2::new(1.78, 1.0);
                            println!("new tile");
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

    pub fn set_focused_tile(&mut self, collection_index: usize, tile_index: usize) {
        if let Some(collection) = self.collections.get_mut(self.focused_collection) {
            if let Some(tile) = collection.tiles.get_mut(self.focused_tile) {
                tile.selected = false;
                collection.dirty_list.push(self.focused_tile);
            }
        }

        self.focused_collection = collection_index;
        self.focused_tile = tile_index;

        if let Some(collection) = self.collections.get_mut(self.focused_collection) {
            if let Some(tile) = collection.tiles.get_mut(self.focused_tile) {
                tile.selected = true;
                collection.dirty_list.push(self.focused_tile);
            }
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
                            Some(
                                direction @ VirtualKeyCode::Down |
                                direction @ VirtualKeyCode::Up |
                                direction @ VirtualKeyCode::Left |
                                direction @ VirtualKeyCode::Right
                            ),
                        ..
                    },
                ..
            } => {
                println!("menu {:?}", direction);
                let mut new_focused_tile = self.focused_tile;
                let mut new_focused_collection = self.focused_collection;

                match direction {
                    VirtualKeyCode::Up => new_focused_collection = new_focused_collection.saturating_sub(1),
                    VirtualKeyCode::Down => new_focused_collection = new_focused_collection.saturating_add(1),
                    VirtualKeyCode::Left => new_focused_tile = new_focused_tile.saturating_sub(1),
                    VirtualKeyCode::Right => new_focused_tile = new_focused_tile.saturating_add(1),
                    _ => { },
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

                self.set_focused_tile(new_focused_collection, new_focused_tile);
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
            Some(home) => {
                let mut done = true;
                for collection in &mut self.collections {
                    done = done && collection.poll(grabber)?;
                }

                Ok(done)
            },
            None => {
                match grabber.poll_request(HOME_URL.to_owned())? {
                    Poll::Pending => Ok(false),
                    Poll::Ready(home) => {
                        // Construct initial homepage.
                        println!("constructing home");
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

#[derive(Debug, Clone)]
pub struct Collection {
    position: Position,
    tiles: Vec<Tile>,
    selected: bool,

    dirty_list: Vec<usize>,
}

impl Collection {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
            tiles: Vec::new(),
            selected: false,
            
            dirty_list: Vec::new(),
        }
    }

    pub fn push_tile(&mut self, mut tile: Tile) {
        tile.set_parent_position(&self.absolute_position());
        tile.set_position(&Vec3::new((ASPECT_RATIO + TILE_SPACING) * self.tiles.len() as f32, 0.0, 0.0));
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
        false
    }
}

impl Pollable for Collection {
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool> {
        let mut done = true;
        for tile in &mut self.tiles {
            done = done && tile.poll(grabber)?;
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

    fn partial_set_render_details(&mut self, renderer: &mut Renderer) {
        if let Some(index) = self.dirty_list.pop() {
            self.tiles.get_mut(index).map(|tile| tile.set_render_details(renderer));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    position: Position,
    size: Vec2,
    selected: bool,

    image_instance: Option<ImageInstanceHandle>,
    image_bytes: Option<bytes::Bytes>,
    details: ImageDetails,
}

impl Tile {
    pub fn new(details: ImageDetails) -> Self {
        Self {
            position: Position::new(),
            size: Vec2::new(0.1, 0.1),
            selected: false,

            image_instance: None,
            image_bytes: None,
            details: details,
        }
    }
}

impl Pollable for Tile {
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool> {
        match &self.image_bytes {
            Some(_image_bytes) => Ok(true),
            None => {
                if let Poll::Ready(bytes) = grabber.poll_request(self.details.url.clone())? {
                    //println!("got response");
                    self.image_bytes = Some(bytes?.clone());
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
    fn input(&mut self, event: &WindowEvent) -> bool {
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
                println!("details: {:?}", self.details.url);
                true
            }
            _ => false,
        }
    }
}

impl SetRenderDetails for Tile {
    fn set_render_details(&mut self, renderer: &mut Renderer) {
        match (&self.image_instance, &self.image_bytes) {
            (Some(image_instance), _) => {
                let mut size = self.size;
                let mut position = self.absolute_position();
                if self.selected {
                    let selected_scaling = Vec2::new(1.2,1.2);
                    size = size * selected_scaling;
                    position.z -= 1.0;
                }

                renderer.set_image_instance_position(*image_instance, Instance {
                    position: self.absolute_position().into(),
                    size: size.into(),
                });
            }
            (None, Some(texture_bytes)) => {
                let texture = match Texture::from_bytes(&renderer.device, &renderer.queue, texture_bytes.as_bytes(), "test.jpeg") {
                    Ok(texture) => texture,
                    Err(_) => {
                        let fallback_bytes = include_bytes!("renderer/test.png");
                        Texture::from_bytes(&renderer.device, &renderer.queue, fallback_bytes, "fallback.png").expect("created texture")
                    },
                };

                let image_handle = renderer.create_image(texture);
                let instance_handle = renderer.create_instance(Instance {
                    position: self.absolute_position().into(),
                    size: self.size.into(),
                });

                self.image_instance = Some(renderer.create_image_instance(image_handle, instance_handle));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use crate::menu::{Collection, Menu, Position, PositionHierarchy, Tile};
    use crate::home::{ImageDetails};
    use glam::{Vec2, Vec3};

    #[test]
    fn hierarchy_test() {
        let dummy_details: ImageDetails = ImageDetails {
            master_width: 0,
            master_height: 0,
            url: "dummy".to_owned(),
        };

        let mut menu = Menu {
            home: None,
            position: Position::new(),
            collections: vec![Collection {
                position: Position::new(),
                tiles: vec![Tile {
                    position: Position::new(),
                    size: Vec2::ZERO,
                    selected: false,
                    image_bytes: None,
                    image_instance: None,
                    details: dummy_details.clone(),
                }],
                selected: false,
            }],
            focused_collection: 0,
            focused_tile: 0,
        };

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

        let mut new_collection = Collection::new();
        let mut new_tile = Tile::new(dummy_details);
        new_collection.push_tile(new_tile);
        menu.push_collection(new_collection);
        println!("{:?}", menu.absolute_position());
        println!("{:?}", menu.collections[1].absolute_position());
        println!("{:?}", menu.collections[1].tiles[0].absolute_position());
    }
}

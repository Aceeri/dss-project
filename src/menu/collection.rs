use anyhow::Result;
use glam::Vec3;
use winit::event::WindowEvent;

use crate::{grabber::HttpGrabber, renderer::Renderer};

use super::{prelude::*, Tile, ASPECT_RATIO};

pub const TILE_SPACING: f32 = 0.2 * SCALE;

#[derive(Debug, Clone)]
pub struct Collection {
    position: Position,
    title_text: Text,
    pub tiles: Vec<Tile>,
    focused: bool,

    dirty_list: Vec<usize>,
}

impl Collection {
    pub fn new(title: String) -> Self {
        let mut title_text = Text::new(title);
        title_text.set_position(&Vec3::new(10.0, 0.0, 0.0));

        let mut new_collection = Self {
            position: Position::new(),
            title_text: title_text,
            tiles: Vec::new(),
            focused: false,

            dirty_list: Vec::new(),
        };

        new_collection.set_child_positions();
        new_collection
    }

    pub fn push_tile(&mut self, mut tile: Tile) {
        tile.set_parent_position(&self.absolute_position());
        tile.set_position(&Vec3::new(
            (ASPECT_RATIO * SCALE + TILE_SPACING) * self.tiles.len() as f32,
            0.0,
            0.0,
        ));
        self.tiles.push(tile);
    }

    pub fn focus_tile(&mut self, tile_index: usize, focused: bool) {
        if let Some(tile) = self.tiles.get_mut(tile_index) {
            tile.set_focus(focused);
            self.dirty_list.push(tile_index);
        }
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
        self.title_text.set_parent_position(&absolute);

        for tile in &mut self.tiles {
            tile.set_parent_position(&absolute);
        }
    }
}

impl Input for Collection {
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
}

impl Poll for Collection {
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool> {
        let mut done = true;
        for tile in &mut self.tiles {
            done = done && tile.poll(grabber)?;
        }

        Ok(done)
    }
}

impl Draw for Collection {
    fn set_render_details(&mut self, renderer: &mut Renderer) {
        self.title_text.set_render_details(renderer);

        for tile in &mut self.tiles {
            tile.set_render_details(renderer);
        }
    }

    fn partial_set_render_details(&mut self, renderer: &mut Renderer) {
        if let Some(index) = self.dirty_list.pop() {
            self.tiles
                .get_mut(index)
                .map(|tile| tile.set_render_details(renderer));
        }
    }
}

use anyhow::Result;
use glam::Vec3;
use winit::event::WindowEvent;

use crate::{
    grabber::HttpGrabber,
    menu::{prelude::*, Tile, ASPECT_RATIO},
    renderer::Renderer,
};

pub const TILE_SPACING: f32 = 0.0;

#[derive(Debug, Clone)]
pub struct Collection {
    position: Position,
    title: String,
    pub tiles: Vec<Tile>,
    focused: bool,

    dirty_list: Vec<usize>,
}

impl Collection {
    pub fn new(title: String) -> Self {
        Self {
            position: Position::new(),
            title: title,
            tiles: Vec::new(),
            focused: false,

            dirty_list: Vec::new(),
        }
    }

    pub fn push_tile(&mut self, mut tile: Tile) {
        tile.set_parent_position(&self.absolute_position());
        tile.set_position(&Vec3::new(
            (ASPECT_RATIO + TILE_SPACING) * self.tiles.len() as f32,
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
        for tile in &mut self.tiles {
            tile.set_parent_position(&absolute);
        }
    }
}

impl EventGrab for Collection {
    fn input(&mut self, _event: &WindowEvent) -> bool {
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
            self.tiles
                .get_mut(index)
                .map(|tile| tile.set_render_details(renderer));
        }
    }
}

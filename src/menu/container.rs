use anyhow::Result;
use glam::{Vec2, Vec3};
use image::EncodableLayout;
use std::task::Poll as PollTask;
use uuid::Uuid;
use winit::event::WindowEvent;

use crate::{
    grabber::HttpGrabber,
    home::{Item, RefSet},
    renderer::Renderer,
};

use super::{prelude::*, Tile, ASPECT_RATIO};

pub const TILE_SPACING: f32 = 0.25 * SCALE;
pub static ASPECT_RATIO_STRING: &'static str = "1.78";

#[derive(Debug, Clone)]
pub struct Container {
    position: Position,
    title_text: Text,
    ref_id: Option<Uuid>,
    refset_loaded: bool,

    pub tiles: Vec<Tile>,
    focused: bool,

    dirty_list: Vec<usize>,
}

impl Container {
    pub fn new(title: String, ref_id: Option<Uuid>) -> Self {
        let mut title_text = Text::new(title);
        title_text.set_position(&Vec3::new(0.0, 0.0, 0.0));
        title_text.set_font_size(36.0);

        let mut new_container = Self {
            position: Position::new(),
            title_text: title_text,
            ref_id: ref_id,
            refset_loaded: false,

            tiles: Vec::new(),
            focused: false,

            dirty_list: Vec::new(),
        };

        new_container.set_child_positions();
        new_container
    }

    pub fn add_items(&mut self, items: &Vec<Item>) {
        for item in items {
            // Get images with the aspect ratio we want.
            if let Some(image) = item.image.tile.get(ASPECT_RATIO_STRING) {
                let image_details = image.details();
                let title = item.text.title.full.details().content.clone();
                let mut tile = Tile::new(title, image_details.clone());
                tile.set_size(Vec2::new(1.78 * SCALE, 1.0 * SCALE));
                self.push_tile(tile);
            }
        }
    }

    pub fn remove_tile(&mut self, tile_index: usize) {
        self.tiles.swap_remove(tile_index);
    }

    pub fn reset_tile_positions(&mut self) {
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            tile.set_position(&Container::tile_position(index));
        }
    }

    pub fn tile_position(tile_index: usize) -> Vec3 {
        Vec3::new( 
            0.75 * SCALE + (ASPECT_RATIO * SCALE + TILE_SPACING) * tile_index as f32,
            SCALE,
            0.0,
        )
    }

    pub fn construct_refset(&mut self, refset: &RefSet) {
        if let Some(items) = &refset.data.set().items {
            self.add_items(items);
        }

        self.refset_loaded = true;
    }

    pub fn push_tile(&mut self, mut tile: Tile) {
        tile.set_parent_position(&self.absolute_position());
        tile.set_position(&Container::tile_position(self.tiles.len()));
        self.tiles.push(tile);
    }

    pub fn focus_tile(&mut self, tile_index: usize, focused: bool) {
        if let Some(tile) = self.tiles.get_mut(tile_index) {
            tile.set_focus(focused);
            self.dirty_list.push(tile_index);
        }
    }
}

impl UpdateDelta for Container {
    fn update_delta(&mut self, delta: f64) {
        for tile in &mut self.tiles {
            tile.update_delta(delta);
        }
    }
}

impl PositionHierarchy for Container {
    fn position(&self) -> &Position { &self.position }
    fn position_mut(&mut self) -> &mut Position { &mut self.position }
    fn set_child_positions(&mut self) {
        let absolute = self.absolute_position();
        self.title_text.set_parent_position(&absolute);

        for tile in &mut self.tiles {
            tile.set_parent_position(&absolute);
        }
    }
}

impl Input for Container {
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
}

impl Poll for Container {
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool> {
        let mut done = true;
        for tile in &mut self.tiles {
            done = done && tile.poll(grabber)?;
        }

        // poll for dynamic ref sets.
        if !self.refset_loaded {
            if let Some(ref_id) = self.ref_id {
                let dynamic_refset = format!(
                    "https://cd-static.bamgrid.com/dp-117731241344/sets/{}.json",
                    ref_id.to_hyphenated().to_string()
                );
                done = done
                    && match grabber.poll_request(dynamic_refset.clone()) {
                        Ok(PollTask::Pending) => false,
                        Ok(PollTask::Ready(refset)) => {
                            println!("got refset: {}", dynamic_refset);

                            let refset = refset?;
                            let refset = serde_json::from_slice(refset.as_bytes())?;
                            self.construct_refset(&refset);
                            false
                        }
                        Err(err) => {
                            eprintln!("fetch dynamic refset: {:?}", err);
                            false
                        }
                    };
            }
        }

        Ok(done)
    }
}

impl Draw for Container {
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

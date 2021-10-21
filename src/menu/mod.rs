

pub const ASPECT_RATIO: f32 = 1.78;

pub mod menu;
pub mod collection;
pub mod tile;
pub mod position;
pub mod prelude;

pub use menu::{Menu};
pub use tile::{Tile};
pub use collection::{Collection};
pub use position::{Position, PositionHierarchy};

use winit::event::WindowEvent;
use crate::{
    grabber::HttpGrabber,
};
use anyhow::Result;

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
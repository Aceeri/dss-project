
pub const ASPECT_RATIO: f32 = 1.78;
pub const SCALE: f32 = 100.0;

pub mod collection;
pub mod menu;
pub mod position;
pub mod prelude;
pub mod tile;
pub mod text;

pub use collection::Collection;
pub use menu::Menu;
pub use position::{InterpPosition, Position, PositionHierarchy};
pub use tile::Tile;
pub use text::Text;

use crate::grabber::HttpGrabber;
use anyhow::Result;
use winit::event::WindowEvent;

pub trait Input {
    // Pass along events to the UI elements.
    //
    // Return true to consume the event.
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
}

pub trait Poll {
    // Might be better to genericize it past just http grabbing for polling, but this is fine for now.
    //
    // Poll responses for http responses, return Ok(true) if done polling.
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool>;
}

pub trait Draw {
    fn set_render_details(&mut self, renderer: &mut crate::renderer::Renderer);

    // Only set portions of the renderer every frame instead of all at once.
    fn partial_set_render_details(&mut self, renderer: &mut crate::renderer::Renderer) {
        self.set_render_details(renderer);
    }
}

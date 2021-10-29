use anyhow::Result;
use glam::{Vec2, Vec3};
use image::EncodableLayout;
use std::task::Poll as PollTask;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{
    grabber::HttpGrabber,
    home::ImageDetails,
    menu::prelude::*,
    renderer::{Renderer, SpriteId, SpriteInstance, Texture},
    util::EaseMethod,
};

#[derive(Debug, Clone)]
pub struct Tile {
    position: Position,
    size: Vec2,
    focused: bool,

    title: String,

    sprite: Option<SpriteId>,
    fallback_text: Option<Text>,
    texture_bytes: Option<bytes::Bytes>,
    details: ImageDetails,

    counter: f64,
    duration: f64,
    alpha: f32,
}

impl Tile {
    pub fn new(title: String, details: ImageDetails) -> Self {
        Self {
            position: Position::new(),
            size: Vec2::new(0.2, 0.2),
            focused: false,

            title: title,
            fallback_text: None,

            sprite: None,
            texture_bytes: None,
            details: details,
            counter: 0.0,
            duration: 0.0,
            alpha: 0.0,
        }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn size(&self) -> &Vec2 {
        &self.size
    }

    pub fn focus(&self) -> bool {
        self.focused
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focused = focus;
    }

    pub fn focused_instance(&self) -> SpriteInstance {
        let mut size = self.size;
        let mut position = self.absolute_position();
        if self.focused {
            let focused_scaling = Vec2::new(1.2, 1.2);
            size = size * focused_scaling;
            position.z += 1.0;
        }

        SpriteInstance {
            size: size.into(),
            position: position.into(),
            alpha: self.alpha,
        }
    }
}

impl UpdateDelta for Tile {
    fn update_delta(&mut self, delta: f64) {
        if self.counter < self.duration {
            self.counter += delta as f64;
            self.alpha = EaseMethod::EaseInOutCubic.ease(0.0, 1.0, (self.counter / self.duration) as f32);
        }
    }
}

impl Poll for Tile {
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool> {
        match &self.texture_bytes {
            Some(_image_bytes) => Ok(true),
            None => {
                if let PollTask::Ready(bytes) = grabber.poll_request(self.details.url.clone())? {
                    self.texture_bytes = Some(bytes?.clone());
                    self.counter = 0.0;
                    self.duration = 1.0;
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
    fn set_child_positions(&mut self) {
        let position = self.absolute_position();
        if let Some(fallback_text) = &mut self.fallback_text {
            fallback_text.set_parent_position(&position);
        }
    }
}

impl Input for Tile {
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

impl Draw for Tile {
    fn set_render_details(&mut self, renderer: &mut Renderer) {
        let focused_instance = self.focused_instance();

        match (&self.sprite, &self.texture_bytes, &mut self.fallback_text) {
            (Some(sprite), _, fallback_text) => {
                renderer
                    .sprite_pass
                    .set_sprite_instance(*sprite, focused_instance);

                if let Some(fallback_text) = fallback_text {
                    fallback_text.set_render_details(renderer);
                }
            }
            (None, Some(texture_bytes), _) => {
                let texture = match Texture::from_bytes(
                    &renderer.context().device(),
                    &renderer.context().queue(),
                    texture_bytes.as_bytes(),
                    "test.jpeg",
                ) {
                    Ok(texture) => texture,
                    Err(err) => {
                        eprintln!("failed to fetch texture, err: {:?}", err);
                        let mut text = Text::new(self.title.clone());
                        text.set_position(&Vec3::new(-0.5 * SCALE, 0.0, 1.0)); // Arbitrary Z value but just so it goes over focused tile.

                        self.fallback_text = Some(text);
                        renderer.sprite_pass.fallback_texture(renderer.context())
                    }
                };

                let Renderer {
                    sprite_pass,
                    context,
                    ..
                } = renderer;
                let image_handle = sprite_pass.add_texture(context.device(), texture);
                let instance_handle = sprite_pass.add_instance(focused_instance);

                self.sprite = Some(sprite_pass.add_sprite(image_handle, instance_handle));
            }
            _ => {}
        }
    }
}

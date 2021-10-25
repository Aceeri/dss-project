use anyhow::Result;
use glam::Vec2;
use image::EncodableLayout;
use std::task::Poll;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{
    grabber::HttpGrabber,
    home::ImageDetails,
    menu::prelude::*,
    renderer::{SpriteId, SpriteInstance, Renderer, Texture},
};

#[derive(Debug, Clone)]
pub struct Tile {
    position: Position,
    size: Vec2,
    focused: bool,

    sprite: Option<SpriteId>,
    texture_bytes: Option<bytes::Bytes>,
    details: ImageDetails,
}

impl Tile {
    pub fn new(details: ImageDetails) -> Self {
        Self {
            position: Position::new(),
            size: Vec2::new(0.2, 0.2),
            focused: false,

            sprite: None,
            texture_bytes: None,
            details: details,
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
        }
    }
}

impl Pollable for Tile {
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool> {
        match &self.texture_bytes {
            Some(_image_bytes) => Ok(true),
            None => {
                if let Poll::Ready(bytes) = grabber.poll_request(self.details.url.clone())? {
                    self.texture_bytes = Some(bytes?.clone());
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
        match (&self.sprite, &self.texture_bytes) {
            (Some(sprite), _) => {
                renderer.sprite_pass.set_sprite_instance(*sprite, self.focused_instance());
            }
            (None, Some(texture_bytes)) => {
                let texture = match Texture::from_bytes(
                    &renderer.context().device(),
                    &renderer.context().queue(),
                    texture_bytes.as_bytes(),
                    "test.jpeg",
                ) {
                    Ok(texture) => texture,
                    Err(_) => {
                        let fallback_bytes = include_bytes!("../renderer/test.png");
                        Texture::from_bytes(
                            &renderer.context().device(),
                            &renderer.context().queue(),
                            fallback_bytes,
                            "fallback.png",
                        )
                        .expect("created texture")
                    }
                };

                let Renderer {
                    sprite_pass,
                    context,
                    ..
                } = renderer;
                let image_handle = sprite_pass.add_texture(context.device(), texture);
                let instance_handle = sprite_pass.add_instance(self.focused_instance());

                self.sprite = Some(sprite_pass.add_sprite(image_handle, instance_handle));
            }
            _ => {}
        }
    }
}

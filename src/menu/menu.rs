use anyhow::Result;
use glam::{ Vec3};
use image::EncodableLayout;
use std::task::Poll as PollTask;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{grabber::HttpGrabber, home::Home, renderer::Renderer};

use super::{prelude::*, Container};

pub static HOME_URL: &'static str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
pub const COLLECTION_SPACING: f32 = 0.75 * SCALE;

#[derive(Debug, Clone)]
pub struct Menu {
    position: InterpPosition,

    // Vertical list of containers, each container being a group of tiles.
    containers: Vec<Container>,
    focused_container: usize,

    // Indices of containers and tiles to iterate through slowly for rendering.
    partial_container: usize,
    partial_tile: usize,

    // List of tiles that need to be re-rendered immediately.
    dirty_list: Vec<usize>,

    home_loaded: bool,
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            position: InterpPosition::new(),
            containers: Vec::new(),
            focused_container: 0,

            partial_container: 0,
            partial_tile: 0,
            dirty_list: Vec::new(),

            home_loaded: false,
        }
    }

    pub fn push_container(&mut self, mut container: Container) {
        container.set_parent_position(&self.absolute_position());
        container.set_position(&Vec3::new(
            0.5 * SCALE,
            (SCALE + COLLECTION_SPACING) * self.containers.len() as f32,
            0.0,
        ));
        self.containers.push(container);
    }

    pub fn construct_home(&mut self, home: &Home) {
        let mut new_containers = Vec::new();

        for container_ref in &home.data.collection().containers {
            let text_details = container_ref.set.text.title.full.details();
            let mut container =
                Container::new(text_details.content.clone(), container_ref.set.ref_id);

            if let Some(items) = &container_ref.set.items {
                container.add_items(items);
            }

            new_containers.push(container);
        }

        for new_container in new_containers {
            self.push_container(new_container);
        }

        self.home_loaded = true;
    }

    pub fn focus_container(&mut self, container_index: usize) {
        if let Some(container) = self.containers.get_mut(self.focused_container) {
            container.focus(false);
        }

        self.focused_container = container_index;

        if let Some(container) = self.containers.get_mut(self.focused_container) {
            container.focus(true);
        }
    }
}

impl UpdateDelta for Menu {
    fn update_delta(&mut self, delta: f64) {
        for container in &mut self.containers {
            container.update_delta(delta);
        }

        self.position.update(delta);
        self.set_child_positions();
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
        for container in &mut self.containers {
            container.set_parent_position(&absolute);
        }
    }
    fn set_position(&mut self, local_position: &Vec3) {
        self.position.set_position(local_position);
        self.set_child_positions();
    }
}

impl Input for Menu {
    fn input(&mut self, event: &WindowEvent) -> bool {
        // Take up/down requests so we cycle through containers.
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
                match self.containers.get(self.focused_container) {
                    Some(container) => {
                        println!("{:?}", container.absolute_position());
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
                self.position.interp_position(new_position, 1.0);
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
                            ),
                        ..
                    },
                ..
            } => {
                let mut new_focused_container = self.focused_container;

                match direction {
                    VirtualKeyCode::Up => {
                        new_focused_container = new_focused_container.saturating_sub(1)
                    }
                    VirtualKeyCode::Down => {
                        new_focused_container = new_focused_container.saturating_add(1)
                    }
                    _ => {}
                };

                if self.containers.len() > 0 {
                    if new_focused_container > self.containers.len() - 1 {
                        new_focused_container = self.containers.len() - 1;
                    }
                } else {
                    new_focused_container = 0;
                }

                self.focus_container(new_focused_container);
                if let Some(_) = self.containers.get(new_focused_container) {
                    let mut position = self.position.wanted_position();
                    position.y = new_focused_container as f32 * (SCALE + COLLECTION_SPACING) * -1.0;
                    self.position.interp_position(position, 0.75);
                }

                return true;
            }
            _ => {}
        }

        if let Some(container) = self.containers.get_mut(self.focused_container) {
            container.input(event)
        } else {
            false
        }
    }
}

impl Poll for Menu {
    fn poll(&mut self, grabber: &mut HttpGrabber) -> Result<bool> {
        if self.home_loaded {
            let mut done = true;
            for container in &mut self.containers {
                done = done && container.poll(grabber)?;
            }

            Ok(done)
        } else {
            match grabber.poll_request(HOME_URL.to_owned())? {
                PollTask::Pending => Ok(false),
                PollTask::Ready(home) => {
                    println!("got homepage, rendering page now");
                    // Construct initial homepage.
                    let home = home?;
                    let home = serde_json::from_slice(home.as_bytes())?;
                    self.construct_home(&home);
                    Ok(false)
                }
            }
        }
    }
}

impl Draw for Menu {
    fn set_render_details(&mut self, renderer: &mut Renderer) {
        for container in &mut self.containers {
            container.set_render_details(renderer);
        }
    }

    fn partial_set_render_details(&mut self, renderer: &mut Renderer) {
        if let Some(container) = self.containers.get_mut(self.partial_container) {
            container.partial_set_render_details(renderer);

            if let Some(tile) = container.tiles.get_mut(self.partial_tile) {
                tile.set_render_details(renderer);
                self.partial_tile += 1;
            }

            if self.partial_tile >= container.tiles.len() {
                self.partial_tile = 0;
                self.partial_container += 1;

                if self.partial_container >= self.containers.len() {
                    self.partial_container = 0;
                }
            }
        } else {
            self.partial_tile = 0;
            self.partial_container = 0;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::home::ImageDetails;
    use crate::menu::{Container, Menu, PositionHierarchy, Tile};
    use glam::Vec3;

    #[test]
    fn hierarchy_test() {
        let dummy_details: ImageDetails = ImageDetails {
            master_width: 0,
            master_height: 0,
            url: "dummy".to_owned(),
        };

        let mut menu = Menu::new();

        let mut container = Container::new("dummy".to_owned(), None);
        let tile = Tile::new("dummy".to_owned(), dummy_details.clone());
        container.push_tile(tile);

        menu.push_container(container);

        assert_eq!(menu.absolute_position(), Vec3::ZERO);
        assert_eq!(menu.containers[0].absolute_position(), Vec3::ZERO);
        assert_eq!(menu.containers[0].tiles[0].absolute_position(), Vec3::ZERO);

        let vec_10_10 = Vec3::new(10.0, 10.0, 10.0);
        menu.set_position(&vec_10_10);

        assert_eq!(menu.absolute_position(), vec_10_10);
        assert_eq!(menu.containers[0].absolute_position(), vec_10_10);
        assert_eq!(menu.containers[0].tiles[0].absolute_position(), vec_10_10);

        menu.containers[0].tiles[0].set_position(&vec_10_10);
        assert_eq!(
            menu.containers[0].tiles[0].absolute_position(),
            Vec3::new(20.0, 20.0, 20.0)
        );

        menu.containers[0].set_position(&vec_10_10);
        assert_eq!(
            menu.containers[0].absolute_position(),
            Vec3::new(20.0, 20.0, 20.0)
        );
        assert_eq!(
            menu.containers[0].tiles[0].absolute_position(),
            Vec3::new(30.0, 30.0, 30.0)
        );

        let mut new_container = Container::new("dummy".to_owned(), None);
        let new_tile = Tile::new("dummy".to_owned(), dummy_details);
        new_container.push_tile(new_tile);
        menu.push_container(new_container);
        println!("{:?}", menu.absolute_position());
        println!("{:?}", menu.containers[1].absolute_position());
        println!("{:?}", menu.containers[1].tiles[0].absolute_position());
    }
}

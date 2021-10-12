
pub use anyhow::Result;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use bytes::Bytes;

pub use crate::{
    renderer::Renderer,
    http_grabber::HttpGrabber,
    menu::{Menu, EventGrab, Collection, Tile},
    home::Home,
    image::EncodableLayout,
};

use std::{
    collections::HashMap;
    task::Poll,
};

pub static HOME_URL: &'static str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";
pub static ASPECT_RATIO: &'static str = "1.78";

pub struct App {
    renderer: Renderer,
    menu: Menu,
    http_grabber: HttpGrabber,
    event_loop: EventLoop<()>,
    window: Window,
}

impl App {
    pub async fn new() -> Result<App> {
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new()
            .with_min_inner_size(LogicalSize::new(400.0, 300.0))
            .with_inner_size(PhysicalSize::new(400.0, 300.0))
            .with_title("DSS Project".to_string());

        let window = window_builder.build(&event_loop).unwrap();
        let mut menu = Menu::new();
        let mut renderer = Renderer::new(&window).await?;
        let mut http_grabber = HttpGrabber::new();

            Ok(App {
                event_loop,
                renderer,
                menu,
                window,
                http_grabber,
        })
    }

    /*
    // Grab home page of API.
    pub fn poll_home(&mut self) -> Result<()> {
        if let Poll::Ready(bytes) = self.http_grabber.poll(HOME_URL.to_owned())? {
            self.home = Some(serde_json::from_slice(bytes.as_bytes())?);
        }

        Ok(())
    }

    // Just simple grab of all the images, ideally this would be somewhere in a kind of MVC situation with collections and tiles managing their own images/polling
    // But don't have a lot of time so this'll do.
    pub fn poll_images(&mut self) -> Result<bool> {
        let still_polling = false;

        if let Some(home) = self.home {
            for container in home.data.standard_collection.containers {
                if let Some(items) = container.set.items {
                    for item in items {
                        if let Some(image_refs) = item.image.tile.get(ASPECT_RATIO) {
                            if let Some(series) = image_refs.series {
                                if let Some(default_image) = series.get("default") {
                                    let found = self.image_cache.contains_key(&default_image.url);
                                    if !found {
                                        still_polling = true;

                                        if let Poll::Ready(bytes) = self.http_grabber.poll(default_image.url)? {
                                            self.image_cache.insert(default_image.url, bytes);
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(still_polling)
    }
    */

    pub fn run(&mut self) -> Result<()> {
        use crate::menu::Pollable;

        let App {
            event_loop,
            window,
            renderer,
            menu,
            http_grabber,
        } = self;

        event_loop.run(move |event, event_loop_window_target, control_flow| {
            *control_flow = ControlFlow::Wait;

            menu.poll(&*http_grabber);

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
                    if !self.renderer.input(event) && !self.menu.input(event) {
                        // give renderer and menu priority over events
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => *control_flow = ControlFlow::Exit,

                            WindowEvent::KeyboardInput {
                                input: KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::F11),
                                    ..
                                },
                                ..
                            } => {

                                if self.window.fullscreen().is_some() {
                                    self.window.set_fullscreen(None);
                                } else {
                                    if let Some(monitor) = event_loop_window_target.primary_monitor() {
                                        if let Some(video_mode) = monitor.video_modes().next() {
                                            self.window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(video_mode)));
                                        }
                                    }
                                }
                            }

                            WindowEvent::Resized(physical_size) => {
                                self.renderer.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                self.renderer.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(_) => {
                    self.renderer.update();
                    match self.renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => self.renderer.resize(self.renderer.size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    // Maybe we should be conservative with this?
                    // As in only drawing when user is more actively using the program.
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}
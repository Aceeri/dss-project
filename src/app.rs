
pub use anyhow::Result;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use bytes::Bytes;
use glam::{Vec2, Vec3};

pub use crate::{
    renderer::Renderer,
    grabber::HttpGrabber,
    menu::{Menu, EventGrab, Collection, PositionHierarchy, Tile},
    home::Home,
    image::EncodableLayout,
};

use std::{
    collections::HashMap,
    task::Poll,
};

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
            .with_min_inner_size(LogicalSize::new(800.0, 600.0))
            .with_inner_size(PhysicalSize::new(800.0, 600.0))
            .with_title("DSS Project".to_string());

        let window = window_builder.build(&event_loop).unwrap();
        let mut menu = Menu::new();
        menu.set_position(&Vec3::new(1.2, 1.0, 0.0));

        let renderer = Renderer::new(&window).await?;
        let http_grabber = HttpGrabber::new();

        Ok(App {
            event_loop,
            renderer,
            menu,
            window,
            http_grabber,
        })
    }

    pub fn run(self) -> Result<()> {
        use crate::menu::{Pollable, SetRenderDetails};

        let App {
            event_loop,
            window,
            mut renderer,
            mut menu,
            mut http_grabber,
        } = self;

        let mut done_polling = false;

        event_loop.run(move |event, event_loop_window_target, control_flow| {
            *control_flow = ControlFlow::Wait;

            if !done_polling {
                done_polling = menu.poll(&mut http_grabber).expect("polling failed");
            }

            //menu.partial_set_render_details(&mut renderer);
            menu.set_render_details(&mut renderer);

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    let renderer_consumed = renderer.input(event);
                    let menu_consumed = menu.input(event);

                    if menu_consumed {
                        menu.set_render_details(&mut renderer);
                    }

                    if !renderer_consumed && !menu_consumed {
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
                                if window.fullscreen().is_some() {
                                    window.set_fullscreen(None);
                                } else {
                                    if let Some(monitor) = event_loop_window_target.primary_monitor() {
                                        if let Some(video_mode) = monitor.video_modes().next() {
                                            window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(video_mode)));
                                        }
                                    }
                                }
                            }

                            WindowEvent::Resized(physical_size) => {
                                renderer.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                renderer.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(_) => {
                    renderer.update();
                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::RedrawEventsCleared => {
                    // Maybe we should be conservative with this?
                    window.request_redraw();
                }
                _ => {}
            }
        });
    }
}
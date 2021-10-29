pub use anyhow::Result;
use glam::Vec3;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use std::time::Instant;

pub use crate::{
    grabber::HttpGrabber,
    home::Home,
    image::EncodableLayout,
    menu::{Container, UpdateDelta, Draw, Input, Menu, Poll, PositionHierarchy, Tile},
    renderer::Renderer,
};

const WANTED_SIZE: PhysicalSize<u32> = PhysicalSize::new(1920, 1080);

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
            .with_min_inner_size(LogicalSize::new(50.0, 50.0))
            .with_inner_size(WANTED_SIZE)
            .with_title("DSS Project".to_string());

        let window = window_builder.build(&event_loop).unwrap();
        let renderer = Renderer::new(&window).await?;

        let mut menu = Menu::new();
        menu.set_position(&Vec3::new(0.0, 0.0, 0.0));

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
        let App {
            event_loop,
            window,
            mut renderer,
            mut menu,
            mut http_grabber,
        } = self;

        let mut done_polling = false;

        let mut previous_instant = Instant::now();
        let mut delta_accumulate = 0.0;

        event_loop.run(move |event, event_loop_window_target, control_flow| {
            *control_flow = ControlFlow::Poll;

            let now = Instant::now();
            let delta: f64 = match now.checked_duration_since(previous_instant) {
                Some(duration) => duration.as_nanos() as f64 / 1_000_000_000.0f64,
                None => 0.0f64,
            };
            delta_accumulate += delta;
            previous_instant = now;

            if !done_polling {
                done_polling = match menu.poll(&mut http_grabber) {
                    Ok(done) => done,
                    Err(err) => {
                        eprintln!("polling failed: {:?}", err);
                        false
                    }
                }
            }

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
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::F11),
                                        ..
                                    },
                                ..
                            } => {
                                if window.fullscreen().is_some() {
                                    window.set_fullscreen(None);
                                } else {
                                    if let Some(monitor) =
                                        event_loop_window_target.primary_monitor()
                                    {
                                        let mut modes = monitor.video_modes().collect::<Vec<_>>();

                                        modes.sort_by(|a, b| a.cmp(&b));
                                        let mut video_mode = modes.get(0).cloned();

                                        modes = modes
                                            .into_iter()
                                            .filter(|mode| mode.size() == WANTED_SIZE)
                                            .collect::<Vec<_>>();
                                        if let Some(wanted_mode) = modes.get(0) {
                                            video_mode = Some(wanted_mode.clone())
                                        }

                                        if let Some(video_mode) = video_mode {
                                            window.set_fullscreen(Some(
                                                winit::window::Fullscreen::Exclusive(video_mode),
                                            ));
                                        }
                                    }
                                }
                            }

                            WindowEvent::Resized(physical_size) => {
                                renderer.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged {
                                new_inner_size,
                                scale_factor,
                            } => {
                                renderer.resize(**new_inner_size);
                                renderer.set_scale_factor(*scale_factor);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(_) => {
                    if delta_accumulate > 1.0 / 144.0 { // Update at a fixed rate.
                        menu.update_delta(delta_accumulate);
                        delta_accumulate = 0.0;
                    }

                    menu.set_render_details(&mut renderer);

                    if let Err(err) = renderer.update() {
                        eprintln!("update error: {:?}", err);
                    }

                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()),
                        Err(wgpu::SurfaceError::Outdated) => {}
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

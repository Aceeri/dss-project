
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

use dss::menu::{Menu, EventGrab, Collection, Tile};
use dss::renderer::{Renderer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(400.0, 300.0))
        .with_inner_size(PhysicalSize::new(400.0, 300.0))
        .with_title("DSS Project".to_string());

    let window = window_builder.build(&event_loop).unwrap();
    let mut menu = Menu::new();
    {
        let mut collection = Collection::new();
        collection.push_tile(Tile::new());
        collection.push_tile(Tile::new());
        collection.push_tile(Tile::new());
        menu.push_collection(collection);
    }

    {
        let mut collection = Collection::new();
        collection.push_tile(Tile::new());
        menu.push_collection(collection);
    }

    {
        let mut collection = Collection::new();
        collection.push_tile(Tile::new());
        collection.push_tile(Tile::new());
        collection.push_tile(Tile::new());
        collection.push_tile(Tile::new());
        collection.push_tile(Tile::new());
        menu.push_collection(collection);
    }

    let mut renderer = Renderer::new(&window).await?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !renderer.input(event) && !menu.input(event) {
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
            Event::MainEventsCleared => {
                // Maybe we should be conservative with this?
                // As in only drawing when user is more actively using the program.
                window.request_redraw();
            }
            _ => {}
        }
    });

    Ok(())
}

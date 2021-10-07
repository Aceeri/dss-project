
use winit::{
    window::{ Window, WindowBuilder },
    event_loop::{EventLoop, ControlFlow},
    dpi::{LogicalSize, PhysicalSize},
};


fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(64.0, 64.0))
        .with_inner_size(PhysicalSize::new(64.0, 64.0))
        .with_title("DSS Project".to_string());
    
    let window = window_builder.build(&event_loop).unwrap();

    event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                winit::event::Event::WindowEvent {
                    event,
                    window_id
                } if window_id == window.id() => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    winit::event::WindowEvent::Resized(dims) => {
                        println!("resized to {:?}", dims);
                    }
                    _ => {}
                },
                winit::event::Event::RedrawEventsCleared => { }
                _ => {}
            }
        });
}

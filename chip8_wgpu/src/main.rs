use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod renderer;
use renderer::Renderer;

fn main() {
    env_logger::init();
    let path = std::env::args().nth(1).expect("No ROM path is provided.");
    let mut chip8 = chip8::Chip8::new();
    chip8.load(path).unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut renderer = Renderer::new(&window).unwrap();
    

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } => {
            if window_id == window.id() {
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
                        WindowEvent::Resized(new_size) => {
                            renderer.resize(Some(new_size));
                        },
                        WindowEvent::ScaleFactorChanged {new_inner_size, ..} => {
                            renderer.resize(Some(*new_inner_size));
                        },
                    _ => {}
                }
            }
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            log::trace!("redraw");
            chip8.cycle();
            chip8.timer();
            match renderer.render(&chip8.display) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.resize(None),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        },
        Event::MainEventsCleared => {
            log::trace!("redraw_requested");
            window.request_redraw();
        },
        _ => {}
    });
}

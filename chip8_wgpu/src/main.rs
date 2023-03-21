use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::time::{Instant, Duration};
mod renderer;
use renderer::Renderer;


const CHIP8_FREQ: f32 = 800.0;
const TIMER_FREQ: f32 = 60.0;

fn main() {
    env_logger::init();
    let path = std::env::args().nth(1).expect("No ROM path is provided.");
    let mut chip8 = chip8::Chip8::new();
    chip8.load(path).unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut renderer = Renderer::new(&window).unwrap();
 
    let start_time = Instant::now();
    let mut cpu_timer = start_time;
    let mut timer = start_time;


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
        },
        Event::MainEventsCleared => {
            let current_time = Instant::now();
            if current_time.duration_since(cpu_timer) 
                > Duration::from_nanos((1.0 / CHIP8_FREQ * 10_f32.powi(9)) as u64) {
                cpu_timer = current_time;
                chip8.cycle();
            }

            if current_time.duration_since(timer)
                >= Duration::from_nanos((1.0 / TIMER_FREQ * 10_f32.powi(9)) as u64)
                {
                log::trace!("FPS: {}", 1.0/(current_time.duration_since(timer).as_secs_f64()));
                    timer = current_time;
                    chip8.timer();
                    match renderer.render(&chip8.display) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(None),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            std::thread::sleep(Duration::from_nanos(1_300_000));
        },
        _ => {}
    });
}

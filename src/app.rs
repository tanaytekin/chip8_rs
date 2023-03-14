use std::sync::mpsc::Receiver;
use std::ffi::CString;
use std::time::{Instant, Duration};
use std::thread::sleep;

use glfw::{Glfw, Action, Context, Key, WindowEvent, WindowHint, OpenGlProfileHint, WindowMode, Window};

use crate::chip8;
use crate::chip8::Chip8;
use crate::gl;
use crate::gl::types::*;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const TITLE: &'static str = "chip8_rs";

const CHIP8_FREQ: f32 = 800.0;
const TIMER_FREQ: f32 = 60.0;


pub struct App {
    window: Window,
    events: Receiver<(f64, WindowEvent)>,
    glfw: Glfw,
    chip8: Chip8,
    pixels: [u32; chip8::DISPLAY_WIDTH * chip8::DISPLAY_HEIGHT],
    gl_context: GlContext,
    _start_time: Instant,
    cpu_timer: Instant,
    timer: Instant,
    frame_count: u64,
}

impl App {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to init GLFW.");

        glfw.window_hint(WindowHint::ContextVersion(3, 3));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::Resizable(true));

        let (mut window, events) = glfw
            .create_window(WIDTH, HEIGHT, TITLE, WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.make_current();

        glfw.set_swap_interval(glfw::SwapInterval::Sync(0));

        gl::load_with(|s| glfw.get_proc_address_raw(s));


        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
        }

        App {
            window,
            events,
            glfw,
            chip8: Chip8::new(),
            pixels: [0; chip8::DISPLAY_WIDTH * chip8::DISPLAY_HEIGHT],
            gl_context: GlContext::new(),
            _start_time: Instant::now(),
            cpu_timer: Instant::now(),
            timer: Instant::now(),
            frame_count: 0,
        }
    }
    
    pub fn run(&mut self) {
        let path = std::env::args().nth(1).expect("No ROM path is provided.");
        self.chip8.load(path).unwrap();
        while !self.window.should_close() {
            let current_time = Instant::now();


            self.glfw.poll_events();

            for (_, event) in glfw::flush_messages(&self.events) {
                match event {
                    WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        self.window.set_should_close(true);
                    },
                    WindowEvent::FramebufferSize(width, height) => {
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                    },
                    WindowEvent::Key(key, _, Action::Press, _) => {
                        match key {
                            Key::Kp1 => self.chip8.keys[0x1] = true,
                            Key::Kp2 => self.chip8.keys[0x2] = true,
                            Key::Kp3 => self.chip8.keys[0x3] = true,
                            Key::Kp4 => self.chip8.keys[0xC] = true,
                            Key::Q   => self.chip8.keys[0x4] = true,
                            Key::W   => self.chip8.keys[0x5] = true,
                            Key::E   => self.chip8.keys[0x6] = true,
                            Key::R   => self.chip8.keys[0xD] = true,
                            Key::A   => self.chip8.keys[0x7] = true,
                            Key::S   => self.chip8.keys[0x8] = true,
                            Key::D   => self.chip8.keys[0x9] = true,
                            Key::F   => self.chip8.keys[0xE] = true,
                            Key::Z   => self.chip8.keys[0xA] = true,
                            Key::X   => self.chip8.keys[0x0] = true,
                            Key::C   => self.chip8.keys[0xB] = true,
                            Key::V   => self.chip8.keys[0xF] = true,
                            _ => {}
                        }
                    },
                    WindowEvent::Key(key, _, Action::Release, _) => {
                        match key {
                            Key::Kp1 => self.chip8.keys[0x1] = false,
                            Key::Kp2 => self.chip8.keys[0x2] = false,
                            Key::Kp3 => self.chip8.keys[0x3] = false,
                            Key::Kp4 => self.chip8.keys[0xC] = false,
                            Key::Q   => self.chip8.keys[0x4] = false,
                            Key::W   => self.chip8.keys[0x5] = false,
                            Key::E   => self.chip8.keys[0x6] = false,
                            Key::R   => self.chip8.keys[0xD] = false,
                            Key::A   => self.chip8.keys[0x7] = false,
                            Key::S   => self.chip8.keys[0x8] = false,
                            Key::D   => self.chip8.keys[0x9] = false,
                            Key::F   => self.chip8.keys[0xE] = false,
                            Key::Z   => self.chip8.keys[0xA] = false,
                            Key::X   => self.chip8.keys[0x0] = false,
                            Key::C   => self.chip8.keys[0xB] = false,
                            Key::V   => self.chip8.keys[0xF] = false,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            if current_time.duration_since(self.cpu_timer) > Duration::from_nanos((1.0/CHIP8_FREQ * 10_f32.powi(9)) as u64) {
                self.cpu_timer = current_time;
                self.chip8.cycle();
            }

            if current_time.duration_since(self.timer) >= Duration::from_nanos((1.0/TIMER_FREQ * 10_f32.powi(9)) as u64) {
                self.timer = current_time;
                self.chip8.timer();
                self.update_texture(0xFF00FF00, 0);
                self.render();
                self.window.swap_buffers();
                self.frame_count += 1;
                //println!("FPS: {}", (self.frame_count as f64) / (current_time.duration_since(self.start_time).as_secs_f64()));
            }

            sleep(Duration::from_nanos(1_500_000));
        }
    }

    pub fn render(&mut self) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            self.gl_context.draw();
        }
    }


    fn update_texture(&mut self, f_color: u32, b_color: u32) {
        for i in 0..(chip8::DISPLAY_WIDTH * chip8::DISPLAY_HEIGHT) {
            if self.chip8.display[i] {
                self.pixels[i] = f_color;
            } else {
                self.pixels[i] = b_color;
            }
        }
 
        unsafe {
            gl::TexSubImage2D(gl::TEXTURE_2D,
                              0,
                              0,
                              0,
                              chip8::DISPLAY_WIDTH as GLsizei,
                              chip8::DISPLAY_HEIGHT as GLsizei,
                              gl::RGBA,
                              gl::UNSIGNED_BYTE,
                              self.pixels.as_ptr() as *const GLvoid,
                              );
        }

    }
}


struct GlContext {
    shader_program: GLuint,
    texture: GLuint,
    vao: GLuint,
}

impl GlContext {
    fn new() -> Self {
        let mut vao = 0;
        let mut vbo = 0;

        let vertices: Vec<f32> = vec![
            -1.0, -1.0, 0.0, 1.0,
             1.0, -1.0, 1.0, 1.0,
             1.0,  1.0, 1.0, 0.0,
             
             1.0,  1.0, 1.0, 0.0,
            -1.0,  1.0, 0.0, 0.0,
            -1.0, -1.0, 0.0, 1.0,
        ];
        
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr, vertices.as_ptr() as *const GLvoid, gl::STATIC_DRAW);

            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, (4 * std::mem::size_of::<f32>()) as GLsizei, 0 as *const GLvoid);
            gl::EnableVertexAttribArray(0);

        }

        GlContext{
            shader_program: Self::load_shader_program(),
            texture: Self::create_texture(),
            vao,
        }
    }
    fn create_texture() -> u32 {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           gl::RGBA as GLint,
                           chip8::DISPLAY_WIDTH as GLint,
                           chip8::DISPLAY_HEIGHT as GLint,
                           0,
                           gl::RGBA,
                           gl::UNSIGNED_BYTE,
                           std::ptr::null());
        }
        texture
    }

    fn compile_shader(source: &str, shader_type: GLenum) ->  GLuint{
        let source = CString::new(source).unwrap();
        let shader = unsafe{ gl::CreateShader(shader_type)  };
        
        unsafe {
            gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);
        }
        
        let mut success = gl::FALSE as gl::types::GLint;
        
        unsafe {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        }
        
        if success == gl::FALSE as GLint {
            let mut len = 0 as GLint;
            unsafe {
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            }
            let buffer = vec![0; len as usize];
            unsafe {
                let info_log_c_string = CString::from_vec_with_nul_unchecked(buffer);
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    std::ptr::null_mut(),
                    info_log_c_string.as_ptr() as *mut GLchar,
                );
                gl::DeleteShader(shader);
                panic!(
                    "Shader Compilation Error:\n{}",
                    info_log_c_string.to_str().unwrap()
                );
            }
        }
        shader
    }


    fn load_shader_program() -> GLuint {
        let vertex_source =
            r#"
            #version 330 core
            layout (location = 0) in vec4 a_vertex;
            out vec2 v_tex_coords;
            void main()
            {
                v_tex_coords = a_vertex.zw;
                gl_Position = vec4(a_vertex.xy, 0.0, 1.0);
            }
        "#;

        let fragment_source =
            r#"
            #version 330 core
            in vec2 v_tex_coords;
            out vec4 o_color;
            uniform sampler2D tex;
            void main()
            {
                o_color = texture(tex, v_tex_coords);
            }
        "#;

        let vertex = Self::compile_shader(vertex_source, gl::VERTEX_SHADER);
        let fragment = Self::compile_shader(fragment_source, gl::FRAGMENT_SHADER);

        let program = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(program, vertex);
            gl::AttachShader(program, fragment);
            gl::LinkProgram(program);
        }

        let mut success = gl::FALSE as GLint;
        unsafe {
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        }

        if success == gl::FALSE as GLint {
            let mut len = 0 as GLint;
            unsafe {
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            }
            let buffer = vec![0; len as usize];
            unsafe {
                let info_log_c_string = CString::from_vec_with_nul_unchecked(buffer);
                gl::GetProgramInfoLog(
                    program,
                    len,
                    std::ptr::null_mut(),
                    info_log_c_string.as_ptr() as *mut GLchar,
                    );
                panic!("Shader Program Linking Error:\n{}", info_log_c_string.to_str().unwrap());
            }
        }
        program
    }

    fn draw(&self) {
        unsafe {
            gl::UseProgram(self.shader_program);
            gl::BindTexture(gl::TEXTURE0, self.texture);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

}

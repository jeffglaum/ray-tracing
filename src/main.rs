// https://dev.to/samkevich/learn-opengl-with-rust-creating-a-window-1792

mod shader;
mod vao;
mod vbo;

use crate::shader::{Shader, ShaderProgram};
use crate::vao::VertexArray;
use crate::vbo::Buffer;
use gl;
use glfw::{fail_on_errors, Action, Context, Key};
use std::fs;

const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;

// vertex data type
type Pos = [f32; 2];
type Color = [f32; 3];

#[repr(C, packed)]
struct Vertex(Pos, Color);

// vertex data for triangle
#[rustfmt::skip]
const VERTICES: [Vertex; 3] = [
    Vertex([-0.5, -0.5], [1.0, 0.0, 0.0]),
    Vertex([0.5,  -0.5], [0.0, 1.0, 0.0]),
    Vertex([0.0,   0.5], [0.0, 0.0, 1.0])
];

#[macro_export]
macro_rules! set_attribute {
    ($vbo:ident, $pos:tt, $t:ident :: $field:tt) => {{
        let dummy = core::mem::MaybeUninit::<$t>::uninit();
        let dummy_ptr = dummy.as_ptr();
        let member_ptr = core::ptr::addr_of!((*dummy_ptr).$field);
        const fn size_of_raw<T>(_: *const T) -> usize {
            core::mem::size_of::<T>()
        }
        let member_offset = member_ptr as i32 - dummy_ptr as i32;
        $vbo.set_attribute::<$t>(
            $pos,
            (size_of_raw(member_ptr) / core::mem::size_of::<f32>()) as i32,
            member_offset,
        )
    }};
}

fn main() {
    // initialize GLFW
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    // request core profile and forward compatible context
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 1));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    // create a windowed mode window and its OGL context
    let (mut window, events) = glfw
        .create_window(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Ray Tracing",
            glfw::WindowMode::Windowed,
        )
        .expect("ERROR: Failed to create GLFW window.");

    // get OGL version
    let context_version = window.get_context_version();
    println!(
        "INFO: OGL context version = {0}.{1}",
        context_version.major, context_version.minor
    );

    // make the window's context current
    window.make_current();
    window.set_key_polling(true);

    // load OGL on the GLFW window
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    // load shaders
    // TODO
    let vertex_shader_source = fs::read_to_string("./src/shaders/vertex_shader.glsl").unwrap();
    let fragment_shader_source = fs::read_to_string("./src/shaders/fragment_shader.glsl").unwrap();
    let vertex_shader =
        unsafe { Shader::new(vertex_shader_source.as_str(), gl::VERTEX_SHADER).unwrap() };
    let fragment_shader =
        unsafe { Shader::new(fragment_shader_source.as_str(), gl::FRAGMENT_SHADER).unwrap() };

    let program = unsafe { ShaderProgram::new(&[vertex_shader, fragment_shader]).unwrap() };

    let vertex_buffer = unsafe { Buffer::new(gl::ARRAY_BUFFER) };
    unsafe { vertex_buffer.set_data(&VERTICES, gl::STATIC_DRAW) };
    let vertex_array = unsafe { VertexArray::new() };
    let pos_attrib = unsafe { program.get_attrib_location("position").unwrap() };
    unsafe { set_attribute!(vertex_array, pos_attrib, Vertex::0) };
    let color_attrib = unsafe { program.get_attrib_location("color").unwrap() };
    unsafe { set_attribute!(vertex_array, color_attrib, Vertex::1) };
    unsafe { vertex_array.bind() };

    // loop until the user closes the window
    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            program.apply();
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
        // swap front and back buffers
        window.swap_buffers();

        // poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }
    }
}

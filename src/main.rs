// https://dev.to/samkevich/learn-opengl-with-rust-creating-a-window-1792
// http://www.opengl-tutorial.org/beginners-tutorials/tutorial-1-opening-a-window/

mod shader;
mod vao;
mod vbo;

use crate::shader::{Shader, ShaderProgram};
use crate::vao::VertexArray;
use crate::vbo::Buffer;
use gl;
use glfw::{fail_on_errors, Action, Context, Key};
use glm;
use std::fs;

const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;

// vertex data type
type Pos = [f32; 3];
type Color = [f32; 3];

#[repr(C, packed)]
struct Vertex(Pos, Color);

// vertex data for triangle (positiom, color)
#[rustfmt::skip]
const _VERTICES_TRIANGE: [Vertex; 3] = [
    Vertex([-0.5, -0.5, 0.0], [1.0, 0.0, 0.0]),
    Vertex([0.5,  -0.5, 0.0], [0.0, 1.0, 0.0]),
    Vertex([0.0,   0.5, 0.0], [0.0, 0.0, 1.0])
];

// a cube has 6 faces with 2 triangles each, so this makes 6*2=12 triangles, and 12*3 vertices
#[rustfmt::skip]
const VERTICES_CUBE: [Vertex; 36] = [
    Vertex([-1.0,-1.0,-1.0], [0.583,  0.771,  0.014]),
    Vertex([-1.0,-1.0, 1.0], [0.609,  0.115,  0.436]),
    Vertex([-1.0, 1.0, 1.0], [0.327,  0.483,  0.844]),
    Vertex([1.0, 1.0,-1.0], [0.822,  0.569,  0.201]),
    Vertex([-1.0,-1.0,-1.0], [0.435,  0.602,  0.223]),
    Vertex([-1.0, 1.0,-1.0], [0.310,  0.747,  0.185]),
    Vertex([1.0,-1.0, 1.0], [0.597,  0.770,  0.761]),
    Vertex([-1.0,-1.0,-1.0], [0.559,  0.436,  0.730]),
    Vertex([1.0,-1.0,-1.0], [0.359,  0.583,  0.152]),
    Vertex([1.0, 1.0,-1.0], [0.483,  0.596,  0.789]),
    Vertex([1.0,-1.0,-1.0], [0.559,  0.861,  0.639]),
    Vertex([-1.0,-1.0,-1.0], [0.195,  0.548,  0.859]),
    Vertex([-1.0,-1.0,-1.0], [0.014,  0.184,  0.576]),
    Vertex([-1.0, 1.0, 1.0], [0.771,  0.328,  0.970]),
    Vertex([-1.0, 1.0,-1.0], [0.406,  0.615,  0.116]),
    Vertex([1.0,-1.0, 1.0], [0.676,  0.977,  0.133]),
    Vertex([-1.0,-1.0, 1.0], [0.971,  0.572,  0.833]),
    Vertex([-1.0,-1.0,-1.0], [0.140,  0.616,  0.489]),
    Vertex([-1.0, 1.0, 1.0], [0.997,  0.513,  0.064]),
    Vertex([-1.0,-1.0, 1.0], [0.945,  0.719,  0.592]),
    Vertex([1.0,-1.0, 1.0], [0.543,  0.021,  0.978]),
    Vertex([1.0, 1.0, 1.0], [0.279,  0.317,  0.505]),
    Vertex([1.0,-1.0,-1.0], [0.167,  0.620,  0.077]),
    Vertex([1.0, 1.0,-1.0], [0.347,  0.857,  0.137]),
    Vertex([1.0,-1.0,-1.0], [0.055,  0.953,  0.042]),
    Vertex([1.0, 1.0, 1.0], [0.714,  0.505,  0.345]),
    Vertex([1.0,-1.0, 1.0], [0.783,  0.290,  0.734]),
    Vertex([1.0, 1.0, 1.0], [0.722,  0.645,  0.174]),
    Vertex([1.0, 1.0,-1.0], [0.302,  0.455,  0.848]),
    Vertex([-1.0, 1.0,-1.0], [0.225,  0.587,  0.040]),
    Vertex([1.0, 1.0, 1.0], [0.517,  0.713,  0.338]),
    Vertex([-1.0, 1.0,-1.0], [0.053,  0.959,  0.120]),
    Vertex([-1.0, 1.0, 1.0], [0.393,  0.621,  0.362]),
    Vertex([1.0, 1.0, 1.0], [0.673,  0.211,  0.457]),
    Vertex([-1.0, 1.0, 1.0], [0.820,  0.883,  0.371]),
    Vertex([1.0,-1.0, 1.0], [0.982,  0.099,  0.879])
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
    unsafe { vertex_buffer.set_data(&VERTICES_CUBE, gl::STATIC_DRAW) };
    let vertex_array = unsafe { VertexArray::new() };
    let pos_attrib = unsafe { program.get_attrib_location("position").unwrap() };
    unsafe { set_attribute!(vertex_array, pos_attrib, Vertex::0) };
    let color_attrib = unsafe { program.get_attrib_location("color").unwrap() };
    unsafe { set_attribute!(vertex_array, color_attrib, Vertex::1) };
    unsafe { vertex_array.bind() };

    // construct a projection matrix
    let proj = glm::ext::perspective_rh(
        glm::radians(45.0),
        SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32,
        0.1,
        100.0,
    );

    // construct a camera matrix
    let view = glm::ext::look_at_rh(
        glm::vec3(4.0, 3.0, 3.0),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
    );

    // construct a model matrix (identity matrix for now)
    // TODO better way to init an identity matrix?
    let model = glm::mat4(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    // build the MVP matrix
    let mvp = proj * view * model;

    unsafe { program.apply() };

    // give the mvp matrix to the shader as a uniform
    let mvp_string_ptr = "MVP".as_ptr() as *const i8;
    let matrix_id = unsafe { gl::GetUniformLocation(program.get_id(), mvp_string_ptr) };

    // send our mvp matrix to the currently-bound shader
    unsafe {
        let mvp_ptr: *const f32 = std::mem::transmute(&mvp);
        gl::UniformMatrix4fv(matrix_id, 1, gl::FALSE, mvp_ptr);
    }

    // set up z-buffering
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
    }

    // loop until the user closes the window
    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            program.apply();
            // TODO - get from vertex array length
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
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

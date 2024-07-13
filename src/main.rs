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
use obj::Obj;
use rand::Rng;
use std::fs;

const SCREEN_WIDTH: u32 = 1024;
const SCREEN_HEIGHT: u32 = 768;

// vertex data type
type Pos = [f32; 3];
type Color = [f32; 3];

#[repr(C, packed)]
struct Vertex(Pos, Color);

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

// load shaders
fn load_shader_program() -> ShaderProgram {
    // TODO
    let vertex_shader_source = fs::read_to_string("./src/shaders/vertex_shader.glsl").unwrap();
    let fragment_shader_source = fs::read_to_string("./src/shaders/fragment_shader.glsl").unwrap();
    let vertex_shader =
        unsafe { Shader::new(vertex_shader_source.as_str(), gl::VERTEX_SHADER).unwrap() };
    let fragment_shader =
        unsafe { Shader::new(fragment_shader_source.as_str(), gl::FRAGMENT_SHADER).unwrap() };

    let program = unsafe { ShaderProgram::new(&[vertex_shader, fragment_shader]).unwrap() };

    unsafe { program.apply() };

    // return the program
    program
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

    // create a window and its OGL context
    let (mut window, events) = glfw
        .create_window(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Ray Tracing",
            glfw::WindowMode::Windowed,
        )
        .expect("ERROR: failed to create GLFW window");

    // get OGL version
    let context_version = window.get_context_version();
    println!(
        "INFO: ogl context version = {0}.{1}",
        context_version.major, context_version.minor
    );

    // make the window's context current
    window.make_current();
    window.set_key_polling(true);

    // load OGL on the GLFW window
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    // load and apply the shader program
    let program = load_shader_program();
    unsafe { program.apply() };

    // load the model from an obj file
    // TODO
    let suzanne = Obj::load("./resources/suzanne.obj").unwrap();
    let mut vertices: Vec<Vertex> = vec![];
    for v in suzanne.data.position {
        // push both vertices and a per-vertex color
        let color = [
            rand::thread_rng().gen_range(0..1000) as f32 / 1000.0,
            rand::thread_rng().gen_range(0..1000) as f32 / 1000.0,
            rand::thread_rng().gen_range(0..1000) as f32 / 1000.0,
        ];
        vertices.push(Vertex(v, color));
    }
    let mut indices: Vec<u32> = vec![];
    for o in suzanne.data.objects {
        for g in o.groups {
            for p in g.polys {
                for it in p.0 {
                    indices.push(it.0 as u32);
                }
            }
        }
    }
    let indices_length = indices.len() as i32;

    // create vertex buffer
    let vertex_buffer = unsafe { Buffer::new(gl::ARRAY_BUFFER) };
    unsafe { vertex_buffer.set_data(vertices, gl::STATIC_DRAW) };

    // create a vertex array
    let vertex_array = unsafe { VertexArray::new() };

    // set vertex array attributes for shader usage: position and color
    let pos_attrib = unsafe { program.get_attrib_location("position").unwrap() };
    unsafe { set_attribute!(vertex_array, pos_attrib, Vertex::0) };
    let color_attrib = unsafe { program.get_attrib_location("color").unwrap() };
    unsafe { set_attribute!(vertex_array, color_attrib, Vertex::1) };

    // create index (element) buffer
    let index_buffer = unsafe { Buffer::new(gl::ELEMENT_ARRAY_BUFFER) };
    unsafe { index_buffer.set_data(indices, gl::STATIC_DRAW) };

    // bind the vertex array
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
        glm::vec3(0.0, 0.0, 5.0),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
    );

    // construct a model matrix (identity matrix for now)
    // TODO better way to init an identity matrix?
    let mut model = glm::mat4(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    // build the MVP matrix
    let mvp = proj * view * model;

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

    let mut show_wireframe = true;

    let camera_location = glm::vec3(0.0, 0.0, 5.0);

    // loop until the user closes the window
    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            if show_wireframe {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            } else {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            }

            // construct a camera matrix
            let view = glm::ext::look_at_rh(
                camera_location,
                glm::vec3(0.0, 0.0, 0.0),
                glm::vec3(0.0, 1.0, 0.0),
            );
            let mvp = proj * view * model;
            let mvp_ptr: *const f32 = std::mem::transmute(&mvp);
            gl::UniformMatrix4fv(matrix_id, 1, gl::FALSE, mvp_ptr);

            gl::BindVertexArray(vertex_array.get_id());
            gl::DrawElements(
                gl::TRIANGLES,
                indices_length,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
        // swap front and back buffers
        window.swap_buffers();

        // poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            //println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::Key(Key::Tab, _, Action::Press, _) => {
                    show_wireframe = !show_wireframe;
                }
                glfw::WindowEvent::Key(Key::Equal, _, Action::Press | Action::Repeat, _) => {
                    model = glm::ext::scale(&model, glm::vec3(1.1, 1.1, 1.1));
                }
                glfw::WindowEvent::Key(Key::Minus, _, Action::Press | Action::Repeat, _) => {
                    model = glm::ext::scale(&model, glm::vec3(0.9, 0.9, 0.9));
                }
                glfw::WindowEvent::Key(Key::Left, _, Action::Press | Action::Repeat, _) => {
                    model = glm::ext::rotate(&model, 0.0875, glm::vec3(0.0, 1.0, 0.0));
                }
                glfw::WindowEvent::Key(Key::Right, _, Action::Press | Action::Repeat, _) => {
                    model = glm::ext::rotate(&model, -0.0875, glm::vec3(0.0, 1.0, 0.0));
                }
                glfw::WindowEvent::Key(Key::Up, _, Action::Press | Action::Repeat, _) => {
                    model = glm::ext::rotate(&model, -0.0875, glm::vec3(1.0, 0.0, 0.0));
                }
                glfw::WindowEvent::Key(Key::Down, _, Action::Press | Action::Repeat, _) => {
                    model = glm::ext::rotate(&model, 0.0875, glm::vec3(1.0, 0.0, 0.0));
                }
                _ => {}
            }
        }
    }
}

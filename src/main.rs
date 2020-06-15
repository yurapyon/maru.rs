use std::env;

use cgmath::{
    prelude::*,
    Vector2,
    Matrix4,
    Ortho,
};
use gl;
use gl::types::*;
use glfw;
use glfw::{Action, Context, Key};
use image;

//

use maru::gfx::*;
use maru::math;
use maru::math::ext::*;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::Resizable(true));

    let (mut window, events) = glfw.create_window(600, 400, "float", glfw::WindowMode::Windowed)
        .expect("failed to create window");

    window.make_current();
    window.set_all_polling(true);

    gl::load_with(| s | window.get_proc_address(s) as *const _);
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    unsafe {
        gl::Viewport(0, 0, 600, 400);
    }

    let prog = {
        let vert = Shader::new(gl::VERTEX_SHADER, &["#version 330 core\n", "\
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 uv;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
}"
        ]).unwrap();

        let frag = Shader::new(gl::FRAGMENT_SHADER, &["#version 330 core\n", "\
out vec4 out_color;

void main() {
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
}"
        ]).unwrap();
        Program::new(&[vert, frag])
    }.unwrap();

    println!("{:?}", prog);

    println!("{:?}", env::current_dir());
    let img = image::open("content/mahou.jpg").unwrap().to_rgba();
    let tex = Texture::new(img).unwrap();

    let quad_mesh = Mesh::new(math::Vertices::quad(false),
        gl::STATIC_DRAW,
        gl::TRIANGLE_STRIP).unwrap();

    let u_proj = Uniform::new(
        UniformData::Mat4(Matrix4::from(Ortho::screen(600., 400.))),
        &prog,
        "projection").unwrap();

    let u_view = Uniform::new(
        UniformData::Mat4(Matrix4::identity()),
        &prog,
        "view").unwrap();

    let u_model = Uniform::new(
        UniformData::Mat4(Matrix4::from_transform2d(
                &math::Transform2d {
                    scale: Vector2::from([100., 100.]),
                    .. Default::default()
                })),
        &prog,
        "model").unwrap();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                _ => {},
            }
        }

        unsafe {
            gl::ClearColor(0., 0., 0., 0.);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(prog.gl());
        }

        u_proj.apply();
        u_view.apply();
        u_model.apply();

        quad_mesh.draw();

        window.swap_buffers();
    }
}

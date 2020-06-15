use std::env;

use gl;
use gl::types::*;
use glfw;
use glfw::{Action, Context, Key};
use image;

//

use maru::gfx;

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

    let prog = {
        let vert = gfx::Shader::new(gl::VERTEX_SHADER, &["#version 330\n", "\
in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
        ]).unwrap();

        let frag = gfx::Shader::new(gl::FRAGMENT_SHADER, &["#version 330\n", "\
out vec4 out_color;

void main() {
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
}"
        ]).unwrap();
        gfx::Program::new(&[vert, frag])
    };

    println!("{:?}", prog);

    println!("{:?}", env::current_dir());
    let img = image::open("content/mahou.jpg").unwrap().to_rgba();
    let tex = gfx::Texture::new(img).unwrap();

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
            gl::ClearColor(1.,1.,1.,1.);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        window.swap_buffers();
    }
}

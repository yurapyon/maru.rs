use std::{
    env,
    thread,
    time,
};

use cgmath::{
    prelude::*,
    Vector2,
    Vector4,
    Matrix4,
    Ortho,
};
use gl;
use gl::types::*;
use glfw;
use glfw::{Action, Context, Key};
use image;

//

use maru::{
    content,
    gfx::*,
    math,
    math::ext::*,
};

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
        let vert = Shader::from_template(gl::VERTEX_SHADER,
            &ShaderTemplate::new(content::shaders::DEFAULT_VERT,
                Some(content::shaders::EXTRAS)).unwrap(),
            None,
        ).unwrap();

        let frag = Shader::from_template(gl::FRAGMENT_SHADER,
            &ShaderTemplate::new(content::shaders::DEFAULT_FRAG,
                Some(content::shaders::EXTRAS)).unwrap(),
            Some("
                vec4 effect() {
                    vec4 c = _base_color;
                    c.r = sin(_time) *0.5+1;
                    c.g = cos(_time) *0.5+1;
                    return c * texture2D(_diffuse, _uv_coord);
                }"),
        ).unwrap();

        Program::new(&[vert, frag])
    }.unwrap();

    println!("{:?}", prog);

    println!("{:?}", env::current_dir());
    println!("{:?}", env!("CARGO_MANIFEST_DIR"));
    let img = image::load_from_memory(content::image::MAHOU).unwrap().to_rgba();
    let tex = Texture::new(img).unwrap();

    let td = TextureData {
        select: gl::TEXTURE0,
        bind_to: gl::TEXTURE_2D,
        texture: &tex,
    };
    let tu_mahou = TextureUniform::new(td, &prog, "_diffuse").unwrap();

    let quad_mesh = Mesh::new(math::Vertices::quad(false),
        gl::STATIC_DRAW,
        gl::TRIANGLE_STRIP).unwrap();

    let u_proj = Uniform::new(
        UniformData::Mat4(Matrix4::from(Ortho::screen(600., 400.))),
        &prog,
        "_projection").unwrap();

    let u_view = Uniform::new(
        UniformData::Mat4(Matrix4::identity()),
        &prog,
        "_view").unwrap();

    let u_model = Uniform::new(
        UniformData::Mat4(Matrix4::from_transform2d(
                &math::Transform2d {
                    scale: Vector2::from([400., 300.]),
                    .. Default::default()
                })),
        &prog,
        "_model").unwrap();

    let u_color = Uniform::new(
        UniformData::Vec4(Vector4::from([1., 1., 1., 1.])),
        &prog,
        "_base_color").unwrap();

    let mut u_time = Uniform::new(
        UniformData::Float(0.),
        &prog,
        "_time").unwrap();

    let s_tm = time::Duration::from_millis(30);

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

        *u_time.float().unwrap() += 0.1;

        unsafe {
            gl::ClearColor(0., 0., 0., 0.);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        prog.gl_use();
        u_proj.apply();
        u_view.apply();
        u_model.apply();
        u_color.apply();
        u_time.apply();

        tu_mahou.apply();

        quad_mesh.draw();

        window.swap_buffers();
        thread::sleep(s_tm);
    }
}

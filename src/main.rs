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
        gl::Enable(gl::BLEND);
        gl::BlendEquation(gl::FUNC_ADD);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let prog = Program::new_default(None,
            Some("
                vec4 effect() {
                    vec4 c = _base_color;
                    c.r = sin(_time) *0.5+1;
                    c.g = cos(_time) *0.5+1;
                    return c * texture2D(_tx_diffuse, _uv_coord);
                }")).unwrap();

    println!("{:?}", prog);

    println!("{:?}", env::current_dir());
    println!("{:?}", env!("CARGO_MANIFEST_DIR"));
    let img = image::load_from_memory(content::image::MAHOU).unwrap().to_rgba();
    let tex = Texture::new(&img);

    let quad_mesh = Mesh::new(math::Vertices::quad(false),
        gl::STATIC_DRAW,
        gl::TRIANGLE_STRIP);

    let locs = DefaultLocations::new(&prog);

    let m4_proj = Matrix4::from(Ortho::screen(600., 400.));

    let m4_view = Matrix4::identity();

    let m4_model =
        Matrix4::from_transform2d(
                &math::Transform2d {
                    scale: Vector2::from([100., 100.]),
                    .. Default::default()
                });

    let mut v4_color = Vector4::from([1., 1., 1., 1.]);

    let mut f_time: GLfloat = 0.;

    let td_diffuse = TextureData {
        select: gl::TEXTURE0,
        bind_to: gl::TEXTURE_2D,
        texture: &tex,
    };

    let s_tm = time::Duration::from_millis(30);

    let draw = Drawer::new(50);

    let mut sb = Spritebatch::new(50);
    let sb_prog = Program::new_default_spritebatch().unwrap();
    let sb_locs = DefaultLocations::new(&sb_prog);

    let fn_img = image::load_from_memory(content::image::SMALL_FONT).unwrap().to_rgba();
    let font = BitmapFont::new(&fn_img,
        " ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890[](){}=+-/^$@#*~%_<>\"'?!|\\&`.,:;");
    let fn_tex = font.texture();
    let fn_tex_uni = TextureData::diffuse(fn_tex);

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

        // *u_time.float().unwrap() += 0.1;
        f_time += 0.1;

        unsafe {
            gl::ClearColor(0., 0., 0., 0.);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        /*
        prog.gl_use();
        m4_proj.uniform(locs.projection());
        m4_view.uniform(locs.view());
        // m4_model.uniform(locs.model());
        v4_color.x = 1.;
        v4_color.y = 1.;
        v4_color.z = 1.;
        v4_color.uniform(locs.base_color());
        f_time.uniform(locs.time());
        // td_diffuse.uniform(locs.diffuse());
        // quad_mesh.draw();

        draw.sprite(&locs, &tex, 
                &math::Transform2d {
                    scale: Vector2::from([0.75, 1.]),
                    .. Default::default()
                });

        v4_color.x = 0.;
        v4_color.y = 0.;
        v4_color.z = 0.;
        v4_color.uniform(locs.base_color());
        draw.filled_rectangle(&locs, Vector4::new(10., 10., 50., 50.));
        */

        sb_prog.gl_use();
        m4_proj.uniform(sb_locs.projection());
        m4_view.uniform(sb_locs.view());
        m4_view.uniform(sb_locs.model());
        v4_color.x = 1.;
        v4_color.y = 1.;
        v4_color.z = 1.;
        v4_color.uniform(sb_locs.base_color());
        f_time.uniform(sb_locs.time());
        td_diffuse.uniform(sb_locs.diffuse());

        sb.begin();

        let tmp = sb.pull();
        tmp.color.x = 0.33;
        tmp.transform.position.x = 0.;
        tmp.transform.scale.x = 150.;
        tmp.transform.scale.y = 150.;
        let tmp = sb.pull();
        tmp.color.x = 0.66;
        tmp.transform.position.x = 50.;
        tmp.transform.scale.x = 150.;
        tmp.transform.scale.y = 150.;
        let tmp = sb.pull();
        tmp.color.x = 0.99;
        tmp.transform.position.x = 100.;
        tmp.transform.scale.x = 150.;
        tmp.transform.scale.y = 150.;
        tmp.uv.z = 0.5;

        sb.end();

        fn_tex_uni.uniform(sb_locs.diffuse());
        sb.print(&font, "hello");

        window.swap_buffers();
        thread::sleep(s_tm);
    }
}

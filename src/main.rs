use std::{
    thread,
    time,
};

use gl::{
    self,
};
use glfw::{
    self,
    Action,
    Context,
    Key,
};
use image;
use nalgebra_glm as glm;

//

use maru::{
    content,
    gfx::*,
    glfw::*,
    math::ext::*,
};

fn main() {
    let mut ctx = GlfwContext::new(GlfwContextSettings {
        window_width: 600,
        window_height: 400,
        .. Default::default()
    });

    let mahou_img = image::load_from_memory(content::image::MAHOU).unwrap().to_rgba();
    let mahou_tex = Texture::new(&mahou_img);
    let mahou_td = TextureData::diffuse(&mahou_tex);

    let m3_screen = ortho_screen(glm::vec2(600, 400));
    let mut time = 0.;

    // let draw = ShapeDrawer::new(50);

    let mut sb = Spritebatch::new(50);
    let sb_prog = Program::new_default_spritebatch().unwrap();
    let sb_locs = DefaultLocations::new(&sb_prog);

    let font = BitmapFont::new_default();
    let font_tex = font.texture();
    let font_td = TextureData::diffuse(font_tex);

    let s_tm = time::Duration::from_millis(30);

    while !ctx.window.should_close() {
        ctx.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&ctx.events) {
            // println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    ctx.window.set_should_close(true)
                },
                _ => {},
            }
        }

        time += 0.1;

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        sb_prog.bind();
        sb_locs.reset();
        m3_screen.uniform(sb_locs.screen());
        time.uniform(sb_locs.time());

        mahou_td.uniform(sb_locs.diffuse());

        sb.begin();

        let tmp = sb.pull();
        *tmp = Default::default();
        tmp.color.r = 0.33;
        tmp.transform.position.x = 0.;
        tmp.transform.scale.x = 150.;
        tmp.transform.scale.y = 150.;
        let tmp = sb.pull();
        *tmp = Default::default();
        tmp.color.r = 0.66;
        tmp.transform.position.x = 50.;
        tmp.transform.scale.x = 150.;
        tmp.transform.scale.y = 150.;
        let tmp = sb.pull();
        *tmp = Default::default();
        tmp.color.r = 0.99;
        tmp.transform.position.x = 100.;
        tmp.transform.scale.x = 150.;
        tmp.transform.scale.y = 150.;
        tmp.uv.corner2.x = 0.5;

        sb.end();

        font_td.uniform(sb_locs.diffuse());
        sb.print(&font, "hello world scl by 2");

        ctx.window.swap_buffers();
        thread::sleep(s_tm);
    }
}

use gl::{
    self,
};
use nalgebra_glm as glm;
use sdl2::{
    keyboard::Keycode,
    event::Event,
};

//

use maru::{
    sdl::{
        Context,
        ContextSettings,
    },
    gfx::*,
    math::ext::*,
    defaults::two_d::{
        self,
        Spritebatch,
        BitmapFont,
        DefaultLocations,
        ShapeDrawer,
    },
    particles::*,
    timer::Timer,
};

fn main() {
    let mut ctx = Context::new(ContextSettings {
        window_width: 600,
        window_height: 400,
        .. Default::default()
    });

    let mahou_tex = two_d::debug_texture();
    let mahou_td = TextureData::diffuse(&mahou_tex);

    let m3_screen = ortho_screen(glm::vec2(600, 400));

    let draw = ShapeDrawer::new(50);
    let prog = two_d::default_program(None, None).unwrap();
    let locs = DefaultLocations::new(&prog);

    let mut sb = Spritebatch::with_quad(50, false);
    let sb_prog = two_d::default_spritebatch_program(None, None).unwrap();
    let sb_locs = DefaultLocations::new(&sb_prog);

    let font = BitmapFont::new_default();
    let font_tex = font.texture();
    let font_td = TextureData::diffuse(font_tex);

    let mut ps = ParticleSystem::new(100);

    for i in 0..50 {
        ps.spawn(| p | {
            p.age = 0.;
            p.lifetime = 15.;
            p.position = glm::vec2(i as f32 * 6. + 10., 300.);
            p.velocity = glm::vec2(20., -10.);
        });
    }

    ps.spawn_some(10, | (i, p) | {
        p.age = 0.;
        p.lifetime = 5.;
        p.position = glm::vec2(i as f32 * 6. + 10., i as f32 * 2. + 100.);
        p.velocity = glm::vec2(20., 10.);
    });

    let mut tm = Timer::new();
    let mut time = 0.;

    'running: loop {
        for event in ctx.events.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                _ => {}
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::DEPTH_TEST);
        }

        let dt = tm.step();
        time += dt;

        ps.update(dt);

        sb_prog.bind();
        sb_locs.reset();
        sb_locs.screen().set(&m3_screen);
        sb_locs.time().set(&(time));
        sb_locs.diffuse().set(&mahou_td);

        ps.draw();

        prog.bind();
        locs.reset();
        locs.screen().set(&m3_screen);
        draw.filled_rectangle(&locs, 20., 20., 30., 30.);

        ctx.window.gl_swap_window();
        tm.sleep_millis(30);
    }


        /*
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
        */
}

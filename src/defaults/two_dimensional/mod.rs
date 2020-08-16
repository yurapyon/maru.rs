mod locations;
pub use locations::*;

mod vertex2d;
pub use vertex2d::*;

mod spritebatch;
pub use spritebatch::*;

mod bitmap_font;
pub use bitmap_font::*;

mod shape_drawer;
pub use shape_drawer::*;

mod drawer2d;
pub use drawer2d::*;

mod program2d;
pub use program2d::*;

//

use image;

use crate::{
    gfx::{
        Shader,
        Program,
        Texture,
    },
};

// TODO prelude

//

mod shaders {
    pub const DEFAULT_VERT: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                             "/content/shaders/default.vert"));

    pub const DEFAULT_FRAG: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                             "/content/shaders/default.frag"));

    pub const EXTRAS: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                             "/content/shaders/lib.incl.glsl"));

    pub const DEFAULT_SB_VERT: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                             "/content/shaders/spritebatch.incl.vert"));

    pub const DEFAULT_SB_FRAG: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                             "/content/shaders/spritebatch.incl.frag"));
}

mod images {
    pub const MAHOU: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
                               "/content/mahou.jpg"));

    pub const SMALL_FONT: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
                               "/content/small_font.png"));
}

//

// TODO
// change shade template to just use one @ symbol ot signify where effect should go
//   and have the default program effect be an incl, like how shader template is now

fn parse_shader_template<'a: 'b, 'b>(template: &'a str, effect: Option<&'b str>) -> Vec<&'b str> {
    let mut ret: Vec<_> = template.split('@').collect();
    assert!(ret.len() == 3);
    if let Some(effect) = effect {
        ret[1] = effect;
    }
    ret
}

pub fn default_program(v_effect: Option<&str>,
                       f_effect: Option<&str>
    ) -> Result<Program, String> {
    let mut v_strs = parse_shader_template(shaders::DEFAULT_VERT, v_effect);
    v_strs.insert(1, shaders::EXTRAS);
    let vert = Shader::new(gl::VERTEX_SHADER, &v_strs)?;

    let mut f_strs = parse_shader_template(shaders::DEFAULT_FRAG, f_effect);
    f_strs.insert(1, shaders::EXTRAS);
    let frag = Shader::new(gl::FRAGMENT_SHADER, &f_strs)?;

    Program::new(&[vert, frag])
}

/// Creates a default maru spritebatch program.
pub fn default_spritebatch_program(v_effect: Option<&str>,
                                   f_effect: Option<&str>
    ) -> Result<Program, String> {
    let mut v_strs = parse_shader_template(shaders::DEFAULT_VERT,
                                           v_effect.or(Some(shaders::DEFAULT_SB_VERT)));
    v_strs.insert(1, shaders::EXTRAS);
    let vert = Shader::new(gl::VERTEX_SHADER, &v_strs)?;

    let mut f_strs = parse_shader_template(shaders::DEFAULT_FRAG,
                                           f_effect.or(Some(shaders::DEFAULT_SB_FRAG)));
    f_strs.insert(1, shaders::EXTRAS);
    let frag = Shader::new(gl::FRAGMENT_SHADER, &f_strs)?;

    Program::new(&[vert, frag])
}

pub fn debug_texture() -> Texture {
    let img = image::load_from_memory(images::MAHOU).unwrap().to_rgba();
    Texture::new(&img)
}

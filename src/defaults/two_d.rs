use std::{
    mem,
};

use image::{
    self,
    RgbaImage,
    Rgba
};
use memoffset::offset_of;
use nalgebra_glm as glm;

use crate::{
    gfx::*,
    math::{
        Color,
        Transform2d,
    },
};

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
//   default program structs that have locations in them ?
//   change shade tamplet to just use one @ symbol ot signify where effect shoudl go
//     and have the default program effect be an incl, like how shader template is now

fn parse_shader_template<'a: 'b, 'b>(template: &'a str, effect: Option<&'b str>) -> Vec<&'b str> {
    let mut ret: Vec<_> = template.split('@').collect();
    assert!(ret.len() == 3);
    if let Some(effect) = effect {
        ret[1] = effect;
    }
    ret
}

//

/// Creates a default maru program, with optionial vert and frag effects.
pub fn default_program(v_effect: Option<&str>, f_effect: Option<&str>) -> Result<Program, GfxError> {
    let mut v_strs = parse_shader_template(shaders::DEFAULT_VERT, v_effect);
    v_strs.insert(1, shaders::EXTRAS);
    let vert = Shader::new(gl::VERTEX_SHADER, &v_strs)?;

    let mut f_strs = parse_shader_template(shaders::DEFAULT_FRAG, f_effect);
    f_strs.insert(1, shaders::EXTRAS);
    let frag = Shader::new(gl::FRAGMENT_SHADER, &f_strs)?;

    Program::new(&[vert, frag])
}

/// Creates a default maru spritebatch program.
pub fn default_spritebatch_program(v_effect: Option<&str>, f_effect: Option<&str>) -> Result<Program, GfxError> {
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

//

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex2d {
    pub position: glm::Vec2,
    pub uv: glm::Vec2,
}

impl Vertex2d {
    pub fn quad(centered: bool) -> Vec<Self> {
        let mut ret = Vec::with_capacity(4);

        ret.push(Self {
            position: glm::vec2(1., 1.),
            uv:       glm::vec2(1., 1.),
        });

        ret.push(Self {
            position: glm::vec2(1., 0.),
            uv:       glm::vec2(1., 0.),
        });

        ret.push(Self {
            position: glm::vec2(0., 1.),
            uv:       glm::vec2(0., 1.),
        });

        ret.push(Self {
            position: glm::vec2(0., 0.),
            uv:       glm::vec2(0., 0.),
        });

        if centered {
            for vert in ret.iter_mut() {
                vert.position.x -= 0.5;
                vert.position.y -= 0.5;
            }
        }

        ret
    }

    pub fn circle(resolution: usize) -> Vec<Self> {
        use std::f32::consts;

        let mut ret = Vec::new();

        let angle_step = (consts::PI * 2.) / (resolution as f32);

        for i in 0..resolution {
            let at = (i as f32) * angle_step;
            let x = at.cos() / 2.;
            let y = at.sin() / 2.;
            ret.push(Self {
                position: glm::vec2(x, y),
                uv:       glm::vec2(x + 0.5, y + 0.5),
            });
        }

        ret
    }
}

impl Vertex for Vertex2d {
    fn set_attributes(vao: &mut VertexArray) {
        let base = VertexAttribute {
            size: 2,
            ty: gl::FLOAT,
            normalized: false,
            stride: mem::size_of::<Self>(),
            offset: offset_of!(Self, position),
            divisor: 0,
        };

        // TODO enum, move these out of here?
        vao.enable_attribute(0, VertexAttribute {
            offset: offset_of!(Self, position),
            .. base
        });
        vao.enable_attribute(1, VertexAttribute {
            offset: offset_of!(Self, uv),
            .. base
        });
    }
}

pub type Mesh2d = Mesh<Vertex2d>;

//

// TODO think about using mat3 instead of t2d
#[derive(Debug)]
#[repr(C)]
pub struct SbSprite {
    pub uv: UvRegion,
    pub transform: Transform2d,
    pub color: Color,
}

impl Default for SbSprite {
    fn default() -> Self {
        Self {
            uv: UvRegion::new(0., 0., 1., 1.),
            transform: Transform2d::identity(),
            color: Color::new_rgba(1., 1., 1., 1.),
        }
    }
}

impl Vertex for SbSprite {
    fn set_attributes(vao: &mut VertexArray) {
        let base = VertexAttribute {
            size: 4,
            ty: gl::FLOAT,
            normalized: false,
            stride: std::mem::size_of::<Self>(),
            offset: 0,
            divisor: 1,
        };

        // TODO use enum for all the attrib locs

        vao.enable_attribute(2, VertexAttribute {
            offset: offset_of!(Self, uv),
            .. base
        });
        vao.enable_attribute(3, VertexAttribute {
            size: 2,
            offset: offset_of!(Self, transform) +
                    offset_of!(Transform2d, position),
            .. base
        });
        vao.enable_attribute(4, VertexAttribute {
            size: 2,
            offset: offset_of!(Self, transform) +
                    offset_of!(Transform2d, scale),
            .. base
        });
        vao.enable_attribute(5, VertexAttribute {
            size: 1,
            offset: offset_of!(Self, transform) +
                    offset_of!(Transform2d, rotation),
            .. base
        });
        vao.enable_attribute(6, VertexAttribute {
            offset: offset_of!(Self, color),
            .. base
        });
    }
}

/// 2d instancer
pub type Spritebatch = Instancer<SbSprite, Vertex2d>;

impl Spritebatch {
    pub fn with_quad(size: usize, centered: bool) -> Self {
        Self::new(size,
                  Mesh2d::new(Vertex2d::quad(centered),
                              Vec::new(),
                              gl::STATIC_DRAW,
                              gl::TRIANGLE_STRIP))
    }

    pub fn print(&mut self, font: &BitmapFont, text: &str) {
        // TODO set font texture as diffuse
        //        cant do this without diffuse location

        self.begin();

        let mut x = 0.;
        let font_h = font.texture().height() as f32;
        for ch in text.chars() {
            let region_w = font.region(ch).width() as f32;
            let sp = self.pull_default();
            sp.uv = font.uv_region(ch);
            sp.transform.position.x = x;
            sp.transform.scale.x = region_w;
            sp.transform.scale.y = font_h;
            x += region_w + 1.;
        }

        self.end();
    }
}

//

pub struct DefaultLocations {
    screen: Location,
    view: Location,
    model: Location,
    time: Location,
    flip_uvs: Location,
    base_color: Location,
    tx_diffuse: Location,
    tx_normal: Location,
}

impl DefaultLocations {
    pub fn new(program: &Program) -> Self {
        Self {
            screen:     Location::new(program, "_screen"),
            view:       Location::new(program, "_view"),
            model:      Location::new(program, "_model"),
            time:       Location::new(program, "_time"),
            flip_uvs:   Location::new(program, "_flip_uvs"),
            base_color: Location::new(program, "_base_color"),
            tx_diffuse: Location::new(program, "_tx_diffuse"),
            tx_normal:  Location::new(program, "_tx_normal"),
        }
    }

    #[inline]
    pub fn screen(&self) -> &Location {
        &self.screen
    }

    #[inline]
    pub fn view(&self) -> &Location {
        &self.view
    }

    #[inline]
    pub fn model(&self) -> &Location {
        &self.model
    }

    #[inline]
    pub fn time(&self) -> &Location {
        &self.time
    }

    #[inline]
    pub fn flip_uvs(&self) -> &Location {
        &self.flip_uvs
    }

    #[inline]
    pub fn base_color(&self) -> &Location {
        &self.base_color
    }

    #[inline]
    pub fn diffuse(&self) -> &Location {
        &self.tx_diffuse
    }

    #[inline]
    pub fn normal(&self) -> &Location {
        &self.tx_normal
    }

    /// Note: does not set textures
    pub fn reset(&self) {
        let m3_iden = glm::Mat3::identity();
        self.screen().set(&m3_iden);
        self.view().set(&m3_iden);
        self.model().set(&m3_iden);
        self.base_color().set(&Color::white());
        self.flip_uvs().set(&false);
        self.time().set(&0.);
    }

    #[inline]
    pub fn set_rectangle(&self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let temp = Transform2d {
            position: glm::vec2(x1, y1),
            scale:    glm::vec2(x2 - x1, y2 - y1),
            .. Transform2d::identity()
        };
        self.model().set(&glm::Mat3::from(temp));
    }

    #[inline]
    pub fn set_sprite_px(&self, texture: &Texture, transform: &Transform2d) {
        self.diffuse().set(&TextureData::diffuse(texture));
        self.model().set(&glm::Mat3::from(*transform));
    }

    #[inline]
    pub fn set_sprite(&self, texture: &Texture, transform: &Transform2d) {
        let (w, h) = texture.dimensions();
        let temp = Transform2d {
            scale: glm::vec2(transform.scale.x * w as f32,
                             transform.scale.y * h as f32),
            .. *transform
        };
        self.set_sprite_px(texture, &temp);
    }

    #[inline]
    pub fn set_base_color_rgba(&self, r: f32, g: f32, b: f32, a: f32) {
        self.base_color.set(&Color::new_rgba(r, g, b, a));
    }
}

//

// TODO use bdf_font
//        lookup table
//        glyph strings
//          for sb.print
//      include medium sized font and large font
//        new_default_small
//        new_default_medium
//        new_default_large
//        just take font size as an enum
//          new_default(FontSize::Small) etc
//      make font with default alphabet
pub struct BitmapFont {
    texture: Texture,
    regions: Vec<TextureRegion>,
    uv_regions: Vec<UvRegion>,
}

impl BitmapFont {
    pub fn new(image: &RgbaImage, alphabet: &str) -> Self {
        let i_height = image.height();
        let base_color = image.get_pixel(0, 0);

        let top_row = image.enumerate_rows().next().unwrap().1;
        let breaks =
            top_row.filter_map(| (x, _y, px) | {
                if px == base_color {
                    Some(x)
                } else {
                    None
                }
            });
        let breaks_cl = breaks.clone();
        let pairs =
            breaks.zip(breaks_cl.skip(1))
                  .map(| (x1, x2) | TextureRegion::new(x1 + 1, 0, x2, i_height))
                  .zip(alphabet.chars());

        let mut regions = Vec::with_capacity(256);
        // set to 0,0,1,1 so if char is not found, uv is base_color
        regions.resize(256, TextureRegion::new(0, 0, 1, 1));

        for (region, ch) in pairs {
            regions[ch as usize] = region;
        }

        let mut texture = Texture::new(image);
        texture.set_wrap(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE);
        texture.set_filter(gl::NEAREST, gl::NEAREST);

        let tx_point = glm::vec2(texture.width() as u32, texture.height() as u32);

        let uv_regions = regions.iter()
                                .map(| region | region.normalized(tx_point))
                                .collect();

        Self {
            texture,
            regions,
            uv_regions,
        }
    }

    pub fn new_default() -> Self {
        let fn_img = image::load_from_memory(images::SMALL_FONT).unwrap().to_rgba();
        Self::new(&fn_img,
        " ABCDEFGHIJKLMNOPQRSTUVWXYZ\
          abcdefghijklmnopqrstuvwxyz\
          1234567890[](){}=+-/^$@#*~%_<>\"'?!|\\&`.,:;")
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn region(&self, ch: char) -> TextureRegion {
        self.regions[ch as usize]
    }

    pub fn uv_region(&self, ch: char) -> UvRegion {
        self.uv_regions[ch as usize]
    }
}

//

// polygon
// line
// triangle

// lined rect

pub struct ShapeDrawer {
    line_thickness: f32,

    mesh_quad: Mesh2d,
    mesh_circle: Mesh2d,
    tex_white: Texture,
}

impl ShapeDrawer {
    pub fn new(circle_resolution: usize) -> Self {
        let white = RgbaImage::from_pixel(1, 1, Rgba::from([255, 255, 255, 255]));
        Self {
            mesh_quad: Mesh2d::new(Vertex2d::quad(false),
                                   Vec::new(),
                                   gl::STATIC_DRAW,
                                   gl::TRIANGLE_STRIP),
            mesh_circle: Mesh2d::new(Vertex2d::circle(circle_resolution),
                                     Vec::new(),
                                     gl::STATIC_DRAW,
                                     gl::TRIANGLE_FAN),
            tex_white: Texture::new(&white),
            line_thickness: 2.0,
        }
    }

    // TODO clean up api in general
    pub fn draw_quad(&self) {
        self.mesh_quad.draw();
    }

    pub fn line_thickness_mut(&mut self) -> &mut f32 {
        &mut self.line_thickness
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn filled_rectangle(&self, locations: &DefaultLocations, x1: f32, y1: f32, x2: f32, y2: f32) {
        let temp = Transform2d {
            position: glm::vec2(x1, y1),
            scale:    glm::vec2(x2 - x1, y2 - y1),
            .. Transform2d::identity()
        };
        locations.set_sprite_px(&self.tex_white, &temp);
        self.mesh_quad.draw();
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn circle(&self, locations: &DefaultLocations, x: f32, y: f32, r: f32) {
        let temp = Transform2d {
            position: glm::vec2(x, y),
            scale:    glm::vec2(r, r),
            .. Transform2d::identity()
        };
        locations.set_sprite_px(&self.tex_white, &temp);
        self.mesh_circle.draw();
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn horizontal_line(&self, locations: &DefaultLocations, y: f32, x1: f32, x2: f32) {
        let y1 = y - self.line_thickness / 2.;
        let y2 = y + self.line_thickness / 2.;
        self.filled_rectangle(locations, x1, y1, x2, y2);
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn vertical_line(&self, locations: &DefaultLocations, x: f32, y1: f32, y2: f32) {
        let x1 = x - self.line_thickness / 2.;
        let x2 = x + self.line_thickness / 2.;
        self.filled_rectangle(locations, x1, y1, x2, y2);
    }

    /// Sets locations.diffuse() and locations.model()
    pub fn line(&self, locations: &DefaultLocations, x1: f32, y1: f32, x2: f32, y2: f32) {
        if x1 == x2 {
            self.vertical_line(locations, x1, y1, y2);
        } else if y1 == y2 {
            self.horizontal_line(locations, y1, x1, x2);
        } else {
            // TODO
        }
    }
}

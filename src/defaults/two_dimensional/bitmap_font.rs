use image::RgbaImage;
use nalgebra_glm as glm;

use crate::{
    gfx::{
        Texture,
        TextureRegion,
        UvRegion,
    },
};

use super::images;

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
        let i_height = image.height() as i32;
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
                  .map(| (x1, x2) | TextureRegion::new(x1 as i32 + 1, 0, x2 as i32, i_height))
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

        let tx_point = glm::vec2(texture.width(), texture.height());

        let uv_regions = regions.iter()
                                .map(| region | region.normalized(tx_point))
                                .collect();

        Self {
            texture,
            regions,
            uv_regions,
        }
    }

    // TODO make a public funciton for this
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

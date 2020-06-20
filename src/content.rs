pub mod shaders {
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

pub mod image {
    pub const MAHOU: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
                               "/content/mahou.jpg"));

    pub const SMALL_FONT: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
                               "/content/small_font.png"));
}

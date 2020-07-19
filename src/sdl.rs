use gl::{
    self,
    types::*,
};

use sdl2;

pub struct ContextSettings {
    pub ogl_version_major: u32,
    pub ogl_version_minor: u32,

    pub window_width: u32,
    pub window_height: u32,
    pub window_name: String,

    pub is_resizable: bool,
}

impl Default for ContextSettings {
    fn default() -> Self {
        Self {
            ogl_version_major: 3,
            ogl_version_minor: 3,
            window_width: 800,
            window_height: 600,
            window_name: String::from("float"),
            is_resizable: true,
        }
    }
}

pub struct Context {
    settings: ContextSettings,
    pub sdl: sdl2::Sdl,
    pub video: sdl2::VideoSubsystem,
    pub window: sdl2::video::Window,
    pub gl_ctx: sdl2::video::GLContext,
    pub events: sdl2::EventPump,
}

impl Context {
    pub fn new(settings: ContextSettings) -> Self {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = video.window(&settings.window_name,
                                  settings.window_width,
                                  settings.window_height)
                          .opengl()
                          .build()
                          .unwrap();

        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(| name | video.gl_get_proc_address(name) as *const _);

        let events = sdl.event_pump().unwrap();

        Self {
            settings,
            sdl,
            video,
            window,
            gl_ctx,
            events,
        }
    }
}

use gl::{
    self,
    types::*,
};
use sdl2::{
    self,
    event::{
        Event,
        WindowEvent,
    },
};

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

        let mut builder = video.window(&settings.window_name,
                                       settings.window_width,
                                       settings.window_height);

        // TODO allow high dpi ??

        if settings.is_resizable {
            builder.resizable();
        }

        builder.opengl();

        let window = builder.build().unwrap();

        // TODO im guessing that this needs to stay alive
        //      check is window keeps a ref to it or something
        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(| name | video.gl_get_proc_address(name) as *const _);

        video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync).unwrap();

        // TODO error on opengl type not avail

        let mut settings = settings;
        let (window_w, window_h) = window.drawable_size();
        settings.window_width = window_w;
        settings.window_height = window_h;

        unsafe {
            gl::Viewport(0, 0, settings.window_width as GLint, settings.window_height as GLint);
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(0., 0., 0., 0.);
        }

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

    pub fn settings(&self) -> &ContextSettings {
        &self.settings
    }

    pub fn maybe_handle_resize(&mut self, event: &Event) -> bool {
        match event {
            Event::Window {
                win_event,
                ..
            } => {
                match win_event {
                    WindowEvent::Resized(w, h) => {
                        self.settings.window_width = *w as u32;
                        self.settings.window_height = *h as u32;
                        return true;
                    }
                    // TODO for now just ignore this but report it as handled
                    //        figure out what the difference is later
                    WindowEvent::SizeChanged(..) => {
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            _ => {
                return false;
            }
        }
    }

    pub fn window_dimensions(&self) -> (u32, u32) {
        (self.settings.window_width, self.settings.window_height)
    }

    /* TODO where to put this
    //    also fix the math
    pub fn set_scissor(&self, x1: u32, y1: u32, x2: u32, y2: u32) {
        // TODO what to do ?
        /*
        if y1 > self.settings.window_height ||
            y2 > self.settings.window_height {
            panic!("invalid window scissor: {} {} {} {}", x1, y1, x2, y2);
        }
        */
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(x1 as i32, (self.settings.window_height - y1) as i32, x2 as i32, (self.settings.window_height - y2) as i32);
        }
    }

    pub fn clear_scissor(&self) {
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
    }
    */
}

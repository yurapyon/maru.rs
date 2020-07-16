use std::{
    sync::mpsc::Receiver,
};
use gl::{
    self,
    types::*,
};
use glfw::{
    self,
    Context,
};

//

// TODO start fullscreen or not
//      handle resize, will be an event
//      get window dimensions as u32vec2

pub struct GlfwContextSettings {
    pub ogl_version_major: u32,
    pub ogl_version_minor: u32,

    pub window_width: u32,
    pub window_height: u32,
    pub window_name: String,

    pub is_resizable: bool,
}

impl Default for GlfwContextSettings {
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

//

/// A basic GLFW context
pub struct GlfwContext {
    settings: GlfwContextSettings,

    // drop order
    pub events: Receiver<(f64, glfw::WindowEvent)>,
    pub window: glfw::Window,
    pub glfw: glfw::Glfw,
}

impl GlfwContext {
    pub fn new(settings: GlfwContextSettings) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersionMajor(settings.ogl_version_major));
        glfw.window_hint(glfw::WindowHint::ContextVersionMinor(settings.ogl_version_minor));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::Resizable(settings.is_resizable));

        // TODO handle error
        //        returns option?
        let (mut window, events) = glfw.create_window(settings.window_width,
                                                      settings.window_height,
                                                      &settings.window_name,
                                                      glfw::WindowMode::Windowed)
                                       .unwrap();

        window.make_current();
        window.set_all_polling(true);

        gl::load_with(| s | window.get_proc_address(s) as *const _);
        glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

        let mut settings = settings;
        let (window_w, window_h) = window.get_framebuffer_size();
        settings.window_width = window_w as u32;
        settings.window_height = window_h as u32;

        unsafe {
            gl::Viewport(0, 0, settings.window_width as GLint, settings.window_height as GLint);
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(0., 0., 0., 0.);
        }

        Self {
            settings,

            events,
            window,
            glfw,
        }
    }

    pub fn settings(&self) -> &GlfwContextSettings {
        &self.settings
    }
}

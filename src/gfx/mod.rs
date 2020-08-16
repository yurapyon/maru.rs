//! # Graphics
//!
//! graphics module

//

// note: GL types are only used for things that interface directly with ogl
//         such as object handles and enums
//       otherwise just use native types based off of:
//         https://www.khronos.org/opengl/wiki/OpenGL_Type

//

// some type of GL trait?
//   unsafe from GLuint
//   to GLuint

// do fns that dont modify rust data but modify GLdata have to be &mut self?
//    bind and unbind dont need to be mutable
//    anything that does change gl properties of an object yes
//      anything that changes data
//      bind and unbind allow user to change data of bound obj and should technically be mut

// clone glfw context into every gl item,
//   dont have the problem with needing to put things in a certain order in structs
//   incerements glfw ref count
//   might not be guaranteed behavior by glfw lib,
//   but i could still do it on my own w/ an Rc<>
// only create objects through glfw context to manage lifetimes

// stuff for modifying underlying vec and making changes to the buf
//   maybe not super necessary if mapping buffers
/*
pub struct Backed<T: GLBuffer<O>> {
    buf: T,
    vec: Vec<O>,
}

grab a slice [], wrapped in something
on drop, will buffer the data to the buffer, just based on how much of the slice you got
*/

//

mod shader;
pub use shader::*;

mod program;
pub use program::*;

mod texture;
pub use texture::*;

mod location;
pub use location::*;

mod uniform;
pub use uniform::*;

mod buffer;
pub use buffer::*;

mod vertex_array;
pub use vertex_array::*;

mod mesh;
pub use mesh::*;

mod instancer;
pub use instancer::*;

mod canvas;
pub use canvas::*;

//

use crate::math::AABB;

pub type TextureRegion = AABB<i32>;
pub type UvRegion = AABB<f32>;

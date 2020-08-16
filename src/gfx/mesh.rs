use std::ptr;

use gl::{
    self,
    types::*,
};

use super::{
    VertexArray,
    Vertex,
    Buffer,
};

//

pub struct Mesh<T: Vertex> {
    vao: VertexArray,
    vbo: Buffer<T>,
    vertices: Vec<T>,
    ebo: Buffer<u32>,
    indices: Vec<u32>,
    _buffer_type: GLenum,
    draw_type: GLenum,
}

impl<T: Vertex> Mesh<T> {
    pub fn new(vertices: Vec<T>, indices: Vec<u32>, buffer_type: GLenum, draw_type: GLenum) -> Self {
        let mut vao = VertexArray::new();
        let vbo = Buffer::from_slice(&vertices, buffer_type);
        let ebo = Buffer::from_slice(&indices, buffer_type);

        vao.bind();
        vbo.bind_to(gl::ARRAY_BUFFER);
        T::set_attributes(&mut vao);
        ebo.bind_to(gl::ELEMENT_ARRAY_BUFFER);
        vao.unbind();

        Self {
            vao,
            vbo,
            vertices,
            ebo,
            indices,
            _buffer_type: buffer_type,
            draw_type,
        }
    }

    pub fn draw(&self) {
        let i_ct = self.indices.len();

        self.vao.bind();
        if i_ct == 0 {
            unsafe {
                gl::DrawArrays(self.draw_type,
                               0,
                               self.vertices.len() as GLint);
            }
        } else {
            unsafe {
                gl::DrawElements(self.draw_type,
                                 i_ct as GLint,
                                 gl::UNSIGNED_INT,
                                 ptr::null());
            }
        }
        self.vao.unbind();
    }

    pub fn draw_instanced(&self, n: usize) {
        let i_ct = self.indices.len();

        self.vao.bind();
        if i_ct == 0 {
            unsafe {
                gl::DrawArraysInstanced(self.draw_type,
                                        0,
                                        self.vertices.len() as GLint,
                                        n as GLint);
            }
        } else {
            unsafe {
                gl::DrawElementsInstanced(self.draw_type,
                                          i_ct as GLint,
                                          gl::UNSIGNED_INT,
                                          ptr::null(),
                                          n as GLint);
            }
        }
        self.vao.unbind();
    }

    pub fn buffer_data(&mut self) {
        self.vbo.buffer_data(&self.vertices);
        self.ebo.buffer_data(&self.indices);
    }

    // TODO
    // pub fn vertices_mut(&mut self) -> &mut Vertices {
        // &mut self.vertices
    // }

    /// Just be careful
    pub fn vao_mut(&mut self) -> &mut VertexArray {
        &mut self.vao
    }
}

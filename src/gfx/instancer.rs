use super::{
    Vertex,
    Buffer,
    Mesh,
};

//

/// Optimized instancer of a mesh.
pub struct Instancer<T: Vertex> {
    ibo: Buffer<T>,
    vec: Vec<T>,
}

impl<T: Vertex> Instancer<T> {
    pub fn new(size: usize) -> Self {
        let ibo = Buffer::empty(size, gl::STREAM_DRAW);
        let vec = Vec::with_capacity(size);

        Self {
            ibo,
            vec,
        }
    }

    pub fn make_mesh_compatible<M: Vertex>(&self, mesh: &mut Mesh<M>) {
        let vao = mesh.vao_mut();

        vao.bind();
        self.ibo.bind_to(gl::ARRAY_BUFFER);
        T::set_attributes(vao);
        vao.unbind();
    }

    /// note: does not change OpenGL state to bind
    pub fn bind<'a, M: Vertex>(&'a mut self, mesh: &'a Mesh<M>) -> BoundInstancer<'a, T, M> {
        let mut ret = BoundInstancer {
            base: self,
            mesh,
        };
        ret.begin();
        ret
    }

    pub fn fill_count(&self) -> usize {
        self.vec.len()
    }

    pub fn empty_count(&self) -> usize {
        self.vec.capacity() - self.vec.len()
    }
}

pub struct BoundInstancer<'a, T: Vertex, M: Vertex> {
    base: &'a mut Instancer<T>,
    mesh: &'a Mesh<M>,
}

impl<'a, T: Vertex, M: Vertex> BoundInstancer<'a, T, M> {
    /// note: does not change OpenGL state
    fn begin(&mut self) {
        self.base.vec.clear();
    }

    fn end(&mut self) {
        if self.base.fill_count() > 0 {
            self.draw();
        }
    }

    /// note: not expensive to call if instancer is empty
    pub fn draw(&mut self) {
        if self.base.fill_count() > 0 {
            self.base.ibo.buffer_sub_data(0, &self.base.vec);
            self.mesh.draw_instanced(self.base.fill_count());
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.base.vec.clear();
    }

    pub fn push(&mut self, obj: T) {
        if self.base.empty_count() == 0 {
            self.draw();
        }
        self.base.vec.push(obj);
    }

    /// Returns an &mut T for the caller to override.
    /// Will be uninitialized.
    /// note: draw call may happen such that this &mut T will not be included in it
    pub fn pull(&mut self) -> &mut T {
        if self.base.empty_count() == 0 {
            self.draw();
        }
        let len = self.base.vec.len();
        let ret = unsafe {
            self.base.vec.set_len(len + 1);
            self.base.vec.get_unchecked_mut(len)
        };
        ret
    }
}

impl<'a, T: Vertex + Default, M: Vertex> BoundInstancer<'a, T, M> {
    /// Returns an &mut T for the caller to override.
    /// Will be initialized with `DEfault::default()`.
    /// note: draw call may happen such that this &mut T will not be included in it
    pub fn pull_default(&mut self) -> &mut T {
        let ret = self.pull();
        *ret = Default::default();
        ret
    }
}

impl<'a, T: Vertex, M: Vertex> Drop for BoundInstancer<'a, T, M> {
    fn drop(&mut self) {
        self.end();
    }
}

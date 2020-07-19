use ika::{
    Pool,
};
use nalgebra_glm as glm;

use crate::{
    defaults::two_d::{
        Spritebatch,
    },
};

#[derive(Default)]
pub struct Particle {
    pub age: f32,
    pub lifetime: f32,
    pub position: glm::Vec2,
    pub velocity: glm::Vec2,
}

pub struct ParticleSystem {
    sb: Spritebatch,
    pool: Pool<Particle>,
}

impl ParticleSystem {
    pub fn new(size: usize) -> Self {
        let sb = Spritebatch::with_quad(size, true);
        let pool = Pool::new(size);
        Self {
            sb,
            pool,
        }
    }

    // TODO take a count? spawn multiple at once
    pub fn spawn<F: Fn(&mut Particle)>(&mut self, init_fn: F) {
        let particle = self.pool.spawn();
        if let Some(mut p) = particle {
            init_fn(&mut p);
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for particle in self.pool.iter_mut() {
            particle.age += delta_time;
            particle.position.x += particle.velocity.x * delta_time;
            particle.position.y += particle.velocity.y * delta_time;
        }
        self.pool.reclaim(| p | p.age >= p.lifetime || p.position.x > 350.);
    }

    pub fn draw(&mut self) {
        self.sb.begin();
        for particle in self.pool.iter() {
            let sprite = self.sb.pull_default();
            sprite.transform.position = particle.position;
            sprite.transform.scale = glm::vec2(25., 25.);
        }
        self.sb.end();
    }
}

use ika::{
    Pool,
};
use nalgebra_glm as glm;

use crate::{
    defaults::two_d::{
        Spritebatch,
    },
};

// make sure this cann support setting textures

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

    pub fn spawn<F: Fn(&mut Particle)>(&mut self, init_fn: F) {
        let particle = self.pool.spawn();
        if let Some(mut p) = particle {
            init_fn(&mut p);
        }
    }

    pub fn spawn_some<F: Fn((usize, &mut Particle))>(&mut self, count: usize, init_fn: F) {
        let mut particles = self.pool.spawn_some(count);
        for pair in particles.drain(..).enumerate() {
            init_fn(pair);
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for particle in &mut self.pool {
            particle.age += delta_time;
            particle.position.x += particle.velocity.x * delta_time;
            particle.position.y += particle.velocity.y * delta_time;
        }
        println!("{}", self.pool.available());
        self.pool.reclaim(| p | p.age >= p.lifetime || p.position.x > 350.);
    }

    pub fn draw(&mut self) {
        self.sb.begin();
        for particle in &self.pool {
            let sprite = self.sb.pull_default();
            sprite.transform.position = particle.position;
            sprite.transform.scale = glm::vec2(25., 25.);
        }
        self.sb.end();
    }
}

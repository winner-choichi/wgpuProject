use crate::constants::*;
use glam::Vec3;

pub enum Spin {
    Up,   // +0.5
    Down, // - 0.5
}

pub trait Particle {
    fn charge(&self) -> f32;
    fn mass(&self) -> f32;
    fn radius(&self) -> f32;
    fn position(&self) -> Vec3;
    fn velocity(&self) -> Vec3;
}

pub struct Electron {
    pub charge: f32,
    pub mass: f32,
    pub radius: f32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub spin: Spin,
}

impl Particle for Electron {
    fn charge(&self) -> f32 {
        self.charge
    }
    fn mass(&self) -> f32 {
        self.mass
    }
    fn radius(&self) -> f32 {
        self.radius
    }
    fn position(&self) -> Vec3 {
        self.position
    }
    fn velocity(&self) -> Vec3 {
        self.velocity
    }
}

impl Electron {
    pub fn new(position: Vec3, velocity: Vec3, spin: Spin) -> Self {
        Self {
            charge: -1.0 * UNIT_CHARGE,
            mass: ELECTRON_MASS,
            radius: 2.8 * UNIT_LENGTH,
            position,
            velocity,
            spin,
        }
    }
}

pub struct Proton {
    pub charge: f32,
    pub mass: f32,
    pub radius: f32,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Particle for Proton {
    fn charge(&self) -> f32 {
        self.charge
    }
    fn mass(&self) -> f32 {
        self.mass
    }
    fn radius(&self) -> f32 {
        self.radius
    }
    fn position(&self) -> Vec3 {
        self.position
    }
    fn velocity(&self) -> Vec3 {
        self.velocity
    }
}

impl Proton {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self {
            charge: UNIT_CHARGE,
            mass: UNIT_MASS,
            radius: 0.84 * UNIT_LENGTH,
            position,
            velocity,
        }
    }
}

pub struct Neutron {
    pub charge: f32,
    pub mass: f32,
    pub radius: f32,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Particle for Neutron {
    fn charge(&self) -> f32 {
        self.charge
    }
    fn mass(&self) -> f32 {
        self.mass
    }
    fn radius(&self) -> f32 {
        self.radius
    }
    fn position(&self) -> Vec3 {
        self.position
    }
    fn velocity(&self) -> Vec3 {
        self.velocity
    }
}

impl Neutron {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self {
            charge: 0,
            mass: UNIT_MASS,
            radius: 0.84 * UNIT_LENGTH,
            position,
            velocity,
        }
    }
}

use glam::Vec3;

/// Shared behaviour for protons, neutrons, and electrons.
pub trait Particle {
    fn mass(&self) -> f32;
    fn charge(&self) -> f32;
    fn position(&self) -> Vec3;
}

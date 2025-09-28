use crate::physics::elements::Element;
use glam::{Vec3, Vec3A};
use std::f32::consts::PI;

/// Representation of an electron bound to a specific atomic orbital.
#[derive(Clone, Debug)]
pub struct Electron {
    orbital: Orbital,
}

impl Electron {
    pub fn new(orbital: Orbital) -> Self {
        Self { orbital }
    }

    pub fn orbital(&self) -> &Orbital {
        &self.orbital
    }
}

/// Principal quantum numbers that define a hydrogen-like orbital.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orbital {
    pub n: u8,
    pub l: u8,
    pub m: i8,
}

impl Orbital {
    pub fn new(n: u8, l: u8, m: i8) -> Self {
        debug_assert!(n > 0, "Principal quantum number n must be >= 1");
        debug_assert!(l < n, "Azimuthal quantum number l must satisfy l < n");
        debug_assert!(
            m.abs() as u8 <= l,
            "Magnetic quantum number |m| must be <= l"
        );
        Self { n, l, m }
    }

    pub fn ground_state() -> Self {
        Self { n: 1, l: 0, m: 0 }
    }

    /// Approximate maximum probability density for the orbital (atomic units).
    pub fn max_density(&self, element: &Element) -> f32 {
        let a = effective_bohr_radius(element);
        match (self.n, self.l) {
            (1, 0) => 1.0 / (PI * a.powi(3)),
            (2, 0) => 1.0 / (32.0 * PI * a.powi(3)),
            _ => 1.0,
        }
    }

    /// Probability density |psi|^2 evaluated at the given position (atomic units).
    pub fn probability_density(&self, element: &Element, position: Vec3) -> f32 {
        let r = position.length();
        let a = effective_bohr_radius(element);

        match (self.n, self.l, self.m) {
            (1, 0, 0) => {
                let norm = 1.0 / (PI * a.powi(3));
                norm * f32::exp(-2.0 * r / a)
            }
            (2, 0, 0) => {
                let norm = 1.0 / (32.0 * PI * a.powi(3));
                let factor = (2.0 - r / a).powi(2);
                norm * f32::exp(-r / a) * factor
            }
            _ => {
                let sigma = self.bounding_radius(element) / 3.0;
                radial_gaussian(Vec3A::from(position), sigma)
            }
        }
    }

    /// Radius that captures the majority of the orbital probability mass.
    pub fn bounding_radius(&self, element: &Element) -> f32 {
        let a = effective_bohr_radius(element);
        match self.n {
            1 => 4.0 * a,
            2 => 8.0 * a,
            n => (n as f32).powi(2) * 4.0 * a,
        }
    }
}

fn effective_bohr_radius(element: &Element) -> f32 {
    const A0: f32 = 0.529_177_210_67; // Angstroms
    let z = element.atomic_number as f32;
    A0 / z.max(1.0)
}

fn radial_gaussian(position: Vec3A, sigma: f32) -> f32 {
    let r2 = position.length_squared();
    let sigma2 = sigma * sigma;
    let norm = 1.0 / ((2.0 * PI).powf(1.5) * sigma.powi(3));
    norm * f32::exp(-0.5 * r2 / sigma2)
}

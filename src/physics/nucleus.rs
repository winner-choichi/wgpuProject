use crate::physics::particle::Particle;
use glam::Vec3;

pub const PROTON_MASS_AMU: f32 = 1.007_276;
pub const NEUTRON_MASS_AMU: f32 = 1.008_665;
pub const ELEMENTARY_CHARGE: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct Proton {
    position: Vec3,
}

#[derive(Clone, Debug)]
pub struct Neutron {
    position: Vec3,
}

impl Proton {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
}

impl Neutron {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
}

impl Particle for Proton {
    fn mass(&self) -> f32 {
        PROTON_MASS_AMU
    }

    fn charge(&self) -> f32 {
        ELEMENTARY_CHARGE
    }

    fn position(&self) -> Vec3 {
        self.position
    }
}

impl Particle for Neutron {
    fn mass(&self) -> f32 {
        NEUTRON_MASS_AMU
    }

    fn charge(&self) -> f32 {
        0.0
    }

    fn position(&self) -> Vec3 {
        self.position
    }
}

#[derive(Clone, Debug)]
pub struct Nucleus {
    pub protons: Vec<Proton>,
    pub neutrons: Vec<Neutron>,
}

impl Nucleus {
    pub fn proton_count(&self) -> usize {
        self.protons.len()
    }

    pub fn neutron_count(&self) -> usize {
        self.neutrons.len()
    }

    pub fn total_mass(&self) -> f32 {
        self.protons
            .iter()
            .map(Particle::mass)
            .chain(self.neutrons.iter().map(Particle::mass))
            .sum()
    }
}

pub struct NucleusBuilder {
    proton_count: usize,
    neutron_count: usize,
}

impl NucleusBuilder {
    pub fn new(proton_count: usize, neutron_count: usize) -> Self {
        Self {
            proton_count,
            neutron_count,
        }
    }

    pub fn build(&self) -> Nucleus {
        let total = (self.proton_count + self.neutron_count).max(1);
        let base_radius = nuclear_radius(total as f32);

        let proton_positions = fibonacci_sphere(self.proton_count, base_radius * 0.6);
        let neutron_positions = fibonacci_sphere(self.neutron_count, base_radius);

        let protons = proton_positions.into_iter().map(Proton::new).collect();
        let neutrons = neutron_positions.into_iter().map(Neutron::new).collect();

        Nucleus { protons, neutrons }
    }
}

fn nuclear_radius(mass_number: f32) -> f32 {
    // Empirical nuclear radius: r = r0 * A^(1/3). Scale down for visualization units.
    let r0 = 1.2_f32;
    (r0 * mass_number.cbrt()) * 0.01
}

fn fibonacci_sphere(count: usize, radius: f32) -> Vec<Vec3> {
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![Vec3::ZERO];
    }

    let mut points = Vec::with_capacity(count);
    let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());

    for i in 0..count {
        let y = 1.0 - (2.0 * (i as f32 + 0.5) / count as f32);
        let radius_xy = (1.0 - y * y).max(0.0).sqrt();
        let theta = golden_angle * i as f32;
        let x = radius_xy * theta.cos();
        let z = radius_xy * theta.sin();
        points.push(Vec3::new(x, y, z) * radius);
    }

    points
}

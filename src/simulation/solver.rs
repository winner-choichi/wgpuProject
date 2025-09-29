use crate::physics::electron::Orbital;
use crate::physics::elements::Element;
use glam::Vec3;
use log::warn;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Gamma};
use std::f32::consts::TAU;

#[derive(Clone, Debug)]
pub struct CloudSample {
    pub position: Vec3,
    pub weight: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SampleConfig {
    pub samples: usize,
}

impl SampleConfig {
    pub const fn new(samples: usize) -> Self {
        Self { samples }
    }
}

pub struct MonteCarloSampler {
    rng: ChaCha8Rng,
}

impl MonteCarloSampler {
    pub fn new() -> Self {
        Self::with_seed(42)
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn sample_orbital(
        &mut self,
        element: &Element,
        orbital: &Orbital,
        config: SampleConfig,
    ) -> Vec<CloudSample> {
        if config.samples == 0 {
            return Vec::new();
        }

        if let Some(samples) = self.try_sample_ground_s_orbital(element, orbital, config) {
            return samples;
        }

        self.sample_orbital_rejection(element, orbital, config)
    }

    fn try_sample_ground_s_orbital(
        &mut self,
        element: &Element,
        orbital: &Orbital,
        config: SampleConfig,
    ) -> Option<Vec<CloudSample>> {
        if !(orbital.n == 1 && orbital.l == 0 && orbital.m == 0) {
            return None;
        }

        let scale = orbital.effective_bohr_radius(element) / 2.0;
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }

        let gamma = Gamma::<f64>::new(3.0, f64::from(scale)).ok()?;
        let max_density = orbital.max_density(element).max(1e-6);
        let mut samples = Vec::with_capacity(config.samples);

        for _ in 0..config.samples {
            let r = gamma.sample(&mut self.rng) as f32;
            let direction = random_unit_vector(&mut self.rng);
            let position = direction * r;
            let density = orbital.probability_density(element, position);
            let weight = (density / max_density).clamp(0.0, 1.0).sqrt();

            samples.push(CloudSample { position, weight });
        }

        Some(samples)
    }

    fn sample_orbital_rejection(
        &mut self,
        element: &Element,
        orbital: &Orbital,
        config: SampleConfig,
    ) -> Vec<CloudSample> {
        let radius = orbital.bounding_radius(element);
        let mut bounds = BoundingBox::cube(radius);
        let max_density = orbital.max_density(element).max(1e-6);
        let mut accepted = Vec::with_capacity(config.samples);

        let mut expansions = 0usize;
        while accepted.len() < config.samples && expansions <= 4 {
            let mut attempts = 0usize;
            let max_attempts = config.samples.saturating_mul(50).max(10_000);

            while accepted.len() < config.samples && attempts < max_attempts {
                attempts += 1;
                let candidate = bounds.random_point(&mut self.rng);
                let density = orbital.probability_density(element, candidate);
                let threshold: f32 = self.rng.gen_range(0.0..=max_density);

                if threshold <= density {
                    let weight = (density / max_density).clamp(0.0, 1.0) as f32;
                    accepted.push(CloudSample {
                        position: candidate,
                        weight: weight.sqrt(),
                    });
                }
            }

            if accepted.len() < config.samples {
                expansions += 1;
                bounds = bounds.scaled(1.5);
            }
        }

        if accepted.len() < config.samples {
            let remaining = config.samples - accepted.len();
            warn!(
                "Monte Carlo sampling accepted {} / {} points for {}-orbital; filling remainder with low-weight samples",
                accepted.len(),
                config.samples,
                orbital.n
            );
            for _ in 0..remaining {
                let candidate = bounds.random_point(&mut self.rng);
                accepted.push(CloudSample {
                    position: candidate,
                    weight: 0.0,
                });
            }
        }

        accepted
    }
}

fn random_unit_vector<R: Rng>(rng: &mut R) -> Vec3 {
    let z = rng.gen_range(-1.0f32..1.0f32);
    let azimuth = rng.gen_range(0.0f32..TAU);
    let radial = (1.0 - z * z).max(0.0).sqrt();
    Vec3::new(radial * azimuth.cos(), radial * azimuth.sin(), z)
}

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn cube(radius: f32) -> Self {
        let half = radius.abs();
        Self {
            min: Vec3::splat(-half),
            max: Vec3::splat(half),
        }
    }

    pub fn random_point<R: Rng>(&self, rng: &mut R) -> Vec3 {
        let x = rng.gen_range(self.min.x..=self.max.x);
        let y = rng.gen_range(self.min.y..=self.max.y);
        let z = rng.gen_range(self.min.z..=self.max.z);
        Vec3::new(x, y, z)
    }

    pub fn scaled(&self, factor: f32) -> Self {
        Self {
            min: self.min * factor,
            max: self.max * factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::elements::Element;

    #[test]
    fn sampler_returns_points() {
        let element = Element::hydrogen();
        let orbital = Orbital::ground_state();
        let mut sampler = MonteCarloSampler::with_seed(1);
        let samples = sampler.sample_orbital(&element, &orbital, SampleConfig::new(100));
        assert_eq!(samples.len(), 100);
        let mean_radius: f32 =
            samples.iter().map(|s| s.position.length()).sum::<f32>() / samples.len() as f32;
        assert!(mean_radius.is_finite());
    }
}

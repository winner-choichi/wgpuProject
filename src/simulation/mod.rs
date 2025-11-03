pub mod atom;
pub mod solver;

pub mod ozone {
    use rand::{Rng, SeedableRng, rngs::StdRng};

    /// Parameters describing the simplified ozone filtering simulation.
    #[derive(Clone, Copy, Debug)]
    pub struct OzoneParameters {
        pub dobson_units: f32,
        pub photon_count: usize,
    }

    impl Default for OzoneParameters {
        fn default() -> Self {
            Self {
                dobson_units: 300.0,
                photon_count: 10_000,
            }
        }
    }

    /// Incident and transmitted UV flux values (arbitrary units proportional to photon count).
    #[derive(Clone, Copy, Debug, Default)]
    pub struct RadiationFlux {
        pub incident_uva: f32,
        pub incident_uvb: f32,
        pub transmitted_uva: f32,
        pub transmitted_uvb: f32,
    }

    impl RadiationFlux {
        pub fn transmitted_total(&self) -> f32 {
            self.transmitted_uva + self.transmitted_uvb
        }

        pub fn incident_total(&self) -> f32 {
            self.incident_uva + self.incident_uvb
        }

        pub fn absorbed_fraction(&self) -> f32 {
            let incident = self.incident_total();
            if incident <= f32::EPSILON {
                0.0
            } else {
                ((incident - self.transmitted_total()) / incident).clamp(0.0, 1.0)
            }
        }
    }

    /// Result of running an ozone filtering pass, including highlighted indices for electron clouds.
    #[derive(Clone, Debug)]
    pub struct OzoneResult {
        pub flux: RadiationFlux,
        pub highlight_indices: Vec<usize>,
        pub highlight_boost: f32,
    }

    impl Default for OzoneResult {
        fn default() -> Self {
            Self {
                flux: RadiationFlux::default(),
                highlight_indices: Vec::new(),
                highlight_boost: 0.0,
            }
        }
    }

    /// Lightweight simulator that approximates how an ozone column attenuates incoming UV light.
    #[derive(Default)]
    pub struct OzoneSimulation;

    impl OzoneSimulation {
        pub fn new() -> Self {
            Self
        }

        pub fn run(&self, params: OzoneParameters, available_vertices: usize) -> OzoneResult {
            let clamped_du = params.dobson_units.clamp(100.0, 600.0);
            let clamped_photons = params.photon_count.max(100);
            let flux = compute_flux(clamped_du, clamped_photons);

            if available_vertices == 0 {
                return OzoneResult {
                    flux,
                    ..OzoneResult::default()
                };
            }

            let absorbed_fraction = flux.absorbed_fraction();
            let highlight_boost = 0.35 + absorbed_fraction * 0.45;
            let target_highlights =
                (available_vertices as f32 * absorbed_fraction * 0.25).round() as usize;
            let highlight_count = target_highlights.max(1).min(available_vertices.min(500));

            let mut rng = seeded_rng(clamped_du, available_vertices as u64);
            let mut highlights = Vec::with_capacity(highlight_count);
            while highlights.len() < highlight_count {
                let idx = rng.gen_range(0..available_vertices);
                if !highlights.contains(&idx) {
                    highlights.push(idx);
                }
            }

            OzoneResult {
                flux,
                highlight_indices: highlights,
                highlight_boost,
            }
        }
    }

    const INCIDENT_UVA_FRACTION: f32 = 0.95;
    const INCIDENT_UVB_FRACTION: f32 = 0.05;
    const SIGMA_UVA: f32 = 0.08;
    const SIGMA_UVB: f32 = 0.6;

    fn compute_flux(dobson_units: f32, photon_count: usize) -> RadiationFlux {
        let column_scale = dobson_units / 300.0;
        let photons = photon_count as f32;
        let incident_uva = INCIDENT_UVA_FRACTION * photons;
        let incident_uvb = INCIDENT_UVB_FRACTION * photons;

        let transmitted_uva = incident_uva * (-SIGMA_UVA * column_scale).exp();
        let transmitted_uvb = incident_uvb * (-SIGMA_UVB * column_scale).exp();

        RadiationFlux {
            incident_uva,
            incident_uvb,
            transmitted_uva,
            transmitted_uvb,
        }
    }

    fn seeded_rng(dobson_units: f32, vertex_hash: u64) -> StdRng {
        let du_scaled = (dobson_units * 10.0).round() as u64;
        let seed = du_scaled ^ vertex_hash.rotate_left(13) ^ 0x9E3779B97F4A7C15;
        StdRng::seed_from_u64(seed)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn absorbed_fraction_changes_with_du() {
            let thick = compute_flux(350.0, 10_000);
            let thin = compute_flux(150.0, 10_000);
            assert!(thin.transmitted_uvb > thick.transmitted_uvb);
            assert!(thick.absorbed_fraction() > thin.absorbed_fraction());
        }

        #[test]
        fn highlight_selection_respects_vertex_count() {
            let simulation = OzoneSimulation::new();
            let result = simulation.run(OzoneParameters::default(), 32);
            assert!(result.highlight_indices.len() <= 32);
            assert!(result.highlight_boost > 0.0);
        }
    }
}

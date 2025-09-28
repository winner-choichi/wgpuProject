use crate::physics::electron::{Electron, Orbital};
use crate::physics::elements::Element;
use crate::physics::nucleus::{Nucleus, NucleusBuilder};

#[derive(Clone, Debug)]
pub struct Atom {
    element: Element,
    nucleus: Nucleus,
    electrons: Vec<Electron>,
    active_orbital: Orbital,
}

impl Atom {
    pub fn new(element: Element) -> Self {
        let active_orbital = Orbital::ground_state();
        let electrons = (0..element.atomic_number)
            .map(|_| Electron::new(active_orbital.clone()))
            .collect();

        let neutron_count = element.default_neutron_count();
        let nucleus = NucleusBuilder::new(element.atomic_number as usize, neutron_count).build();

        Self {
            element,
            nucleus,
            electrons,
            active_orbital,
        }
    }

    pub fn element(&self) -> &Element {
        &self.element
    }

    pub fn nucleus(&self) -> &Nucleus {
        &self.nucleus
    }

    pub fn electrons(&self) -> &[Electron] {
        &self.electrons
    }

    pub fn active_orbital(&self) -> &Orbital {
        &self.active_orbital
    }

    pub fn set_active_orbital(&mut self, orbital: Orbital) {
        self.active_orbital = orbital.clone();
        for electron in &mut self.electrons {
            *electron = Electron::new(orbital.clone());
        }
    }
}

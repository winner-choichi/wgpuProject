use serde::{Deserialize, Serialize};

/// Basic metadata describing a chemical element.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Element {
    pub atomic_number: u8,
    pub symbol: &'static str,
    pub name: &'static str,
    pub standard_atomic_weight: f32,
    pub default_neutrons: u8,
}

const ELEMENTS: [Element; 2] = [
    Element::new(1, "H", "Hydrogen", 1.008, 0),
    Element::new(2, "He", "Helium", 4.0026, 2),
];

impl Element {
    pub const fn new(
        atomic_number: u8,
        symbol: &'static str,
        name: &'static str,
        standard_atomic_weight: f32,
        default_neutrons: u8,
    ) -> Self {
        Self {
            atomic_number,
            symbol,
            name,
            standard_atomic_weight,
            default_neutrons,
        }
    }

    pub const fn hydrogen() -> Self {
        Self::new(1, "H", "Hydrogen", 1.008, 0)
    }

    pub const fn helium() -> Self {
        Self::new(2, "He", "Helium", 4.0026, 2)
    }

    pub fn by_atomic_number(z: u8) -> Option<Self> {
        ELEMENTS
            .iter()
            .find(|element| element.atomic_number == z)
            .cloned()
    }

    pub fn all() -> &'static [Element] {
        &ELEMENTS
    }

    pub fn default_neutron_count(&self) -> usize {
        usize::from(self.default_neutrons)
    }

    pub fn symbol(&self) -> &'static str {
        self.symbol
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

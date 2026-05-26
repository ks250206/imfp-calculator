use periodic_table_on_an_enum::{Element, periodic_table};
use tpp2m_core::Tpp2mInput;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialPreset {
    pub material_name: &'static str,
    pub density_g_cm3: f64,
    pub molar_mass_g_mol: f64,
    pub valence_electrons: f64,
    pub band_gap_e_v: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct XrayPreset {
    pub label: &'static str,
    pub electron_energy_e_v: f64,
}

pub const XRAY_PRESETS: [XrayPreset; 4] = [
    XrayPreset {
        label: "Al Kα",
        electron_energy_e_v: 1486.6,
    },
    XrayPreset {
        label: "Mg Kα",
        electron_energy_e_v: 1253.6,
    },
    XrayPreset {
        label: "Cr Kα",
        electron_energy_e_v: 5414.8,
    },
    XrayPreset {
        label: "Ga Kα",
        electron_energy_e_v: 9252.13,
    },
];

pub fn element_presets() -> Vec<MaterialPreset> {
    periodic_table()
        .map(|element| {
            let atomic_number = element.get_atomic_number();
            MaterialPreset {
                material_name: element.get_symbol(),
                density_g_cm3: positive_or_one(element.get_density()),
                molar_mass_g_mol: positive_or_one(element.get_atomic_mass()),
                valence_electrons: TPP2M_VALENCE_ELECTRONS[atomic_number - 1],
                band_gap_e_v: if element.get_symbol() == "Si" {
                    1.12
                } else {
                    0.0
                },
            }
        })
        .collect()
}

pub fn preset_by_symbol(symbol: &str) -> Option<MaterialPreset> {
    let element = Element::from_symbol(symbol)?;
    let atomic_number = element.get_atomic_number();
    Some(MaterialPreset {
        material_name: element.get_symbol(),
        density_g_cm3: positive_or_one(element.get_density()),
        molar_mass_g_mol: positive_or_one(element.get_atomic_mass()),
        valence_electrons: TPP2M_VALENCE_ELECTRONS[atomic_number - 1],
        band_gap_e_v: if symbol == "Si" { 1.12 } else { 0.0 },
    })
}

impl MaterialPreset {
    pub fn to_input(self, electron_energy_e_v: f64, allow_extrapolate: bool) -> Tpp2mInput {
        Tpp2mInput {
            electron_energy_e_v,
            density_g_cm3: self.density_g_cm3,
            molar_mass_g_mol: self.molar_mass_g_mol,
            valence_electrons: self.valence_electrons,
            band_gap_e_v: self.band_gap_e_v,
            allow_extrapolate,
        }
    }
}

fn positive_or_one(value: f32) -> f64 {
    let value = f64::from(value);
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[rustfmt::skip]
const TPP2M_VALENCE_ELECTRONS: [f64; 118] = [
    1.0, 2.0,
    1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
    1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
    1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
    1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
    1.0, 2.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0,
    4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
    1.0, 2.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0,
    4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_all_element_presets() {
        let presets = element_presets();

        assert_eq!(presets.len(), 118);
        assert_eq!(presets[0].material_name, "H");
        assert_eq!(presets[5].material_name, "C");
        assert!(presets.iter().all(|preset| preset.density_g_cm3 > 0.0));
        assert!(presets.iter().all(|preset| preset.molar_mass_g_mol > 0.0));
        assert!(presets.iter().all(|preset| preset.valence_electrons > 0.0));
    }

    #[test]
    fn silicon_preset_keeps_semiconductor_band_gap() {
        let silicon = preset_by_symbol("Si").expect("Si preset exists");

        assert_eq!(silicon.material_name, "Si");
        assert_eq!(silicon.valence_electrons, 4.0);
        assert_eq!(silicon.band_gap_e_v, 1.12);
    }

    #[test]
    fn xray_presets_match_requested_energies() {
        assert_eq!(XRAY_PRESETS[0].electron_energy_e_v, 1486.6);
        assert_eq!(XRAY_PRESETS[3].electron_energy_e_v, 9252.13);
    }
}

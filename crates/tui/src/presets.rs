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

#[derive(Clone, Copy, Debug, PartialEq)]
struct Tpp2mPresetOverride {
    symbol: &'static str,
    density_g_cm3: f64,
    molar_mass_g_mol: Option<f64>,
    valence_electrons: f64,
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

const TPP2M_PRESET_OVERRIDES: [Tpp2mPresetOverride; 7] = [
    Tpp2mPresetOverride {
        symbol: "H",
        density_g_cm3: 0.071,
        molar_mass_g_mol: None,
        valence_electrons: 1.0,
    },
    Tpp2mPresetOverride {
        symbol: "Li",
        density_g_cm3: 0.53,
        molar_mass_g_mol: Some(6.94),
        valence_electrons: 1.0,
    },
    Tpp2mPresetOverride {
        symbol: "Si",
        density_g_cm3: 2.3296,
        molar_mass_g_mol: None,
        valence_electrons: 4.0,
    },
    Tpp2mPresetOverride {
        symbol: "Fe",
        density_g_cm3: 7.86,
        molar_mass_g_mol: None,
        valence_electrons: 8.0,
    },
    Tpp2mPresetOverride {
        symbol: "Ru",
        density_g_cm3: 12.2,
        molar_mass_g_mol: None,
        valence_electrons: 8.0,
    },
    Tpp2mPresetOverride {
        symbol: "Ce",
        density_g_cm3: 6.668,
        molar_mass_g_mol: None,
        valence_electrons: 9.0,
    },
    Tpp2mPresetOverride {
        symbol: "Bi",
        density_g_cm3: 9.8,
        molar_mass_g_mol: None,
        valence_electrons: 5.0,
    },
];

pub fn element_presets() -> Vec<MaterialPreset> {
    periodic_table().map(material_preset_from_element).collect()
}

pub fn preset_by_symbol(symbol: &str) -> Option<MaterialPreset> {
    Element::from_symbol(symbol).map(material_preset_from_element)
}

fn material_preset_from_element(element: Element) -> MaterialPreset {
    let symbol = element.get_symbol();
    let atomic_number = element.get_atomic_number();
    let override_values = tpp2m_override(symbol);
    MaterialPreset {
        material_name: symbol,
        density_g_cm3: override_values
            .map(|values| values.density_g_cm3)
            .unwrap_or_else(|| positive_or_one(element.get_density())),
        molar_mass_g_mol: override_values
            .and_then(|values| values.molar_mass_g_mol)
            .unwrap_or_else(|| positive_or_one(element.get_atomic_mass())),
        valence_electrons: override_values
            .map(|values| values.valence_electrons)
            .unwrap_or(TPP2M_VALENCE_ELECTRONS[atomic_number - 1]),
        band_gap_e_v: if symbol == "Si" { 1.12 } else { 0.0 },
    }
}

fn tpp2m_override(symbol: &str) -> Option<Tpp2mPresetOverride> {
    TPP2M_PRESET_OVERRIDES
        .iter()
        .copied()
        .find(|values| values.symbol == symbol)
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

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let diff = (actual - expected).abs();
        assert!(
            diff <= tolerance,
            "actual={actual}, expected={expected}, diff={diff}, tolerance={tolerance}"
        );
    }

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
    fn lithium_preset_uses_precise_molar_mass() {
        let lithium = preset_by_symbol("Li").expect("Li preset exists");

        assert_eq!(lithium.density_g_cm3, 0.53);
        assert_eq!(lithium.molar_mass_g_mol, 6.94);
    }

    #[test]
    fn xray_presets_match_requested_energies() {
        assert_eq!(XRAY_PRESETS[0].electron_energy_e_v, 1486.6);
        assert_eq!(XRAY_PRESETS[3].electron_energy_e_v, 9252.13);
    }

    #[test]
    fn element_presets_match_extracted_reference_cases() {
        // Extracted from examples/test_case.xlsx; the workbook itself is not a test fixture.
        let cases = [
            ("H", 1000.0, 3.417),
            ("Li", 1000.0, 3.413),
            ("Si", 1000.0, 2.386),
            ("Fe", 1000.0, 1.639),
            ("Ru", 1000.0, 1.438),
            ("Ce", 1000.0, 1.973),
            ("Bi", 1000.0, 2.318),
            ("H", 8000.0, 20.699),
            ("Li", 8000.0, 19.812),
            ("Si", 8000.0, 13.217),
            ("Fe", 8000.0, 8.676),
            ("Ru", 8000.0, 7.485),
            ("Ce", 8000.0, 10.540),
            ("Bi", 8000.0, 12.227),
            ("H", 1486.6, 4.785),
            ("Li", 1486.6, 4.730),
            ("Si", 1486.6, 3.265),
            ("Fe", 1486.6, 2.216),
            ("Ru", 1486.6, 1.936),
            ("Ce", 1486.6, 2.675),
            ("Bi", 1486.6, 3.133),
        ];

        for (symbol, energy, expected_imfp_nm) in cases {
            let preset = preset_by_symbol(symbol).expect("reference element preset exists");
            let result = tpp2m_core::calculate(preset.to_input(energy, true)).unwrap();

            assert_close(result.imfp_nm, expected_imfp_nm, 0.0015);
        }
    }
}

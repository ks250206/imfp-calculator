use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tpp2mInput {
    pub electron_energy_e_v: f64,
    pub density_g_cm3: f64,
    pub molar_mass_g_mol: f64,
    pub valence_electrons: f64,
    pub band_gap_e_v: f64,
    #[serde(default)]
    pub allow_extrapolate: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Spacing {
    #[default]
    Log,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SweepInput {
    pub material: Tpp2mInput,
    #[serde(default = "default_energy_min")]
    pub energy_min_e_v: f64,
    #[serde(default = "default_energy_max")]
    pub energy_max_e_v: f64,
    #[serde(default = "default_points")]
    pub points: usize,
    #[serde(default)]
    pub spacing: Spacing,
}

impl SweepInput {
    pub fn with_defaults(material: Tpp2mInput) -> Self {
        Self {
            material,
            energy_min_e_v: default_energy_min(),
            energy_max_e_v: default_energy_max(),
            points: default_points(),
            spacing: Spacing::Log,
        }
    }
}

pub const fn default_energy_min() -> f64 {
    50.0
}

pub const fn default_energy_max() -> f64 {
    2000.0
}

pub const fn default_points() -> usize {
    200
}

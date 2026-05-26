use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn calc_json_matches_reference_vector() {
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    let output = command
        .args([
            "calc", "-E", "1000", "-r", "2.3296", "-M", "28.0855", "-v", "4", "-g", "1.12",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["input"]["electron_energy_e_v"], 1000.0);
    let imfp = json["output"]["imfp_nm"].as_f64().unwrap();
    assert!((imfp - 2.3864329956020653).abs() < 1e-12);
}

#[test]
fn calc_text_prints_default_result() {
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    command
        .args([
            "calc", "-E", "1000", "-r", "2.3296", "-M", "28.0855", "-v", "4", "-g", "1.12",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("IMFP: 2.386433 nm"));
}

#[test]
fn calc_verbose_prints_diagnostic_coefficients() {
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    command
        .args([
            "--verbose",
            "calc",
            "-E",
            "1000",
            "-r",
            "2.3296",
            "-M",
            "28.0855",
            "-v",
            "4",
            "-g",
            "1.12",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ep: 16.589072 eV"))
        .stdout(predicate::str::contains("beta: 0.031865"))
        .stdout(predicate::str::contains("gamma: 0.125139 1/eV"));
}

#[test]
fn sweep_csv_uses_fixed_header() {
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    command
        .args([
            "sweep",
            "--energy-min",
            "50",
            "--energy-max",
            "100",
            "--points",
            "2",
            "--spacing",
            "linear",
            "-r",
            "2.3296",
            "-M",
            "28.0855",
            "-v",
            "4",
            "-g",
            "1.12",
            "--csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "electron_energy_e_v,imfp_nm,imfp_angstrom,warning",
        ));
}

#[test]
fn sweep_json_contains_points_and_input() {
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    let output = command
        .args([
            "sweep",
            "--energy-min",
            "50",
            "--energy-max",
            "100",
            "--points",
            "2",
            "--spacing",
            "linear",
            "-r",
            "2.3296",
            "-M",
            "28.0855",
            "-v",
            "4",
            "-g",
            "1.12",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["points"].as_array().map(Vec::len), Some(2));
    assert_eq!(json["input"]["energy_min_e_v"], 50.0);
}

#[test]
fn plot_writes_smooth_svg_polyline() {
    let output_path = std::env::temp_dir().join(format!(
        "tpp2m-plot-{}-{}.svg",
        std::process::id(),
        unique_suffix()
    ));
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    command
        .args([
            "plot",
            "--energy-min",
            "50",
            "--energy-max",
            "2000",
            "--points",
            "32",
            "--spacing",
            "log",
            "-r",
            "2.3296",
            "-M",
            "28.0855",
            "-v",
            "4",
            "-g",
            "1.12",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let svg = std::fs::read_to_string(&output_path).unwrap();
    let _ = std::fs::remove_file(&output_path);

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<polyline"));
    assert!(svg.contains(r#"stroke="red""#));
    assert!(svg.contains(r#"stroke-linejoin="round""#));
    assert!(svg.contains("Electron Energy (eV)"));
}

#[test]
fn extrapolation_flag_allows_out_of_range_calc() {
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    command
        .args([
            "calc",
            "-E",
            "5000",
            "-r",
            "2.3296",
            "-M",
            "28.0855",
            "-v",
            "4",
            "--allow-extrapolate",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("IMFP:"));
}

#[test]
fn output_format_conflict_exits_with_code_3() {
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    command
        .args([
            "sweep", "-r", "2.3296", "-M", "28.0855", "-v", "4", "--json", "--csv",
        ])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("OutputFormatConflict"));
}

#[test]
fn out_of_range_requires_extrapolation_flag() {
    let mut command = Command::cargo_bin("tpp2m").unwrap();

    command
        .args([
            "calc", "-E", "5000", "-r", "2.3296", "-M", "28.0855", "-v", "4",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("OutOfRecommendedRange"));
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

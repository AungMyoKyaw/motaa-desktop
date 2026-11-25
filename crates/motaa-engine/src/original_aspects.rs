//! Pure numeric reconstruction from supplied APK: main.C/D/E/F/G/D0.
//! It does not implement full MOTAA predictions or planetary conventions.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalAspectCodes {
    pub focal_longitude_deg: f64,
    pub codes: [u8; 13],
    pub source: String,
}
fn matches_angle(mut a: f64, mut b: f64, orb: f64) -> bool {
    if a > 360.0 {
        a -= 360.0
    }
    if b > 360.0 {
        b -= 360.0
    }
    let mut delta = (a - b).abs();
    if delta > 360.0 - orb {
        delta = 360.0 - delta
    }
    delta <= orb
}
pub fn original_c(a: f64, b: f64) -> bool {
    matches_angle(a, b, 2.0)
}
pub fn original_d(a: f64, b: f64) -> bool {
    matches_angle(a, b, 3.0)
}
pub fn original_e(a: f64, b: f64) -> bool {
    matches_angle(a, b, 6.0)
}
pub fn original_f(a: f64, b: f64) -> bool {
    matches_angle(a, b, 8.0)
}
/// Original APK main.G, accepting 13 longitude parameters (d2..d14).
/// Last matching code overrides earlier ones, mirroring independent Java ifs.
pub fn original_g(longitudes: [f64; 13]) -> Result<OriginalAspectCodes, String> {
    if longitudes.iter().any(|n| !n.is_finite()) {
        return Err("Nonfinite original aspect input".into());
    }
    let focal = longitudes[0];
    let rules = [
        (0.0, 8.0, 1),
        (180.0, 8.0, 2),
        (120.0, 8.0, 3),
        (90.0, 8.0, 4),
        (60.0, 6.0, 5),
        (150.0, 3.0, 6),
        (30.0, 2.0, 7),
        (45.0, 2.0, 8),
        (135.0, 2.0, 9),
        (72.0, 2.0, 10),
        (144.0, 2.0, 11),
    ];
    let mut codes = [0; 13];
    for index in 1..13 {
        let base = longitudes[index];
        for (offset, orb, code) in rules {
            if matches_angle(focal, base + offset, orb) {
                codes[index] = code
            }
        }
    }
    Ok(OriginalAspectCodes {
        focal_longitude_deg: focal,
        codes,
        source: "Original APK main.G aspect code classification".into(),
    })
}
/// Original APK main.H returns the directed 13×13 aspect-code matrix.
/// Row is target body and column is focal body, with zero diagonal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalAspectMatrix {
    pub codes: [[u8; 13]; 13],
    pub source: String,
}
fn code_for_pair(focal: f64, other: f64) -> u8 {
    let rules = [
        (0.0, 8.0, 1),
        (180.0, 8.0, 2),
        (120.0, 8.0, 3),
        (90.0, 8.0, 4),
        (60.0, 6.0, 5),
        (150.0, 3.0, 6),
        (30.0, 2.0, 7),
        (45.0, 2.0, 8),
        (135.0, 2.0, 9),
        (72.0, 2.0, 10),
        (144.0, 2.0, 11),
    ];
    let mut code = 0;
    for (offset, orb, matches_code) in rules {
        if matches_angle(focal, other + offset, orb) {
            code = matches_code
        }
    }
    code
}
pub fn original_h(longitudes: [f64; 13]) -> Result<OriginalAspectMatrix, String> {
    if longitudes.iter().any(|n| !n.is_finite()) {
        return Err("Nonfinite original aspect input".into());
    }
    let mut matrix = [[0; 13]; 13];
    for row in 0..13 {
        for col in 0..13 {
            if row != col {
                matrix[row][col] = code_for_pair(longitudes[col], longitudes[row])
            }
        }
    }
    Ok(OriginalAspectMatrix {
        codes: matrix,
        source: "Original APK main.H directed angle-code matrix".into(),
    })
}
pub const ORIGINAL_D0_TABLE: [[u8; 9]; 9] = [
    [3, 3, 1, 2, 2, 3, 2, 3, 2],
    [3, 3, 2, 1, 2, 3, 2, 3, 3],
    [3, 0, 2, 0, 1, 3, 2, 3, 0],
    [2, 2, 0, 2, 3, 2, 3, 2, 1],
    [0, 0, 2, 0, 2, 0, 1, 0, 0],
    [2, 1, 3, 0, 0, 2, 3, 2, 2],
    [2, 2, 3, 0, 0, 2, 0, 1, 0],
    [0, 2, 3, 3, 3, 1, 0, 2, 2],
    [1, 0, 0, 3, 0, 0, 0, 0, 0],
];
/// Table from original APK main.D0, independent of modern reference aspects.
pub fn original_d0(planet_code: u8, longitude: f64) -> u8 {
    if !(1..=9).contains(&planet_code) || !longitude.is_finite() || longitude < 0.0 {
        return 0;
    }
    let mansion = ((longitude / 13.333333333333334) as usize) % 27;
    ORIGINAL_D0_TABLE[(planet_code - 1) as usize][mansion % 9]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_orbs() {
        assert!(!original_f(0.0, 352.0));
        assert!(original_f(0.0, 352.001));
        assert!(!original_f(0.0, 351.99));
        assert!(!original_e(0.0, 354.0));
        assert!(original_e(0.0, 354.001));
        assert!(!original_d(0.0, 357.0));
        assert!(original_d(0.0, 357.001));
        assert!(!original_c(0.0, 358.0));
        assert!(original_c(0.0, 358.001));
    }
    #[test]
    fn g_codes() {
        let a = [
            0.0, 0.0, 180.0, 240.0, 270.0, 300.0, 210.0, 330.0, 315.0, 225.0, 288.0, 216.0, 100.0,
        ];
        let got = original_g(a).unwrap();
        assert_eq!(&got.codes[0..12], &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }
    #[test]
    fn d0_table() {
        assert_eq!(original_d0(1, 0.0), 3);
        assert_eq!(original_d0(4, 13.4), 2);
        assert_eq!(original_d0(11, 0.0), 0);
    }
    #[derive(serde::Deserialize)]
    struct JavaCase {
        input: [f64; 13],
        expected: [u8; 13],
    }
    #[test]
    fn original_g_matches_120_java_oracle_vectors() {
        let data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/original_g_oracle.json"
        ));
        let cases: Vec<JavaCase> = serde_json::from_str(data).unwrap();
        assert_eq!(cases.len(), 120);
        for (i, c) in cases.iter().enumerate() {
            let actual = original_g(c.input).unwrap();
            assert_eq!(
                actual.codes, c.expected,
                "original Java case {i}: {:?}",
                c.input
            );
        }
    }
    #[derive(serde::Deserialize)]
    struct JavaMatrixCase {
        input: [f64; 13],
        expected: [[u8; 13]; 13],
    }
    #[test]
    fn original_h_matches_42_java_oracle_matrices() {
        let data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/original_h_oracle.json"
        ));
        let cases: Vec<JavaMatrixCase> = serde_json::from_str(data).unwrap();
        assert_eq!(cases.len(), 42);
        for (i, c) in cases.iter().enumerate() {
            let got = original_h(c.input).unwrap();
            assert_eq!(got.codes, c.expected, "original H Java case {i}");
        }
    }
    #[test]
    fn nan_rejected() {
        let mut a = [0.0; 13];
        a[6] = f64::NAN;
        assert!(original_g(a).is_err())
    }
}

//! Recovered exact pure numeric mappings from original MOTAA APK main.X0/Y0.
//! The six outputs and 30 sentinel preserve original Java behavior.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalMansionMap {
    pub angle: f64,
    pub x0: [i32; 6],
    pub y0: [i32; 6],
    pub m0: [i32; 9],
    pub positions: crate::original_positions::OriginalPositionDiagnostic,
    pub source: String,
}
pub const ORIGINAL_Y0: [[i32; 6]; 27] = [
    [0, 9, 15, 17, 20, 21],
    [1, 10, 16, 18, 30, 22],
    [2, 11, 17, 19, 21, 23],
    [3, 12, 18, 20, 22, 24],
    [4, 13, 19, 30, 23, 25],
    [5, 14, 20, 21, 24, 26],
    [6, 15, 30, 22, 25, 0],
    [7, 16, 21, 23, 26, 1],
    [8, 17, 22, 24, 0, 2],
    [9, 18, 23, 25, 1, 3],
    [10, 19, 24, 26, 2, 4],
    [11, 20, 25, 0, 3, 5],
    [12, 30, 26, 1, 4, 6],
    [13, 21, 0, 2, 5, 7],
    [14, 22, 1, 3, 6, 8],
    [15, 23, 2, 4, 7, 9],
    [16, 24, 3, 5, 8, 10],
    [17, 25, 4, 6, 9, 11],
    [18, 26, 5, 7, 10, 12],
    [19, 0, 6, 8, 11, 13],
    [20, 1, 7, 9, 12, 14],
    [21, 3, 9, 11, 14, 16],
    [22, 4, 10, 12, 15, 17],
    [23, 5, 11, 13, 16, 18],
    [24, 6, 12, 14, 17, 19],
    [25, 7, 13, 15, 18, 20],
    [26, 8, 14, 16, 19, 30],
];
pub fn original_x0(angle: f64) -> [i32; 6] {
    if !angle.is_finite() {
        return [0; 6];
    }
    let i = (angle / 13.333333333333334_f64) as i32;
    let mut out = [i, i + 9, i + 15, i + 17, i + 20, i + 22];
    for code in &mut out {
        if *code >= 27 {
            *code -= 27
        }
    }
    out
}
pub fn original_y0(angle: f64) -> [i32; 6] {
    if !angle.is_finite() {
        return [0; 6];
    }
    let i = (angle / 13.333333333333334_f64) as i32;
    if (0..27).contains(&i) {
        ORIGINAL_Y0[i as usize]
    } else {
        [0; 6]
    }
}
pub fn calculate(angle: f64) -> Result<OriginalMansionMap, String> {
    if !angle.is_finite() {
        return Err("Angle must be finite".into());
    }
    Ok(OriginalMansionMap {
        angle,
        x0: original_x0(angle),
        y0: original_y0(angle),
        m0: crate::original_m0::original_m0(angle),
        positions: crate::original_positions::calculate(angle)?,
        source: "Original MOTAA APK main.X0/Y0, 116 Java oracle cases".into(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(serde::Deserialize)]
    struct Case {
        mode: String,
        angle: f64,
        expected: [i32; 6],
    }
    #[test]
    fn oracle_116_cases() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/original_nakshatra_mapping.json"
        ));
        let cases: Vec<Case> = serde_json::from_str(source).unwrap();
        assert_eq!(cases.len(), 116);
        for (i, c) in cases.iter().enumerate() {
            let got = if c.mode == "X" {
                original_x0(c.angle)
            } else {
                original_y0(c.angle)
            };
            assert_eq!(got, c.expected, "Java oracle case {i}");
        }
    }
    #[test]
    fn preserves_special_30() {
        assert_eq!(original_y0(14.0)[4], 30)
    }
    #[test]
    fn positions_embedded_in_original_mansion_diagnostic() {
        let actual = calculate(100.0).unwrap();
        assert_eq!(actual.positions.angle, actual.angle);
        assert_eq!(
            actual.positions.b0,
            crate::original_positions::original_b0(100.0)
        );
        assert_eq!(
            actual.positions.x0,
            crate::original_positions::original_x0(100.0)
        );
        let json = serde_json::to_value(&actual).unwrap();
        assert!(json["positions"]["b0"].as_array().unwrap().len() == 10);
        assert!(json["positions"]["x0"].as_array().unwrap().len() == 5);
        assert!(json["positions"]["zodiacName"].as_str().is_some());
    }
}

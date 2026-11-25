//! APK-derived coordinate/mansion helpers: main.B0, V0, W0, x0, y, y0, z.
//! Source: research/motaa-v31.1/decompiled/sources/motaa/myanmar/main.java
//! Evidence: tests/oracles/OriginalPositionOracle.java + 254 Java fixtures.
//! Each is a standalone compatibility primitive, NOT an ephemeris or full chart.

use serde::{Deserialize, Serialize};

mod tables {
    include!("original_position_tables.rs");
}

/// Ten angular reference positions for a given source mansion.
/// The APK's sector 7 / index 6 value is 281.0, NOT a regular multiple of
/// 13.3333333. Full literal Java results preserve that exception.
pub fn original_b0(angle: f64) -> [f64; 10] {
    let i = (angle / 13.333333333333334_f64) as i32;
    if (0..27).contains(&i) {
        tables::B0_VALUES[i as usize]
    } else {
        [0.0; 10]
    }
}

/// APK main.V0: original Burmese zodiac name, original spelling preserved.
pub fn original_v0(angle: f64) -> &'static str {
    let i = (angle / 30.0) as i32;
    if (0..12).contains(&i) {
        tables::ZODIAC_BURMESE[i as usize]
    } else {
        ""
    }
}

/// APK main.W0: Java signed remainder; do not use rem_euclid.
pub fn original_w0(angle: f64) -> i32 {
    ((angle / 30.0) as i32) % 12
}

/// APK main.y: Java truncating integer cast on mansion divisions.
pub fn original_y(angle: f64) -> i32 {
    (angle / 13.333333333333334_f64) as i32
}

/// APK main.z: bracketed original Burmese mansion label.
/// Out-of-range sector yields literal "[]", not a placeholder mansion.
pub fn original_z(angle: f64) -> String {
    let i = original_y(angle);
    if (0..27).contains(&i) {
        format!("[{}]", tables::NAKSHATRA_BURMESE_APK[i as usize])
    } else {
        "[]".to_string()
    }
}

/// APK main.x0: [mansion 0-based index, pada 0-based index, minute,
/// second, subsecond], preserving 8-decimal boundaries and truncated casts.
/// Original classification is not conventional modulo-360 normalization.
pub fn original_x0(angle: f64) -> [i32; 5] {
    let mut result = [0_i32; 5];
    let mut within = if angle < 0.0 || angle >= 13.33333333 {
        0.0
    } else {
        angle
    };
    for sector in 1..27 {
        if angle >= tables::X0_BOUNDS[sector] && angle < tables::X0_BOUNDS[sector + 1] {
            result[0] = sector as i32;
            within = angle - tables::X0_BOUNDS[sector];
        }
    }

    if (0.0..3.3333333333).contains(&within) {
        result[1] = 0;
    }
    if (3.3333333333..6.666666666).contains(&within) {
        result[1] = 1;
        within -= 3.3333333333;
    }
    if (6.666666666..10.0).contains(&within) {
        result[1] = 2;
        within -= 6.666666666;
    }
    if (10.0..13.33333333).contains(&within) {
        result[1] = 3;
        within -= 10.0;
    }

    let value = (within * 60.0) / 13.33333333;
    result[2] = value as i32;
    let seconds = (value - (result[2] as f64)) * 60.0;
    result[3] = seconds as i32;
    result[4] = ((seconds - (result[3] as f64)) * 60.0) as i32;
    result
}

/// APK main.y0: [zodiac index, within-sign degrees, arcminutes,
/// arcseconds, fractional arcseconds], all Java-style truncated.
pub fn original_y0(angle: f64) -> [i32; 5] {
    let d3 = angle / 30.0;
    let a = d3 as i32;
    let d4 = (d3 - (a as f64)) * 30.0;
    let b = d4 as i32;
    let d5 = (d4 - (b as f64)) * 60.0;
    let c = d5 as i32;
    let d6 = (d5 - (c as f64)) * 60.0;
    let d = d6 as i32;
    [a, b, c, d, ((d6 - (d as f64)) * 60.0) as i32]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalPositionDiagnostic {
    pub angle: f64,
    pub b0: [f64; 10],
    pub x0: [i32; 5],
    pub y: i32,
    pub y0: [i32; 5],
    pub zodiac_name: String,
    pub zodiac_code: i32,
    pub nakshatra_name: String,
    pub source: String,
}
pub fn calculate(angle: f64) -> Result<OriginalPositionDiagnostic, String> {
    if !angle.is_finite() {
        return Err("Longitude must be finite".into());
    }
    Ok(OriginalPositionDiagnostic {
        angle,
        b0: original_b0(angle),
        x0: original_x0(angle),
        y: original_y(angle),
        y0: original_y0(angle),
        zodiac_name: original_v0(angle).into(),
        zodiac_code: original_w0(angle),
        nakshatra_name: original_z(angle),
        source: "Isolated original APK main.B0/V0/W0/x0/y/y0/z; not a verified planetary ephemeris"
            .into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Fixture {
        cases: Vec<Case>,
    }
    #[derive(Deserialize)]
    struct Case {
        angle: f64,
        b0: [f64; 10],
        x0: [i32; 5],
        y: i32,
        y0: [i32; 5],
        z: String,
        v0: String,
        w0: i32,
    }

    #[test]
    fn matches_original_java_at_254_angles() {
        let cases: Fixture = serde_json::from_str(include_str!(
            "../../../tests/fixtures/original_positions.json"
        ))
        .unwrap();
        assert_eq!(cases.cases.len(), 254);
        for case in cases.cases {
            let actual = calculate(case.angle).unwrap();
            for i in 0..10 {
                assert!(
                    (actual.b0[i] - case.b0[i]).abs() < 1e-11,
                    "B0 input {} index {}: {} != {}",
                    case.angle,
                    i,
                    actual.b0[i],
                    case.b0[i]
                );
            }
            assert_eq!(actual.x0, case.x0, "x0 input {}", case.angle);
            assert_eq!(actual.y, case.y, "y input {}", case.angle);
            assert_eq!(actual.y0, case.y0, "y0 input {}", case.angle);
            assert_eq!(actual.nakshatra_name, case.z, "z input {}", case.angle);
            assert_eq!(actual.zodiac_name, case.v0, "V0 input {}", case.angle);
            assert_eq!(actual.zodiac_code, case.w0, "W0 input {}", case.angle);
        }
    }

    #[test]
    fn preserves_original_quirks() {
        assert_eq!(original_b0(7.0 * 13.333333333333334 + 5.0)[6], 281.0);
        assert_eq!(original_w0(-45.0), -1); // Java signed remainder
        assert_eq!(original_z(360.0), "[]");
        assert!(original_v0(360.0).is_empty());
        assert_eq!(original_y(-0.1), 0); // Java truncates toward zero
        assert!(calculate(f64::NAN).is_err());
        assert!(calculate(f64::INFINITY).is_err());
    }
}

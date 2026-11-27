//! Isolated exact APK main.D0, U0 and s0 numerical helpers.
//! Java-compatible indexing, one-time +360 wrap, boundary guards and rounding.
use serde::{Deserialize, Serialize};

pub fn original_d0(code: u8, angle: f64) -> Result<i32, String> {
    if !angle.is_finite() {
        return Err("D0 angle must be finite".into());
    }
    if !(1..=9).contains(&code) {
        return Ok(0);
    }
    let i = ((angle / 13.333333333333334_f64) as i32) % 27;
    if i < 0 {
        return Err("Original D0 would index a negative Java array offset".into());
    }
    Ok(i32::from(
        crate::original_aspects::ORIGINAL_D0_TABLE[(code - 1) as usize][(i % 9) as usize],
    ))
}
fn java_round(n: f64) -> i32 {
    (n + 0.5).floor() as i32
}
pub fn original_u0(positions: &[f64; 10], source: usize, target: usize) -> Result<i32, String> {
    if source >= 10 || target >= 10 {
        return Err("U0 index outside 0..9".into());
    }
    if positions.iter().any(|n| !n.is_finite()) {
        return Err("U0 inputs must be finite".into());
    }
    let mut d = positions[target] - positions[source];
    if d < 0.0 {
        d += 360.0;
    }
    let mut score = 0;
    if [1, 2, 4, 6].contains(&source) {
        if (30.0..60.0).contains(&d) {
            d -= 30.0;
            score = java_round(d / 2.0);
        }
        if (60.0..90.0).contains(&d) {
            d -= 60.0;
            score = java_round(d) + 15;
        }
        if (90.0..120.0).contains(&d) {
            d = 120.0 - d;
            score = java_round(d / 2.0) + 30;
        }
        if (120.0..150.0).contains(&d) {
            d = 150.0 - d;
            score = java_round(d);
        }
        if (150.0..180.0).contains(&d) {
            d -= 150.0;
            score = java_round(d * 2.0);
        }
        if (180.0..300.0).contains(&d) {
            d = 300.0 - d;
            score = java_round(d / 2.0);
        }
    }
    if source == 3 {
        if (30.0..60.0).contains(&d) {
            d -= 30.0;
            score = java_round(d / 2.0);
        }
        if (60.0..90.0).contains(&d) {
            d -= 60.0;
            score = java_round(d / 2.0 + d + 15.0);
        }
        if (90.0..120.0).contains(&d) {
            d -= 90.0;
            score = 60 - java_round(d);
        }
        if (120.0..150.0).contains(&d) {
            d = 150.0 - d;
            score = java_round(d);
        }
        if (150.0..180.0).contains(&d) {
            d -= 150.0;
            score = java_round(d * 2.0);
        }
        if (180.0..210.0).contains(&d) {
            score = 60;
        }
        if (210.0..240.0).contains(&d) {
            score = 270 - java_round(d);
        }
        if (240.0..300.0).contains(&d) {
            d = 300.0 - d;
            score = java_round(d / 2.0);
        }
    }
    if source == 7 {
        if (30.0..60.0).contains(&d) {
            d -= 30.0;
            score = java_round(d * 2.0);
        }
        if (60.0..90.0).contains(&d) {
            d -= 60.0;
            score = 60 - java_round(d / 2.0);
        }
        if (90.0..120.0).contains(&d) {
            d = 120.0 - d;
            score = java_round(d / 2.0) + 30;
        }
        if (120.0..150.0).contains(&d) {
            d = 150.0 - d;
            score = java_round(d);
        }
        if (150.0..180.0).contains(&d) {
            d -= 150.0;
            score = java_round(d * 2.0);
        }
        if (180.0..240.0).contains(&d) {
            d = 300.0 - d;
            score = java_round(d / 2.0);
        }
        if (240.0..270.0).contains(&d) {
            d -= 240.0;
            score = java_round(d + 30.0);
        }
        if (270.0..300.0).contains(&d) {
            d = 300.0 - d;
            score = java_round(d * 2.0);
        }
    }
    if ![5, 8, 9].contains(&source) {
        return Ok(score);
    }
    if (30.0..60.0).contains(&d) {
        d -= 30.0;
        score = java_round(d / 2.0);
    }
    if (60.0..90.0).contains(&d) {
        d -= 60.0;
        score = java_round(d) + 15;
    }
    if (90.0..120.0).contains(&d) {
        d -= 90.0;
        score = java_round(d / 2.0) + 45;
    }
    if (120.0..150.0).contains(&d) {
        d -= 120.0;
        score = 60 - java_round(d * 2.0);
    }
    if (150.0..180.0).contains(&d) {
        d -= 150.0;
        score = java_round(d * 2.0);
    }
    if (180.0..210.0).contains(&d) {
        d = 300.0 - d;
        score = java_round(d / 2.0);
    }
    if (210.0..240.0).contains(&d) {
        d -= 210.0;
        score = java_round(d / 2.0) + 45;
    }
    if (240.0..270.0).contains(&d) {
        d -= 240.0;
        score = java_round((30.0 - d) * 1.5) + 15;
    }
    if (270.0..300.0).contains(&d) {
        return Ok(java_round((300.0 - d) / 2.0));
    }
    Ok(score)
}
pub fn original_s0(angle: f64, b: &[f64; 13]) -> Result<u8, String> {
    if !angle.is_finite() || b.iter().any(|n| !n.is_finite()) {
        return Err("s0 must receive finite inputs".into());
    }
    let mut class = 0_u8;
    for i in 1..12 {
        if angle >= b[i] && angle < b[i + 1] {
            class = i as u8;
        }
    }
    if angle >= b[12] && angle < b[1] {
        class = 12;
    }
    let mut value = 1_u8;
    if b[1] <= b[2] || ((b[1] > angle || angle >= 360.0) && (b[2] <= angle || angle < 0.0)) {
        value = class;
    }
    for i in 2..=11 {
        if !(b[i] <= b[i + 1]
            || ((b[i] > angle || angle >= 360.0) && (b[i + 1] <= angle || angle < 0.0)))
        {
            value = i as u8;
        }
    }
    if b[12] > b[1] {
        if b[12] <= angle && angle < 360.0 {
            return Ok(12);
        }
        if b[1] > angle && angle >= 0.0 {
            return Ok(12);
        }
    }
    Ok(value)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalStrengthDiagnostic {
    pub source_positions: [f64; 10],
    pub intensity_matrix: [[i32; 9]; 9],
    pub d0_code: u8,
    pub d0_angle: f64,
    pub d0_class: i32,
    pub house_angle: f64,
    pub house_boundaries: [f64; 12],
    pub house_sector: u8,
    pub source: String,
}
pub fn calculate(
    positions: [f64; 10],
    d0_code: u8,
    d0_angle: f64,
    house_angle: f64,
    house_boundaries: [f64; 12],
) -> Result<OriginalStrengthDiagnostic, String> {
    if positions.iter().any(|n| !n.is_finite()) {
        return Err("Provide ten finite longitudes".into());
    }
    if !(1..=9).contains(&d0_code) {
        return Err("D0 code must be 1..9".into());
    }
    let mut boundaries = [0.0; 13];
    boundaries[1..].copy_from_slice(&house_boundaries);
    let house_sector = original_s0(house_angle, &boundaries)?;
    let d0_class = original_d0(d0_code, d0_angle)?;
    let mut intensity_matrix = [[0; 9]; 9];
    for from in 1..=9 {
        for to in 1..=9 {
            intensity_matrix[from - 1][to - 1] = original_u0(&positions, from, to)?;
        }
    }
    Ok(OriginalStrengthDiagnostic{
  source_positions:positions,intensity_matrix,d0_code,d0_angle,d0_class,
  house_angle,house_boundaries,house_sector,
  source:"Original APK main.U0/D0/s0 numeric compatibility; original body mapping not established".into(),
 })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Fixture {
        u0: Vec<U>,
        d0: Vec<D>,
        s0: Vec<S>,
    }
    #[derive(Deserialize)]
    struct U {
        source: usize,
        target: usize,
        delta: f64,
        score: i32,
    }
    #[derive(Deserialize)]
    struct D {
        code: u8,
        angle: f64,
        #[serde(rename = "class")]
        class: i32,
    }
    #[derive(Deserialize)]
    struct S {
        pattern: usize,
        angle: f64,
        house: u8,
    }
    #[test]
    fn original_java_differential() {
        let cases: Fixture = serde_json::from_str(include_str!(
            "../../../tests/fixtures/original_strengths.json"
        ))
        .unwrap();
        assert_eq!(cases.u0.len(), 5400);
        assert_eq!(cases.d0.len(), 457);
        assert_eq!(cases.s0.len(), 305);
        for c in cases.u0 {
            let mut positions = [100.0; 10];
            if c.source != c.target {
                positions[c.target] = 100.0 + c.delta;
            }
            assert_eq!(
                original_u0(&positions, c.source, c.target).unwrap(),
                c.score,
                "U0 source={} target={} delta={}",
                c.source,
                c.target,
                c.delta
            );
        }
        for c in cases.d0 {
            assert_eq!(
                original_d0(c.code, c.angle).unwrap(),
                c.class,
                "D0 code={} angle={}",
                c.code,
                c.angle
            );
        }
        for c in cases.s0 {
            let mut b = [0.0; 13];
            for (i, value) in b.iter_mut().enumerate().skip(1) {
                *value = match c.pattern {
                    0 => (i - 1) as f64 * 30.0,
                    1 => ((i - 1) as f64 * 30.0 + 45.0) % 360.0,
                    2 => ((i - 1) as f64 * 30.0 + 190.0) % 360.0,
                    3 => ((i - 1) as f64 * 30.0 + 359.0) % 360.0,
                    4 => (i - 1) as f64 * 30.0 + 120.0,
                    _ => panic!("Bad test pattern"),
                };
            }
            assert_eq!(
                original_s0(c.angle, &b).unwrap(),
                c.house,
                "s0 pattern={} angle={}",
                c.pattern,
                c.angle
            );
        }
    }
    #[test]
    fn native_input_validation() {
        assert!(original_u0(&[0.0; 10], 10, 1).is_err());
        assert!(original_u0(&[f64::NAN; 10], 1, 2).is_err());
        assert!(original_d0(1, -30.0).is_err());
        assert_eq!(original_d0(0, -30.0).unwrap(), 0);
        assert!(original_s0(f64::NAN, &[0.0; 13]).is_err());
        assert!(calculate([0.0; 10], 2, 0.0, 0.0, [0.0; 12]).is_ok());
    }
}

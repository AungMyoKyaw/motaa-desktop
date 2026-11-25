//! Verified primitive numerical helpers from the supplied MOTAA APK.
//!
//! These reproduce individual arithmetic methods, not whole-screen geometry.
//! Decompiled evidence:
//! - main.G0: main.java:2666
//! - main.H0: main.java:8037
//! - main.L0: main.java:8137
//! - main.O0: main.java:8655
//! - main.P0: main.java:8664 (with AbstractC0273n.f)
//!
//! Java oracle: tests/oracles/OriginalPrimitiveOracle.java
//! Differential fixtures: tests/fixtures/original_primitives.json
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryRequest {
    pub angle: f64,
    pub target: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub radius: f64,
    pub year: i32,
    pub month: u8,
    pub day: u8,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalGeometry {
    pub angle: f64,
    pub target: f64,
    pub x: f32,
    pub y: f32,
    pub mansion_codes: [i32; 9],
    pub degree_label: String,
    pub minute_label: String,
    pub original_burmese_digits: String,
    pub sector_code: i32,
    pub julian_day_number: f64,
    pub source: String,
}

#[allow(clippy::neg_multiply)] // Preserve the Java expression's evaluation order for numerical parity.
pub fn original_g0(angle: f64, center: f64, radius: f64) -> f32 {
    // Preserve the original split at 270° and its f32 return precision.
    let mut d = (if angle <= 270.0 {
        angle + 90.0
    } else {
        angle - 270.0
    }) - 15.0;
    if d < 0.0 {
        d = center - ((d.to_radians().cos() * radius) * -1.0);
    } else if d >= 0.0 {
        d = (d.to_radians().cos() * radius) + center;
    }
    d as f32
}
#[allow(clippy::neg_multiply)] // Mirror the original ternary branch arithmetic exactly.
pub fn original_h0(angle: f64, center: f64, radius: f64) -> f32 {
    let d = (if angle <= 270.0 {
        angle + 90.0
    } else {
        angle - 270.0
    }) - 15.0;
    let sin = d.to_radians().sin() * radius;
    (if d >= 0.0 {
        center - sin
    } else {
        center + sin * -1.0
    }) as f32
}
/// Original Java remainder uses signed %, unlike euclidean modulus.
pub fn original_l0(angle: f64) -> [i32; 9] {
    let base = (angle / 13.333333333333334_f64) as i32;
    let offsets = [0, 4, 7, 13, 17, 20, 21, 22, 23];
    offsets.map(|n| {
        if n == 0 {
            base
        } else {
            base.wrapping_add(n) % 27
        }
    })
}
/// Original 12-degree angular difference uses a single wrap, NOT mathematical rem_euclid.
pub fn original_o0(focal: f64, target: f64) -> i32 {
    let mut d = target - focal;
    if d < 0.0 {
        d += 360.0;
    }
    (d / 12.0) as i32
}
/// Direct port of main.P0. Inputs are (day, month, year), not year/month/day.
/// This preserves the APK Gregorian transition logic and BC-year adjustment.
#[allow(clippy::assign_op_pattern)] // Preserve Java addition order near Julian-day cutovers.
pub fn original_p0(day: i32, month: i32, year: i32) -> f64 {
    let mut y = if year < 0 { year + 1 } else { year };
    if year < 0 {
        y += 1;
    }
    let m = if month > 2 {
        month + 1
    } else {
        y -= 1;
        month + 13
    };
    let yd = y as f64;
    let mut jd = (m as f64 * 30.6001).floor() + (365.25 * yd).floor() + day as f64 + 1720995.0;
    if ((year * 12 + month) * 31 + day) >= 588829 {
        let century = (yd * 0.01) as i32;
        // AbstractC0273n.f(d2,d3,d4,d5) = d2*d3 + d4 + d5.
        jd = (century as f64 * 0.25) + (2 - century) as f64 + jd;
    }
    jd.floor()
}
pub fn calculate(request: &GeometryRequest) -> Result<OriginalGeometry, String> {
    if [
        request.angle,
        request.target,
        request.center_x,
        request.center_y,
        request.radius,
    ]
    .iter()
    .any(|v| !v.is_finite())
    {
        return Err("All geometric inputs must be finite".into());
    }
    if request.radius.abs() > 100_000.0
        || request.center_x.abs() > 1_000_000.0
        || request.center_y.abs() > 1_000_000.0
        || request.angle.abs() > 100_000.0
        || request.target.abs() > 100_000.0
    {
        return Err("Geometry inputs exceed safe inspector range".into());
    }
    if request.day == 0 || request.day > crate::days_in_month(request.year, request.month) {
        return Err("Invalid Gregorian calendar date".into());
    }
    if !(-4712..=9999).contains(&request.year) {
        return Err("Year must be between -4712 and 9999".into());
    }
    Ok(OriginalGeometry {
        angle: request.angle,
        target: request.target,
        x: original_g0(request.angle, request.center_x, request.radius),
        y: original_h0(request.angle, request.center_y, request.radius),
        mansion_codes: original_l0(request.angle),
        degree_label: crate::original_text::original_k(request.angle),
        minute_label: crate::original_text::original_m(request.angle),
        original_burmese_digits: crate::original_text::original_i(
            &crate::original_text::original_l(request.angle),
        ),
        sector_code: original_o0(request.angle, request.target),
        julian_day_number: original_p0(request.day as i32, request.month as i32, request.year),
        source: "APK main.G0/H0/L0/O0/P0 — numeric primitives, NOT complete chart drawing parity"
            .into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Oracle {
        angles: Vec<Angle>,
        dates: Vec<Date>,
    }
    #[derive(Deserialize)]
    struct Angle {
        angle: f64,
        target: f64,
        center_x: f64,
        center_y: f64,
        radius: f64,
        g0: f64,
        h0: f64,
        o0: i32,
        l0: [i32; 9],
    }
    #[derive(Deserialize)]
    struct Date {
        day: i32,
        month: i32,
        year: i32,
        jdn: f64,
    }
    fn fixture() -> Oracle {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/original_primitives.json"
        )))
        .expect("Java oracle valid JSON")
    }
    #[test]
    fn java_geometry_oracle() {
        let fixture = fixture();
        assert_eq!(fixture.angles.len(), 138);
        for (i, a) in fixture.angles.iter().enumerate() {
            let x = original_g0(a.angle, a.center_x, a.radius) as f64;
            let y = original_h0(a.angle, a.center_y, a.radius) as f64;
            assert!(
                (x - a.g0).abs() < 0.0001,
                "Java G0 case {i}: {x} != {}",
                a.g0
            );
            assert!(
                (y - a.h0).abs() < 0.0001,
                "Java H0 case {i}: {y} != {}",
                a.h0
            );
            assert_eq!(original_l0(a.angle), a.l0, "Java L0 case {i}");
            assert_eq!(original_o0(a.angle, a.target), a.o0, "Java O0 case {i}");
        }
    }
    #[test]
    fn java_julian_oracle() {
        let cases = fixture().dates;
        assert_eq!(cases.len(), 14);
        for date in cases {
            let got = original_p0(date.day, date.month, date.year);
            assert_eq!(
                got, date.jdn,
                "Java P0 {:04}-{:02}-{:02}",
                date.year, date.month, date.day
            );
        }
    }
    #[test]
    fn preserve_original_anomalies() {
        assert_eq!(original_l0(-360.0)[0], -27);
        assert_eq!(original_l0(f64::MAX)[1], i32::MAX.wrapping_add(4) % 27);
        assert_eq!(original_o0(0.0, 360.0), 30);
        assert_eq!(original_o0(720.0, 0.0), -30); // single wrap only
    }
    #[test]
    fn safe_inspector_rejects_invalid_input() {
        let mut r = GeometryRequest {
            angle: 270.0,
            target: 360.0,
            center_x: 256.0,
            center_y: 256.0,
            radius: 157.0,
            year: 2024,
            month: 4,
            day: 17,
        };
        assert!(calculate(&r).is_ok());
        r.radius = f64::NAN;
        assert!(calculate(&r).is_err());
        r.radius = 10.0;
        r.month = 2;
        r.day = 30;
        assert!(calculate(&r).is_err());
    }
}

//! Exact MOTAA V31.1 chart-label color rules from APK main.C0/N/O/x.
//! Original Java extracted to tests/oracles/OriginalColorOracle.java.
//! This is NOT a planetary calculation; inputs must be separately supplied.
//! Do not normalize angles: original code truncates degrees / 30 toward zero.
use serde::{Deserialize, Serialize};

/// main.C0: original classification by label identifier and zodiac sector.
/// Columns are 30-degree sectors 0..11, rows are the original codes 1..9.
pub const ORIGINAL_C0: [[u8; 12]; 9] = [
    [4, 3, 2, 2, 1, 2, 5, 2, 2, 3, 3, 2],
    [2, 4, 3, 1, 2, 3, 3, 5, 2, 3, 3, 2],
    [1, 0, 0, 5, 2, 0, 0, 1, 2, 4, 3, 2],
    [3, 2, 1, 2, 0, 4, 2, 3, 3, 2, 2, 5],
    [2, 0, 0, 4, 2, 0, 0, 2, 1, 5, 0, 1],
    [0, 1, 2, 0, 3, 5, 1, 0, 3, 2, 2, 4],
    [5, 2, 0, 0, 3, 0, 4, 0, 0, 1, 1, 0],
    [3, 2, 2, 3, 3, 2, 2, 3, 0, 2, 2, 0],
    [0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0],
];
pub fn original_c0(code: u8, longitude: f64) -> u8 {
    if !(1..=9).contains(&code) {
        return 0;
    }
    // Java double-to-int narrowing truncates toward zero, like Rust's cast.
    let sector = (longitude / 30.0) as i32;
    if !(0..12).contains(&sector) {
        return 0;
    }
    ORIGINAL_C0[(code - 1) as usize][sector as usize]
}

/// main.x: quirky sign-dependent remapping of longitudes for main.O.
pub fn original_x(angle: f64) -> f64 {
    let div = 3.3333333333333335_f64;
    let mut d3 = if angle < 0.0 || angle >= 30.0 {
        0.0
    } else {
        (angle / div) * 30.0
    };
    if (120.0..150.0).contains(&angle) {
        d3 = ((angle - 120.0) / div) * 30.0;
    }
    if (240.0..270.0).contains(&angle) {
        d3 = ((angle - 240.0) / div) * 30.0;
    }
    if (90.0..120.0).contains(&angle) {
        d3 = ((angle - 90.0) / div) * 30.0 + 90.0;
    }
    if (210.0..240.0).contains(&angle) {
        d3 = ((angle - 210.0) / div) * 30.0 + 90.0;
    }
    if angle >= 330.0 {
        d3 = ((angle - 330.0) / div) * 30.0 + 90.0;
    }
    if (180.0..210.0).contains(&angle) {
        d3 = ((angle - 180.0) / div) * 30.0 + 180.0;
    }
    if (60.0..90.0).contains(&angle) {
        d3 = ((angle - 60.0) / div) * 30.0 + 180.0;
    }
    if (300.0..330.0).contains(&angle) {
        d3 = ((angle - 300.0) / div) * 30.0 + 180.0;
    }
    if (270.0..300.0).contains(&angle) {
        d3 = ((angle - 270.0) / div) * 30.0 + 270.0;
    }
    if (30.0..60.0).contains(&angle) {
        d3 = ((angle - 30.0) / div) * 30.0 + 270.0;
    }
    if (150.0..180.0).contains(&angle) {
        d3 = ((angle - 150.0) / div) * 30.0 + 270.0;
    }
    if d3 < 360.0 {
        d3
    } else if d3 >= 360.0 {
        d3 - 360.0
    } else {
        0.0
    }
}

/// These symbols are the literal labels passed to main.N/O. The APK uses
/// Burmese digit zero (၀) as identifier 7; not Burmese consonant Wa (ဝ).
pub const SYMBOLS: [&str; 9] = ["၁", "၂", "၃", "၄", "၅", "၆", "၀", "၈", "၉"];

pub fn original_symbol_code(symbol: &str) -> u8 {
    SYMBOLS
        .iter()
        .position(|&value| value == symbol)
        .map(|index| (index + 1) as u8)
        .unwrap_or(0)
}

/// The original pair is [background ARGB, foreground ARGB] as signed i32.
/// Background=0 means transparent. Exact RGB 76/153/0 matches Color.rgb.
pub fn original_color_for_class(class: u8) -> [i32; 2] {
    match class {
        1 => [0, -16776961],             // blue
        2 => [0, 0xff4c9900_u32 as i32], // green
        3 => [0, -65536],                // red
        4 => [-256, -16777216],          // yellow background
        5 => [-3355444, -16777216],      // pale gray background
        _ => [0, -16777216],             // black
    }
}

pub fn original_n(symbol: &str, longitude: f64) -> [i32; 2] {
    original_color_for_class(original_c0(original_symbol_code(symbol), longitude))
}
pub fn original_o(symbol: &str, longitude: f64) -> [i32; 2] {
    original_color_for_class(original_c0(
        original_symbol_code(symbol),
        original_x(longitude),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalColorEntry {
    pub symbol: String,
    pub index: u8,
    pub longitude: f64,
    pub class_n: u8,
    pub colors_n: [i32; 2],
    pub transformed_longitude: f64,
    pub class_o: u8,
    pub colors_o: [i32; 2],
}

pub fn calculate(original_longitudes: [f64; 9]) -> Result<Vec<OriginalColorEntry>, String> {
    if original_longitudes.iter().any(|x| !x.is_finite()) {
        return Err("All nine longitudes must be finite".into());
    }
    Ok(original_longitudes
        .iter()
        .enumerate()
        .map(|(i, &longitude)| {
            let index = (i + 1) as u8;
            let transformed = original_x(longitude);
            OriginalColorEntry {
                symbol: SYMBOLS[i].into(),
                index,
                longitude,
                class_n: original_c0(index, longitude),
                colors_n: original_n(SYMBOLS[i], longitude),
                transformed_longitude: transformed,
                class_o: original_c0(index, transformed),
                colors_o: original_o(SYMBOLS[i], longitude),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Fixture {
        c0: Vec<C0Case>,
        colors: Vec<ColorCase>,
        x: Vec<XCase>,
    }
    #[derive(Deserialize)]
    struct C0Case {
        code: u8,
        longitude: f64,
        result: u8,
    }
    #[derive(Deserialize)]
    struct ColorCase {
        symbol: String,
        longitude: f64,
        n: [i32; 2],
        o: [i32; 2],
    }
    #[derive(Deserialize)]
    struct XCase {
        longitude: f64,
        result: f64,
    }

    #[test]
    fn original_java_color_oracle() {
        let fixture: Fixture =
            serde_json::from_str(include_str!("../../../tests/fixtures/original_colors.json"))
                .unwrap();
        assert_eq!(
            (fixture.c0.len(), fixture.colors.len(), fixture.x.len()),
            (429, 405, 27)
        );
        for row in fixture.c0 {
            assert_eq!(
                original_c0(row.code, row.longitude),
                row.result,
                "C0 code {} at {}",
                row.code,
                row.longitude
            );
        }
        for row in fixture.colors {
            assert_eq!(
                original_n(&row.symbol, row.longitude),
                row.n,
                "N symbol {} at {}",
                row.symbol,
                row.longitude
            );
            assert_eq!(
                original_o(&row.symbol, row.longitude),
                row.o,
                "O symbol {} at {}",
                row.symbol,
                row.longitude
            );
        }
        for row in fixture.x {
            let actual = original_x(row.longitude);
            assert!(
                (actual - row.result).abs() < 1e-11,
                "x at {}: {} != {}",
                row.longitude,
                actual,
                row.result
            );
        }
    }
    #[test]
    fn c0_edges_and_unicode_digit_zero() {
        assert_eq!(original_c0(1, 359.999), 2);
        assert_eq!(original_c0(1, 360.0), 0);
        assert_eq!(original_c0(1, -0.1), 4); // Java integer truncation!
        assert_eq!(original_symbol_code("၀"), 7);
        assert_eq!(original_symbol_code("ဝ"), 0);
        assert_eq!(original_n("invalid", 10.0), [0, -16777216]);
        assert!(calculate([0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0]).is_ok());
        assert!(calculate([f64::NAN; 9]).is_err());
    }
}

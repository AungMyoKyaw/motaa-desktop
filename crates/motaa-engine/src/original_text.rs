//! Exact small text/angle formatting mappings recovered from MOTAA V31.1.
//!
//! Evidence: main.F0 (main.java:2191), I (8044), J0 (8049),
//! K (8054), L (8098), M (8143).
//! These intentionally preserve original quirks (notably zero -> Burmese Wa).
pub fn original_f0(code: i32) -> Option<&'static str> {
    match code {
        0 => Some("လ"),
        1 => Some("၁"),
        2 => Some("၂"),
        3 => Some("၃"),
        4 => Some("၄"),
        5 => Some("၅"),
        6 => Some("၆"),
        7 => Some("ဝ"),
        8 => Some("၈"),
        9 => Some("၉"),
        10 => Some("ဒ"),
        _ => None,
    }
}
pub fn original_i(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0' => 'ဝ',
            '1' => '၁',
            '2' => '၂',
            '3' => '၃',
            '4' => '၄',
            '5' => '၅',
            '6' => '၆',
            '7' => '၇',
            '8' => '၈',
            '9' => '၉',
            _ => c,
        })
        .collect()
}
pub fn original_j0(s: &str) -> &'static str {
    if s == "R" { "℞" } else { "" }
}
// The original does *not* wrap negative or >=360-degree inputs.
fn zodiac_segment(angle: f64) -> f64 {
    if !angle.is_finite() || !(0.0..360.0).contains(&angle) {
        return 0.0;
    }
    let sector = (angle / 30.0) as i32;
    angle - (sector * 30) as f64
}
pub fn original_k(angle: f64) -> String {
    format!("{}°", zodiac_segment(angle) as i32)
}
pub fn original_l(angle: f64) -> String {
    (zodiac_segment(angle) as i32).to_string()
}
pub fn original_m(angle: f64) -> String {
    let local = zodiac_segment(angle);
    (((local - local.trunc()) * 60.0) as i32).to_string()
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Oracle {
        angles: Vec<Angle>,
        codes: Vec<Code>,
        texts: Vec<Text>,
    }
    #[derive(Deserialize)]
    struct Angle {
        angle: f64,
        k: String,
        l: String,
        m: String,
    }
    #[derive(Deserialize)]
    struct Code {
        code: i32,
        f0: Option<String>,
    }
    #[derive(Deserialize)]
    struct Text {
        input: String,
        i: String,
        j0: String,
    }
    #[test]
    fn java_text_oracle() {
        let raw = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/original_text.json"
        ));
        let cases: Oracle = serde_json::from_str(raw).unwrap();
        assert_eq!(cases.angles.len(), 33);
        assert_eq!(cases.codes.len(), 14);
        assert_eq!(cases.texts.len(), 9);
        for c in cases.angles {
            assert_eq!(original_k(c.angle), c.k, "K at {}", c.angle);
            assert_eq!(original_l(c.angle), c.l, "L at {}", c.angle);
            assert_eq!(original_m(c.angle), c.m, "M at {}", c.angle);
        }
        for c in cases.codes {
            assert_eq!(original_f0(c.code), c.f0.as_deref(), "F0 at {}", c.code);
        }
        for c in cases.texts {
            assert_eq!(original_i(&c.input), c.i, "I for {}", c.input);
            assert_eq!(original_j0(&c.input), c.j0, "J0 for {}", c.input);
        }
    }
    #[test]
    fn preserves_unusual_zero_and_seven() {
        assert_eq!(original_i("07"), "ဝ၇");
        assert_eq!(original_f0(7), Some("ဝ"));
        assert_eq!(original_f0(0), Some("လ"));
        assert_eq!(original_j0("R"), "℞");
    }
}

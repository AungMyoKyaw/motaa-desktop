//! Generated exact constants from supplied MOTAA APK main.M0 (main.java:8199).
//! Run `python3 scripts/extract_m0.py` to regenerate.
//! Code 30 is an original sentinel, never silently replaced with 27.
pub const ORIGINAL_M0: [[i32; 9]; 27] = [
    [0, 4, 7, 13, 17, 20, 30, 21, 22],
    [1, 5, 8, 14, 18, 30, 21, 22, 23],
    [2, 6, 9, 15, 19, 21, 22, 23, 24],
    [3, 7, 10, 16, 20, 22, 23, 24, 25],
    [4, 8, 11, 17, 30, 23, 24, 25, 26],
    [5, 9, 12, 18, 21, 24, 25, 26, 27],
    [6, 10, 13, 19, 22, 25, 26, 0, 1],
    [7, 11, 14, 20, 23, 26, 0, 1, 2],
    [8, 12, 15, 30, 24, 0, 1, 2, 3],
    [9, 13, 16, 21, 25, 1, 2, 3, 4],
    [10, 14, 17, 22, 26, 2, 3, 4, 5],
    [11, 16, 18, 23, 0, 3, 4, 5, 6],
    [12, 16, 19, 24, 1, 4, 5, 6, 7],
    [13, 17, 20, 25, 2, 5, 6, 7, 8],
    [14, 18, 30, 26, 3, 6, 7, 8, 9],
    [15, 19, 21, 0, 4, 7, 8, 9, 10],
    [16, 20, 22, 1, 5, 8, 9, 10, 11],
    [17, 30, 23, 2, 6, 9, 10, 11, 12],
    [18, 21, 24, 3, 7, 10, 11, 12, 13],
    [19, 22, 25, 4, 8, 11, 12, 13, 14],
    [20, 23, 26, 5, 9, 12, 13, 14, 15],
    [21, 25, 1, 7, 11, 14, 15, 16, 17],
    [22, 26, 2, 8, 12, 15, 16, 17, 18],
    [23, 0, 3, 9, 13, 16, 17, 18, 19],
    [24, 1, 4, 10, 14, 17, 18, 19, 20],
    [25, 2, 5, 11, 15, 18, 19, 20, 30],
    [26, 3, 6, 12, 16, 19, 20, 30, 21],
];
/// Preserve Java's truncating double-to-int cast and uninitialized [0;9] fallback.
pub fn original_m0(angle: f64) -> [i32; 9] {
    let idx = (angle / 13.333333333333334_f64) as i32;
    ORIGINAL_M0.get(idx as usize).copied().unwrap_or([0; 9])
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Case {
        angle: f64,
        expected: [i32; 9],
    }
    #[test]
    fn all_27_apk_rows_and_boundaries() {
        let cases: Vec<Case> = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/original_m0.json"
        )))
        .unwrap();
        assert_eq!(cases.len(), 57);
        for c in cases {
            assert_eq!(original_m0(c.angle), c.expected, "M0 angle {}", c.angle);
        }
    }
    #[test]
    fn preserves_30_sentinel() {
        assert!(ORIGINAL_M0.iter().flatten().any(|code| *code == 30));
    }
}

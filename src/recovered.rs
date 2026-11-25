use motaa_engine::reference_ephemeris::ReferenceEphemeris;

use crate::model::Settings;

pub const BODIES: [&str; 10] = [
    "Ascendant",
    "Sun",
    "Moon",
    "Mars",
    "Mercury",
    "Jupiter",
    "Venus",
    "Saturn",
    "Rahu",
    "Ketu",
];

pub fn positions(
    ephemeris: &ReferenceEphemeris,
    ascendant: &ReferenceEphemeris,
    settings: Settings,
) -> Option<[f64; 10]> {
    let mut positions = [0.; 10];
    positions[0] = if settings.tropical {
        ascendant.ascendant_tropical_deg
    } else {
        ascendant.ascendant_sidereal_deg
    };
    for (index, name) in BODIES.iter().enumerate().take(8).skip(1) {
        positions[index] = ephemeris.longitude(name, settings.tropical)?;
    }
    positions[8..].copy_from_slice(&ephemeris.lunar_nodes(settings.mean_node, settings.tropical));
    Some(positions)
}

pub fn sign(angle: f64) -> usize {
    (angle.rem_euclid(360.) / 30.) as usize
}

/// Exact V31.1 `main.R` (`f3326z`) Rasi Drishti (sign aspect) lookup table.
/// Index is source sign (`0..12`, Aries..Pisces); values are the three aspected signs.
pub const ORIGINAL_RASI_ASPECTS: [[usize; 3]; 12] = [
    [4, 7, 10],
    [3, 6, 9],
    [4, 8, 11],
    [7, 10, 1],
    [6, 9, 0],
    [2, 8, 11],
    [10, 1, 4],
    [0, 3, 9],
    [11, 2, 5],
    [1, 4, 7],
    [0, 3, 6],
    [2, 5, 8],
];

/// V31.1 `main.R` / `main.T` (`f3309q == 9`): whether `source_longitude`'s sign casts a Rasi aspect onto `target_longitude`'s sign.
pub fn rasi_aspect(source_longitude: f64, target_longitude: f64) -> bool {
    ORIGINAL_RASI_ASPECTS[sign(source_longitude)].contains(&sign(target_longitude))
}

/// V31.1 `main.Q` (`f3317u == 1`): whether `source_longitude`'s sign casts a Rasi aspect onto whole-sign house `house_index` (`0..12`) from `ascendant_longitude`.
pub fn rasi_house_aspect(
    source_longitude: f64,
    ascendant_longitude: f64,
    house_index: usize,
) -> bool {
    let target_sign = (sign(ascendant_longitude) + house_index) % 12;
    ORIGINAL_RASI_ASPECTS[sign(source_longitude)].contains(&target_sign)
}

fn aspect_offset(source: usize, offset: usize) -> bool {
    match source {
        1 | 2 | 4 | 6 => offset == 6,
        3 => matches!(offset, 3 | 6 | 7),
        5 | 8 | 9 => matches!(offset, 4 | 6 | 8),
        7 => matches!(offset, 2 | 6 | 9),
        _ => false,
    }
}

pub fn whole_sign_aspect(source: usize, longitude: f64, target: f64) -> bool {
    aspect_offset(source, (12 + sign(target) - sign(longitude)) % 12)
}

/// V31.1 `main.Q` (`f3320w`): Graha Drishti from a planet's occupied house index (`0..12`)
/// onto a target house index (`0..12`).
pub fn house_aspect(source: usize, planet_house_index: usize, target_house_index: usize) -> bool {
    aspect_offset(source, (12 + target_house_index - planet_house_index) % 12)
}

pub fn navamsa(longitude: f64) -> f64 {
    (longitude * 9.).rem_euclid(360.)
}

pub fn equal_bhava(longitude: f64, ascendant: f64) -> u8 {
    (longitude - ascendant + 15.)
        .rem_euclid(360.)
        .div_euclid(30.) as u8
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_sectors_wrap_at_boundaries() {
        assert_eq!(navamsa(30.), 270.);
        assert_eq!(navamsa(40.), 0.);
        assert_eq!(equal_bhava(350., 5.), 1);
        assert_eq!(equal_bhava(349.99, 5.), 12);
        assert_eq!(equal_bhava(20., 5.), 2);
    }

    #[test]
    fn whole_sign_relationships_wrap_without_orbs() {
        assert!(whole_sign_aspect(3, 359.9, 60.));
        assert!(!whole_sign_aspect(3, 359.9, 59.9));
        assert!(whole_sign_aspect(7, 359.9, 30.));
        assert!(whole_sign_aspect(5, 359.9, 90.));
        assert!(!whole_sign_aspect(1, 359.9, 90.));
    }

    #[test]
    fn rasi_aspects_match_v31_1_main_r_and_q_tables() {
        // Aries (0°) aspects Leo (125°), Scorpio (215°), Aquarius (305°).
        assert!(rasi_aspect(0., 125.));
        assert!(rasi_aspect(0., 215.));
        assert!(rasi_aspect(0., 305.));
        assert!(!rasi_aspect(0., 35.));
        // House rotation from Taurus Ascendant (35°, sign 1): house index 3 is Leo (sign 4).
        assert!(rasi_house_aspect(0., 35., 3));
        assert!(!rasi_house_aspect(0., 35., 0));
    }

    #[test]
    fn house_aspect_uses_occupied_house_rather_than_sign() {
        // Asc at Virgo 4°36' (154.6°), Equal houses: House 1 starts at Leo 19°36' (139.6°).
        // A planet at Leo 25° is in House 1 (index 0), so Sun (source 1) aspects House 7 (index 6),
        // not House 8 (index 7) as sign-based distance to House 8 cusp would imply.
        assert!(house_aspect(1, 0, 6));
        assert!(!house_aspect(1, 0, 7));
        // Mars (source 3) in House 1 (index 0) aspects Houses 4, 7, 8 (indices 3, 6, 7).
        assert!(house_aspect(3, 0, 3));
        assert!(house_aspect(3, 0, 6));
        assert!(house_aspect(3, 0, 7));
        assert!(!house_aspect(3, 0, 4));
    }
}

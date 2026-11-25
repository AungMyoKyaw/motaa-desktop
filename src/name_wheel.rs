use std::sync::OnceLock;

use crate::{recovered, varga};

#[derive(serde::Deserialize)]
struct SourceRing {
    syllables: Vec<String>,
    #[serde(rename = "displayNames")]
    names: [String; 27],
}

fn source_ring() -> &'static SourceRing {
    static RING: OnceLock<SourceRing> = OnceLock::new();
    RING.get_or_init(|| {
        let ring: SourceRing =
            serde_json::from_str(include_str!("../design/source/circle-data.json"))
                .expect("checked MOTAA syllable ring");
        assert_eq!(ring.syllables.len(), 108);
        ring
    })
}

pub fn syllables() -> &'static [String] {
    &source_ring().syllables
}

pub fn mansion_name(index: usize) -> Option<&'static str> {
    source_ring().names.get(index).map(String::as_str)
}

pub fn jamma_groups(longitude: f64, use_28: bool) -> [Option<usize>; 6] {
    // V31.1 main.X0/Y0 both use the 27-sector reference index. Y0's
    // sentinel value 30 suppresses that classification in the wheel.
    const GROUPS_28: [[usize; 6]; 27] = [
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
    let index = sector(longitude) / 4;
    if use_28 {
        GROUPS_28[index].map(|value| (value != 30).then_some(value))
    } else {
        [0, 9, 15, 17, 20, 22].map(|delta| Some((index + delta) % 27))
    }
}

pub fn sector(longitude: f64) -> usize {
    (longitude.rem_euclid(360.) * 108. / 360.) as usize
}

pub fn center(index: usize) -> f64 {
    (index as f64 + 0.5) * 360. / 108.
}

pub struct SatapatiHouseRow {
    pub house: usize,
    pub sign: &'static str,
    pub natal_bodies: String,
    pub transit_bodies: String,
    pub aspects_to_lagna: String,
}

/// Builds the 12-house Satapati chart rotated so that `lagna_sector` (0..108) is House 1.
pub fn satapati_houses(
    lagna_sector: usize,
    natal: Option<[f64; 10]>,
    transit: Option<[f64; 10]>,
) -> Vec<SatapatiHouseRow> {
    let lagna_lon = center(lagna_sector % 108);
    let lagna_sign = recovered::sign(lagna_lon);
    (0..12)
        .map(|house_offset| {
            let sign_idx = (lagna_sign + house_offset) % 12;
            let collect_bodies = |pos: Option<[f64; 10]>| -> String {
                let Some(pos) = pos else {
                    return "—".into();
                };
                let list: Vec<&str> = (1..10)
                    .filter(|&i| recovered::sign(pos[i]) == sign_idx)
                    .map(|i| recovered::BODIES[i])
                    .collect();
                if list.is_empty() {
                    "—".into()
                } else {
                    list.join(", ")
                }
            };
            let aspects = natal.map_or_else(
                || "—".into(),
                |pos| {
                    let list: Vec<&str> = (1..10)
                        .filter(|&i| {
                            recovered::sign(pos[i]) == sign_idx
                                && recovered::whole_sign_aspect(i, pos[i], lagna_lon)
                        })
                        .map(|i| recovered::BODIES[i])
                        .collect();
                    if list.is_empty() {
                        "—".into()
                    } else {
                        list.join(", ")
                    }
                },
            );
            SatapatiHouseRow {
                house: house_offset + 1,
                sign: varga::SIGNS[sign_idx],
                natal_bodies: collect_bodies(natal),
                transit_bodies: collect_bodies(transit),
                aspects_to_lagna: aspects,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rahu_bands_match_shipped_motaa_runtime() {
        let fixture = std::env::var_os("MOTAA_ANDROID_RAHU").map_or_else(
            || include_bytes!("../tests/fixtures/motaa-v31.1-rahu-oracle.json").to_vec(),
            |path| std::fs::read(path).expect("read Android Rahu oracle"),
        );
        type Bands = ([f64; 10], [i32; 9], [i32; 9], [usize; 6], [usize; 6]);
        let entries: Vec<(f64, Bands)> = serde_json::from_slice(&fixture).unwrap();
        assert_eq!(entries.len(), 81);
        for (longitude, (zones, sun_27, sun_28, jamma_27, jamma_28)) in entries {
            let actual = motaa_engine::original_positions::original_b0(longitude);
            assert!(
                actual
                    .into_iter()
                    .zip(zones)
                    .all(|(a, b)| (a - b).abs() < 1e-9),
                "Rahu zones at {longitude}"
            );
            assert_eq!(
                motaa_engine::original_geometry::original_l0(longitude),
                sun_27,
                "Sun 27 at {longitude}"
            );
            assert_eq!(
                motaa_engine::original_m0::original_m0(longitude),
                sun_28,
                "Sun 28 at {longitude}"
            );
            for (use_28, expected) in [(false, jamma_27), (true, jamma_28)] {
                assert_eq!(
                    jamma_groups(longitude, use_28),
                    expected.map(|index| (index != 30).then_some(index)),
                    "Jamma ({use_28}) at {longitude}"
                );
            }
        }
    }

    #[test]
    fn all_source_cells_resolve_back_to_themselves() {
        assert_eq!(syllables().len(), 108);
        for index in 0..108 {
            assert_eq!(sector(center(index)), index);
        }
        assert_eq!(sector(360.), 0);
        assert_eq!(syllables()[0], "စု");
        assert!(syllables().iter().any(|syllable| syllable == "ဝ"));
    }

    #[test]
    fn satapati_houses_rotates_around_selected_syllable_sector() {
        // Sector 9 is in Taurus (sign 1, ~31.67°).
        let pos = [0., 35., 215., 90., 120., 150., 180., 210., 240., 60.];
        let rows = satapati_houses(9, Some(pos), None);
        assert_eq!(rows.len(), 12);
        assert_eq!(rows[0].house, 1);
        assert_eq!(rows[0].sign, "Taurus");
        assert_eq!(rows[0].natal_bodies, "Sun");
        // Moon & Saturn in Scorpio (215°, 210°) both aspect Taurus (7th house)
        assert_eq!(rows[6].aspects_to_lagna, "Moon, Saturn");
    }
}

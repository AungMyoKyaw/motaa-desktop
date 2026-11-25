use crate::{recovered, tara};

pub const GRID: [[&str; 9]; 9] = [
    [
        "ī",
        "Dhan",
        "Shata",
        "P. Bhadra",
        "U. Bhadra",
        "Revati",
        "Ashwini",
        "Bharani",
        "a",
    ],
    [
        "Shravana", "ṝ", "ga", "sa", "da", "ca", "la", "u", "Krittika",
    ],
    [
        "Abhijit", "kha", "ai", "Aquarius", "Pisces", "Aries", "ḷ", "a", "Rohini",
    ],
    [
        "U. Ashadha",
        "ja",
        "Capricorn",
        "aḥ",
        "Rikta",
        "o",
        "Taurus",
        "va",
        "Mrigashira",
    ],
    [
        "P. Ashadha",
        "bha",
        "Sagittarius",
        "Jaya",
        "Purna",
        "Nanda",
        "Gemini",
        "ka",
        "Ardra",
    ],
    [
        "Mula",
        "ya",
        "Scorpio",
        "aṃ",
        "Bhadra",
        "au",
        "Cancer",
        "ha",
        "Punarvasu",
    ],
    [
        "Jyeshtha", "na", "e", "Libra", "Virgo", "Leo", "ḹ", "ḍa", "Pushya",
    ],
    [
        "Anuradha", "ṛ", "ai", "ra", "pa", "ṭa", "ma", "ū", "Ashlesha",
    ],
    [
        "i",
        "Vishakha",
        "Swati",
        "Chitra",
        "Hasta",
        "U. Phalguni",
        "P. Phalguni",
        "Magha",
        "ā",
    ],
];

const ORDER: [(usize, usize); 28] = [
    (0, 6),
    (0, 7),
    (1, 8),
    (2, 8),
    (3, 8),
    (4, 8),
    (5, 8),
    (6, 8),
    (7, 8),
    (8, 7),
    (8, 6),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
    (7, 0),
    (6, 0),
    (5, 0),
    (4, 0),
    (3, 0),
    (2, 0),
    (1, 0),
    (0, 1),
    (0, 2),
    (0, 3),
    (0, 4),
    (0, 5),
];

pub fn cell(longitude: f64, abhijit: bool) -> (usize, usize) {
    let longitude = longitude.rem_euclid(360.);
    if abhijit && ((276. + 40. / 60.)..(280. + 53. / 60. + 20. / 3600.)).contains(&longitude) {
        return ORDER[21];
    }
    let index = tara::mansion(longitude);
    ORDER[index + usize::from(index >= 21)]
}

pub fn rays(origin: (usize, usize)) -> [Vec<(usize, usize)>; 3] {
    let (row, col) = (origin.0 as i32, origin.1 as i32);
    let directions = match origin {
        (0, _) => [(1, 0), (1, 1), (1, -1)],
        (8, _) => [(-1, 0), (-1, -1), (-1, 1)],
        (_, 0) => [(0, 1), (-1, 1), (1, 1)],
        _ => [(0, -1), (1, -1), (-1, -1)],
    };
    directions.map(|(dr, dc)| {
        (1..=8)
            .map(|step| (row + dr * step, col + dc * step))
            .take_while(|(row, col)| (0..=8).contains(row) && (0..=8).contains(col))
            .map(|(row, col)| (row as usize, col as usize))
            .collect()
    })
}

pub struct SpecialPoints {
    pub janma_cell: (usize, usize),
    pub nighata_cell: (usize, usize),
    pub vajira_cell: (usize, usize),
    pub bhumi_cell: (usize, usize),
    pub latta_sun: (usize, usize),
    pub latta_mars: (usize, usize),
    pub latta_jupiter: (usize, usize),
    pub latta_saturn: (usize, usize),
}

/// Computes classical Sarvatobhadra special nakshatra cells (Nighata, Vajira, Bhumi, and Latta kicks).
pub fn special_points(
    natal_moon_lon: f64,
    transit_positions: &[f64; 10],
    abhijit: bool,
) -> SpecialPoints {
    let count = if abhijit { 28 } else { 27 };
    let nak_index = |lon: f64| -> usize {
        if abhijit {
            let (row, col) = cell(lon, true);
            ORDER
                .iter()
                .position(|&c| c == (row, col))
                .unwrap_or_else(|| tara::mansion(lon))
        } else {
            tara::mansion(lon)
        }
    };
    let cell_at = |idx: usize| -> (usize, usize) {
        let i = idx.rem_euclid(count);
        if abhijit {
            ORDER[i]
        } else {
            ORDER[i + usize::from(i >= 21)]
        }
    };
    let janma = nak_index(natal_moon_lon);
    // Classical Sarvatobhadra counts from Janma Nakshatra (1-based: 19=Adhana/Vajira, 22=Vainashika/Nighata, 23=Manasa/Bhumi)
    let nighata = (janma + 21) % count;
    let vajira = (janma + 18) % count;
    let bhumi = (janma + 22) % count;

    // Classical Latta (planetary kick):
    // Forward kicks: Sun 12th, Mars 3rd, Jupiter 6th, Saturn 8th
    let latta_sun = cell_at(nak_index(transit_positions[1]) + 11);
    let latta_mars = cell_at(nak_index(transit_positions[3]) + 2);
    let latta_jupiter = cell_at(nak_index(transit_positions[5]) + 5);
    let latta_saturn = cell_at(nak_index(transit_positions[7]) + 7);

    SpecialPoints {
        janma_cell: cell_at(janma),
        nighata_cell: cell_at(nighata),
        vajira_cell: cell_at(vajira),
        bhumi_cell: cell_at(bhumi),
        latta_sun,
        latta_mars,
        latta_jupiter,
        latta_saturn,
    }
}

/// Returns short labels for all bodies occupying each cell in the 9x9 grid.
pub fn cell_occupants(
    positions: &[f64; 10],
    abhijit: bool,
) -> Vec<((usize, usize), Vec<&'static str>)> {
    let mut map: Vec<((usize, usize), Vec<&'static str>)> = Vec::new();
    for (idx, &lon) in positions.iter().enumerate().skip(1) {
        let c = cell(lon, abhijit);
        let abbr = match recovered::BODIES[idx] {
            "Sun" => "Su",
            "Moon" => "Mo",
            "Mars" => "Ma",
            "Mercury" => "Me",
            "Jupiter" => "Ju",
            "Venus" => "Ve",
            "Saturn" => "Sa",
            "Rahu" => "Ra",
            "Ketu" => "Ke",
            _ => "",
        };
        if let Some((_, list)) = map.iter_mut().find(|(pos, _)| *pos == c) {
            list.push(abbr);
        } else {
            map.push((c, vec![abbr]));
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classical_krittika_and_rohini_rays_hit_published_targets() {
        let terminal = |angle| {
            rays(cell(angle, true)).map(|ray| {
                let &(row, col) = ray.last().unwrap();
                GRID[row][col]
            })
        };
        assert_eq!(terminal(28.), ["Shravana", "Vishakha", "Bharani"]);
        assert_eq!(terminal(42.), ["Abhijit", "Swati", "Ashwini"]);
        assert_eq!(cell(279., true), (2, 0));
        assert_eq!(cell(279., false), (3, 0));
        assert_eq!(cell(281., true), (1, 0));
    }

    #[test]
    fn special_points_and_occupants_resolve_to_valid_grid_cells() {
        let positions = [0., 15., 45., 75., 105., 135., 165., 195., 225., 45.];
        let sp = special_points(45., &positions, false);
        assert!(sp.janma_cell.0 < 9 && sp.janma_cell.1 < 9);
        assert!(sp.nighata_cell.0 < 9 && sp.nighata_cell.1 < 9);
        let occ = cell_occupants(&positions, false);
        assert!(!occ.is_empty());
    }
}

use motaa_engine::{EN, myanmar_calendar};

use crate::{model::Settings, recovered, tara, varga};

pub const MAHABOTE_HOUSES: [&str; 7] = [
    "Binga", "Marana", "Atun", "Thike", "Yaza", "Puti", "Adipati",
];

/// Weekday order in Burmese astrology: 0=Sun, 1=Mon, 2=Tue, 3=Wed, 4=Thu, 5=Fri, 6=Sat.
pub const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Planetary ruler of each weekday (0=Sun..6=Sat) mapped to `recovered::BODIES` indices.
pub const WEEKDAY_PLANETS: [usize; 7] = [1, 2, 3, 4, 5, 6, 7];

/// Burmese Mahabote planetary number cycle: 1(Sun), 4(Wed), 0(Sat), 3(Tue), 6(Fri), 2(Mon), 5(Thu).
const MAHABOTE_CYCLE: [usize; 7] = [0, 3, 6, 2, 5, 1, 4];

/// Natural friendship table among the 7 classical planets (Sun=1..Saturn=7 -> index 0..6).
/// Returns +1 for Friend, 0 for Neutral, -1 for Enemy.
pub fn natural_maitri(planet_a: usize, planet_b: usize) -> i8 {
    // 0:Sun, 1:Moon, 2:Mars, 3:Merc, 4:Jup, 5:Ven, 6:Sat
    const TABLE: [[i8; 7]; 7] = [
        [0, 1, 1, 0, 1, -1, -1],  // Sun
        [1, 0, 0, 1, 0, 0, 0],    // Moon
        [1, 1, 0, -1, 1, 0, 0],   // Mars
        [1, -1, 0, 0, 0, 1, 0],   // Mercury
        [1, 1, 1, -1, 0, -1, 0],  // Jupiter
        [-1, -1, 0, 1, 0, 0, 1],  // Venus
        [-1, -1, -1, 1, 0, 1, 0], // Saturn
    ];
    if planet_a == planet_b {
        return 1;
    }
    let Some(a) = planet_a.checked_sub(1).filter(|&i| i < 7) else {
        return 0;
    };
    let Some(b) = planet_b.checked_sub(1).filter(|&i| i < 7) else {
        return 0;
    };
    TABLE[a][b]
}

/// Sign ruler mapped to `recovered::BODIES` index (1=Sun..7=Saturn).
pub fn sign_lord(sign: usize) -> usize {
    match sign % 12 {
        0 | 7 => 3,  // Mars
        1 | 6 => 6,  // Venus
        2 | 5 => 4,  // Mercury
        3 => 2,      // Moon
        4 => 1,      // Sun
        8 | 11 => 5, // Jupiter
        _ => 7,      // Saturn
    }
}

/// Nakshatra ruler (0..26) mapped to `recovered::BODIES` index (1..9).
pub fn nakshatra_lord(mansion: usize) -> usize {
    const LORDS: [usize; 9] = [9, 6, 1, 2, 3, 8, 5, 7, 4];
    LORDS[mansion % 9]
}

/// Panchadha Maitri (five-fold friendship) between `planet` and `other` given chart `positions`.
pub fn panchadha_maitri(planet: usize, other: usize, positions: &[f64; 10]) -> &'static str {
    if planet == other {
        return "Self";
    }
    if !(1..=7).contains(&planet) || !(1..=7).contains(&other) {
        return "Neutral";
    }
    let nat = natural_maitri(planet, other);
    let offset = (12 + recovered::sign(positions[other]) - recovered::sign(positions[planet])) % 12;
    let temp = if matches!(offset, 1 | 2 | 3 | 9 | 10 | 11) {
        1_i8
    } else {
        -1_i8
    };
    match nat + temp {
        2 => "Best Friend",
        1 => "Friend",
        0 => "Neutral",
        -1 => "Enemy",
        _ => "Bitter Enemy",
    }
}

pub struct PillarEntry {
    pub house_name: &'static str,
    pub planet: &'static str,
    pub weekday: &'static str,
    pub pillar_role: &'static str,
}

pub struct AyuMoonRow {
    pub body: &'static str,
    pub epoch: &'static str,
    pub mansion: &'static str,
    pub pada: usize,
    pub navamsa_sign: &'static str,
    pub navamsa_lord: &'static str,
    pub maitri_to_moon: &'static str,
    pub padakhike_status: &'static str,
}

pub struct TenPillarsReport {
    pub myanmar_year: i32,
    pub mahabote_seed: usize,
    pub birth_weekday: &'static str,
    pub birth_house: &'static str,
    pub adipati_planet: &'static str,
    pub ratana_pillar: String,
    pub suvanna_pillar: String,
    pub rajita_pillar: String,
    pub ayu_moon_mansion: &'static str,
    pub ayu_moon_pada: usize,
    pub ayu_moon_navamsa: &'static str,
    pub ayu_moon_status: &'static str,
    pub pillars: Vec<PillarEntry>,
    pub rows: Vec<AyuMoonRow>,
}

pub fn calculate(
    myanmar_year: i32,
    birth_weekday_iso: usize, // 0=Sun..6=Sat
    natal_positions: [f64; 10],
    transit_positions: Option<[f64; 10]>,
    _settings: Settings,
) -> TenPillarsReport {
    // Burmese remainder: myanmar_year % 7, with 0 -> 7
    let rem = match myanmar_year.rem_euclid(7) {
        0 => 7,
        v => v as usize,
    };
    // Map remainder 1..=7 into MAHABOTE_CYCLE start index
    // Weekday indices: 0=Sun(1), 1=Mon(2), 2=Tue(3), 3=Wed(4), 4=Thu(5), 5=Fri(6), 6=Sat(0/7)
    let seed_weekday = (rem - 1) % 7;
    let start_pos = MAHABOTE_CYCLE
        .iter()
        .position(|&w| w == seed_weekday)
        .unwrap_or(0);

    let mut birth_house = MAHABOTE_HOUSES[0];
    let mut pillars = Vec::with_capacity(7);
    let mut house_planets = [""; 7];
    for (house_idx, house_name) in MAHABOTE_HOUSES.iter().enumerate() {
        let wd = MAHABOTE_CYCLE[(start_pos + house_idx) % 7];
        let planet_idx = WEEKDAY_PLANETS[wd];
        let planet_name = recovered::BODIES[planet_idx];
        house_planets[house_idx] = planet_name;
        if wd == birth_weekday_iso % 7 {
            birth_house = house_name;
        }
        let role = match house_idx {
            0 => "Mula / Binga",
            2 => "Suvanna (Wealth)",
            3 => "Thike / Rajita",
            4 => "Yaza (Authority)",
            5 => "Puti (Support)",
            6 => "Adipati (Crown)",
            _ => "Marana (Trial)",
        };
        pillars.push(PillarEntry {
            house_name,
            planet: planet_name,
            weekday: WEEKDAYS[wd],
            pillar_role: role,
        });
    }

    let natal_moon = natal_positions[2];
    let moon_mansion = tara::mansion(natal_moon);
    let moon_pada = ((natal_moon.rem_euclid(360.) % (360. / 27.)) / (360. / 108.)) as usize + 1;
    let moon_navamsa = recovered::sign(recovered::navamsa(natal_moon));
    let moon_lord = nakshatra_lord(moon_mansion);
    let ayu_status = match moon_pada {
        1 => "Upad · Rising (Pada 1)",
        2 => "Sote · Steady (Pada 2)",
        3 => "Sote · Peak (Pada 3)",
        _ => "Padakhike · Critical (Pada 4)",
    };

    let build_rows = |epoch: &'static str, pos: [f64; 10]| -> Vec<AyuMoonRow> {
        (1..10)
            .map(|idx| {
                let lon = pos[idx];
                let m = tara::mansion(lon);
                let pada = ((lon.rem_euclid(360.) % (360. / 27.)) / (360. / 108.)) as usize + 1;
                let nav_sign = recovered::sign(recovered::navamsa(lon));
                let nav_lord = sign_lord(nav_sign);
                let n_lord = nakshatra_lord(m);
                let maitri = panchadha_maitri(moon_lord, n_lord, &natal_positions);
                let padakhike = if pada == 4 && matches!(maitri, "Enemy" | "Bitter Enemy") {
                    "Padakhike (Afflicted)"
                } else if matches!(maitri, "Best Friend" | "Friend") {
                    "Sote (Favorable)"
                } else if maitri == "Neutral" {
                    "Balanced"
                } else {
                    "Upad (Caution)"
                };
                AyuMoonRow {
                    body: recovered::BODIES[idx],
                    epoch,
                    mansion: EN[m],
                    pada,
                    navamsa_sign: varga::SIGNS[nav_sign],
                    navamsa_lord: recovered::BODIES[nav_lord],
                    maitri_to_moon: maitri,
                    padakhike_status: padakhike,
                }
            })
            .collect()
    };

    let mut rows = build_rows("Natal", natal_positions);
    if let Some(transit) = transit_positions {
        rows.extend(build_rows("Transit", transit));
    }

    TenPillarsReport {
        myanmar_year,
        mahabote_seed: rem,
        birth_weekday: WEEKDAYS[birth_weekday_iso % 7],
        birth_house,
        adipati_planet: house_planets[6],
        ratana_pillar: format!(
            "{} · {} · {}",
            house_planets[3], house_planets[4], house_planets[6]
        ),
        suvanna_pillar: format!("{} · {}", house_planets[2], house_planets[4]),
        rajita_pillar: format!("{} · {}", house_planets[3], house_planets[5]),
        ayu_moon_mansion: EN[moon_mansion],
        ayu_moon_pada: moon_pada,
        ayu_moon_navamsa: varga::SIGNS[moon_navamsa],
        ayu_moon_status: ayu_status,
        pillars,
        rows,
    }
}

pub fn myanmar_year_for_date(year: i32, month: u8, day: u8) -> Option<i32> {
    let jdn = motaa_engine::original_geometry::original_p0(i32::from(day), i32::from(month), year);
    myanmar_calendar::from_julian_day_number(jdn as i64).map(|d| d.year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mahabote_assigns_seven_unique_planets_to_seven_houses() {
        let positions = [0., 10., 45., 90., 120., 150., 200., 250., 300., 120.];
        let report = calculate(1345, 2, positions, None, Settings::default());
        assert_eq!(report.pillars.len(), 7);
        let mut names: Vec<&str> = report.pillars.iter().map(|p| p.planet).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 7);
        assert_eq!(report.rows.len(), 9);
    }

    #[test]
    fn panchadha_maitri_combines_natural_and_temporary_friendship() {
        // Sun at Aries (0°), Jupiter at Leo (125°, 5th sign -> temporary enemy, natural friend => Neutral)
        // Sun at Aries (0°), Mars at Taurus (35°, 2nd sign -> temporary friend, natural friend => Best Friend)
        let positions = [0., 0., 60., 35., 90., 125., 180., 210., 240., 60.];
        assert_eq!(panchadha_maitri(1, 3, &positions), "Best Friend");
        assert_eq!(panchadha_maitri(1, 5, &positions), "Neutral");
    }
}

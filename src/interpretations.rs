use motaa_engine::{EN, dasha};

use crate::{ashtakavarga, recovered, tara, ten_pillars, varga};

pub struct YogaMatch {
    pub category: &'static str,
    pub name: &'static str,
    pub participants: String,
    pub meaning: &'static str,
}

/// Planetary dignity in a sidereal sign (Sun=1..Saturn=7).
pub fn dignity(planet: usize, sign: usize) -> &'static str {
    let exalt = match planet {
        1 => Some(0),  // Sun in Aries
        2 => Some(1),  // Moon in Taurus
        3 => Some(9),  // Mars in Capricorn
        4 => Some(5),  // Mercury in Virgo
        5 => Some(3),  // Jupiter in Cancer
        6 => Some(11), // Venus in Pisces
        7 => Some(6),  // Saturn in Libra
        8 => Some(1),  // Rahu in Taurus
        9 => Some(7),  // Ketu in Scorpio
        _ => None,
    };
    let debil = exalt.map(|s| (s + 6) % 12);
    if exalt == Some(sign) {
        return "Exalted";
    }
    if debil == Some(sign) {
        return "Debilitated";
    }
    if ten_pillars::sign_lord(sign) == planet {
        return "Own Sign";
    }
    let lord = ten_pillars::sign_lord(sign);
    match ten_pillars::natural_maitri(planet, lord) {
        1 => "Friendly Sign",
        -1 => "Enemy Sign",
        _ => "Neutral Sign",
    }
}

pub fn planet_nature(planet_name: &str) -> &'static str {
    match planet_name {
        "Sun" => "Authority, vitality, soul purpose, leadership, father, government",
        "Moon" => "Mind, emotions, mother, public connection, nourishment, adaptability",
        "Mars" => "Courage, initiative, siblings, land, technical skill, decisiveness",
        "Mercury" => "Intellect, commerce, speech, writing, analytical skill, adaptability",
        "Jupiter" => "Wisdom, dharma, teachers, expansion, children, fortune",
        "Venus" => "Harmony, arts, relationships, comfort, luxury, diplomacy",
        "Saturn" => "Discipline, endurance, longevity, service, structure, karmic duty",
        "Rahu" => "Ambition, unconventional paths, foreign ties, sudden amplification",
        "Ketu" => "Detachment, spiritual insight, research, past-life mastery, liberation",
        _ => "Chart factor",
    }
}

pub fn house_domain(house: u8) -> &'static str {
    match house {
        1 => "Self, vitality, physical constitution, personal direction",
        2 => "Wealth, family, speech, accumulated resources",
        3 => "Courage, siblings, communication, short journeys, effort",
        4 => "Home, mother, emotional peace, vehicles, landed property",
        5 => "Creativity, intelligence, children, merit, speculative insight",
        6 => "Service, health, daily work, overcoming obstacles and rivals",
        7 => "Partnerships, marriage, contracts, public dealings",
        8 => "Transformation, longevity, research, hidden inheritance",
        9 => "Fortune, dharma, higher learning, father, long journeys",
        10 => "Career, public standing, achievements, authority",
        11 => "Gains, elder siblings, networks, fulfillment of aspirations",
        _ => "Expenditure, foreign residence, retreat, spiritual liberation",
    }
}

pub fn pada_focus(pada: usize) -> &'static str {
    match pada {
        1 => {
            "Pada 1 (Aries/Dharma impulse): direct initiation, pioneering drive, rapid expression."
        }
        2 => {
            "Pada 2 (Taurus/Artha impulse): steady consolidation, practical resourcefulness, patience."
        }
        3 => {
            "Pada 3 (Gemini/Kama impulse): curiosity, communication, social exchange, versatility."
        }
        _ => {
            "Pada 4 (Cancer/Moksha impulse): emotional depth, protective instinct, reflective sensitivity."
        }
    }
}

pub fn planet_summary(
    planet: usize,
    positions: &[f64; 10],
    use_bhava: bool,
    epoch_label: &str,
) -> (String, String, String) {
    let lon = positions[planet];
    let sign_idx = recovered::sign(lon);
    let house = if use_bhava {
        recovered::equal_bhava(lon, positions[0])
    } else {
        ((12 + sign_idx - recovered::sign(positions[0])) % 12) as u8 + 1
    };
    let dig = dignity(planet, sign_idx);
    let title = format!(
        "{} ({epoch_label}) · {} · House {house} ({}) · {dig}",
        recovered::BODIES[planet],
        varga::SIGNS[sign_idx],
        if use_bhava { "Bhava" } else { "Rasi" }
    );
    let nature = format!(
        "Signification: {}",
        planet_nature(recovered::BODIES[planet])
    );
    let house_text = format!("House {house} focus: {}", house_domain(house));
    (title, nature, house_text)
}

pub fn dasha_pd_summary(
    md: dasha::Period,
    ad: Option<dasha::Period>,
    pd: Option<dasha::Period>,
    natal_positions: &[f64; 10],
) -> (String, String) {
    let md_name = md.name();
    let ad_name = ad.map_or("—", dasha::Period::name);
    let pd_name = pd.map_or("—", dasha::Period::name);
    let chain = format!("{md_name} → {ad_name} → {pd_name}");

    let body_idx = |name: &str| {
        recovered::BODIES
            .iter()
            .position(|&b| b == name)
            .unwrap_or(1)
    };
    let md_idx = body_idx(md_name);
    let pd_idx = pd.map_or(md_idx, |p| body_idx(p.name()));
    let md_dig = dignity(md_idx, recovered::sign(natal_positions[md_idx]));
    let pd_dig = dignity(pd_idx, recovered::sign(natal_positions[pd_idx]));
    let maitri = ten_pillars::panchadha_maitri(md_idx, pd_idx, natal_positions);

    let commentary = format!(
        "Mahadasha {md_name} ({md_dig}) activates: {}. Pratyantardasha {pd_name} ({pd_dig}, {maitri} to {md_name}) colors daily events with: {}.",
        planet_nature(md_name),
        planet_nature(pd_name)
    );
    (chain, commentary)
}

/// Detects single-planet, two-planet, and multi-planet Sambandha/Yoga combinations.
pub fn detect_yogas(positions: &[f64; 10]) -> Vec<YogaMatch> {
    let mut yogas = Vec::new();
    let lagna_sign = recovered::sign(positions[0]);
    let signs = std::array::from_fn::<_, 10, _>(|i| recovered::sign(positions[i]));
    let house_of = |body: usize| ((12 + signs[body] - lagna_sign) % 12) + 1;
    let is_kendra = |h: usize| matches!(h, 1 | 4 | 7 | 10);
    let is_trikona = |h: usize| matches!(h, 1 | 5 | 9);

    // 1. Single-planet Pancha Mahapurusha & Exaltation yogas
    let mahapurusha: [(usize, &str, &[usize], &str); 5] = [
        (
            3,
            "Ruchaka Yoga",
            &[0, 7, 9],
            "Courage, command, and decisive leadership",
        ),
        (
            4,
            "Bhadra Yoga",
            &[2, 5],
            "Sharp intellect, eloquence, and commercial acumen",
        ),
        (
            5,
            "Hamsa Yoga",
            &[3, 8, 11],
            "Wisdom, ethical stature, and spiritual grace",
        ),
        (
            6,
            "Malavya Yoga",
            &[1, 6, 11],
            "Refinement, artistic talent, and material comfort",
        ),
        (
            7,
            "Sasa Yoga",
            &[6, 9, 10],
            "Organizational authority, endurance, and longevity",
        ),
    ];
    for (body, name, strong_signs, meaning) in mahapurusha {
        let h = house_of(body);
        if is_kendra(h) && strong_signs.contains(&signs[body]) {
            yogas.push(YogaMatch {
                category: "Single-planet Yoga",
                name,
                participants: format!(
                    "{} in {} (H{h})",
                    recovered::BODIES[body],
                    varga::SIGNS[signs[body]]
                ),
                meaning,
            });
        }
    }

    // 2. Two-planet Yogas
    // Gajakesari: Jupiter in Kendra (1,4,7,10) from Moon
    let jup_from_moon = ((12 + signs[5] - signs[2]) % 12) + 1;
    if is_kendra(jup_from_moon) {
        yogas.push(YogaMatch {
            category: "Two-planet Yoga",
            name: "Gajakesari Yoga",
            participants: format!("Jupiter H{jup_from_moon} from Moon"),
            meaning: "Fame, generous intellect, and enduring reputation.",
        });
    }
    // Budhaditya: Sun & Mercury in same sign
    if signs[1] == signs[4] {
        yogas.push(YogaMatch {
            category: "Two-planet Yoga",
            name: "Budhaditya Yoga",
            participants: format!("Sun + Mercury in {}", varga::SIGNS[signs[1]]),
            meaning: "Analytical brilliance, administrative skill, and clarity of speech.",
        });
    }
    // Chandra-Mangala: Moon & Mars conjunct or mutual 7th
    let moon_mars = (12 + signs[3] - signs[2]) % 12;
    if moon_mars == 0 || moon_mars == 6 {
        yogas.push(YogaMatch {
            category: "Two-planet Yoga",
            name: "Chandra-Mangala Yoga",
            participants: format!(
                "Moon ({}) & Mars ({})",
                varga::SIGNS[signs[2]],
                varga::SIGNS[signs[3]]
            ),
            meaning: "Enterprise, financial drive, and energetic resourcefulness.",
        });
    }
    // Guru-Mangala: Jupiter & Mars conjunct or mutual 7th
    let jup_mars = (12 + signs[5] - signs[3]) % 12;
    if jup_mars == 0 || jup_mars == 6 {
        yogas.push(YogaMatch {
            category: "Two-planet Yoga",
            name: "Guru-Mangala Yoga",
            participants: format!(
                "Jupiter ({}) & Mars ({})",
                varga::SIGNS[signs[5]],
                varga::SIGNS[signs[3]]
            ),
            meaning: "Principled action, strategic energy, and protective leadership.",
        });
    }

    // 3. Kendra-Trikona Raja Yoga & Dhana / Bhava-lordship Sambandha
    let lord_of_house = |h: usize| ten_pillars::sign_lord((lagna_sign + h - 1) % 12);
    let kendra_lords: Vec<usize> = [1, 4, 7, 10].into_iter().map(lord_of_house).collect();
    let trikona_lords: Vec<usize> = [1, 5, 9].into_iter().map(lord_of_house).collect();
    for &kl in &kendra_lords {
        for &tl in &trikona_lords {
            if kl == tl {
                continue;
            }
            if signs[kl] == signs[tl] {
                yogas.push(YogaMatch {
                    category: "Bhava Sambandha",
                    name: "Kendra–Trikona Raja Yoga",
                    participants: format!(
                        "{} + {} in {}",
                        recovered::BODIES[kl],
                        recovered::BODIES[tl],
                        varga::SIGNS[signs[kl]]
                    ),
                    meaning: "Conjunction of angular and trinal lords; elevates status and purpose.",
                });
            }
        }
    }

    // 4. Multi-planet stellium (3+ classical planets in one sign)
    for sign_idx in 0..12 {
        let cluster: Vec<&str> = (1..=7)
            .filter(|&b| signs[b] == sign_idx)
            .map(|b| recovered::BODIES[b])
            .collect();
        if cluster.len() >= 3 {
            yogas.push(YogaMatch {
                category: "Multi-planet Yoga",
                name: "Pravrajya / Stellium Yoga",
                participants: format!("{} in {}", cluster.join(" + "), varga::SIGNS[sign_idx]),
                meaning: "Concentrated planetary energy in one sign/house; strong life focus.",
            });
        }
    }

    // 5. Ashtakavarga high-bindu sign note
    let av_signs = std::array::from_fn(|i| signs[(i + 1) % 8]);
    let (_, total) = ashtakavarga::calculate(av_signs);
    if total[lagna_sign] >= 30 {
        yogas.push(YogaMatch {
            category: "Ashtakavarga Strength",
            name: "Strong Lagna Bindus",
            participants: format!(
                "{} ({} bindus)",
                varga::SIGNS[lagna_sign],
                total[lagna_sign]
            ),
            meaning: "Above-average Sarvashtakavarga support on the Ascendant sign.",
        });
    }

    let _ = (is_trikona, EN[0], tara::mansion(0.));
    yogas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_budhaditya_and_gajakesari_yogas() {
        // Aries Lagna (0°), Sun & Mercury in Aries (10°, 15°), Moon in Taurus (40°), Jupiter in Leo (130°, 4th from Moon -> Gajakesari)
        let pos = [5., 10., 40., 200., 15., 130., 250., 300., 60., 240.];
        let yogas = detect_yogas(&pos);
        assert!(yogas.iter().any(|y| y.name == "Budhaditya Yoga"));
        assert!(yogas.iter().any(|y| y.name == "Gajakesari Yoga"));
    }
}

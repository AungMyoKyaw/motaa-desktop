//! Reference Panchanga limbs and approximate visible solar and lunar horizon crossings.
use astro::{lunar, nutation, sun};

use crate::reference_ephemeris::reference_delta_t;

const WIDTH: f64 = 360. / 27.;
const SECONDS_PER_DAY: f64 = 86_400.;
const YOGAS: [&str; 27] = [
    "Vishkambha",
    "Priti",
    "Ayushman",
    "Saubhagya",
    "Shobhana",
    "Atiganda",
    "Sukarman",
    "Dhriti",
    "Shula",
    "Ganda",
    "Vriddhi",
    "Dhruva",
    "Vyaghata",
    "Harshana",
    "Vajra",
    "Siddhi",
    "Vyatipata",
    "Variyana",
    "Parigha",
    "Shiva",
    "Siddha",
    "Sadhya",
    "Shubha",
    "Shukla",
    "Brahma",
    "Indra",
    "Vaidhriti",
];

fn indices(sun: f64, moon: f64, ayanamsha: f64) -> [usize; 4] {
    let elongation = (moon - sun).rem_euclid(360.);
    [
        (elongation / 12.) as usize,
        ((moon - ayanamsha).rem_euclid(360.) / WIDTH) as usize,
        ((moon + sun - 2. * ayanamsha).rem_euclid(360.) / WIDTH) as usize,
        (elongation / 6.) as usize,
    ]
}

fn karana(index: usize) -> &'static str {
    match index {
        0 => "Kimstughna",
        57 => "Shakuni",
        58 => "Chatushpada",
        59 => "Naga",
        _ => [
            "Bava", "Balava", "Kaulava", "Taitila", "Gara", "Vanija", "Vishti",
        ][(index - 1) % 7],
    }
}

#[derive(Debug)]
pub struct Limb {
    pub kind: &'static str,
    pub active: String,
    pub next: String,
    pub next_jdut: Option<f64>,
}

#[derive(Debug)]
pub struct DailyTiming {
    pub limbs: [Limb; 4],
    pub remaining_day_transitions: [Vec<(f64, String)>; 4],
    pub sunrise: Option<f64>,
    pub sunset: Option<f64>,
    pub moonrise: Option<f64>,
    pub moonset: Option<f64>,
}

fn label(kind: usize, index: usize) -> String {
    match kind {
        0 => format!(
            "{} {}",
            if index < 15 { "Waxing" } else { "Waning" },
            index % 15 + 1
        ),
        1 => crate::EN[index].into(),
        2 => YOGAS[index].into(),
        _ => karana(index).into(),
    }
}

fn sun_moon(jdut: f64, delta_t: f64) -> [f64; 2] {
    let jde = jdut + delta_t / SECONDS_PER_DAY;
    let (solar, distance) = sun::geocent_ecl_pos(jde);
    let lunar = lunar::geocent_ecl_pos(jde).0;
    let nutation = nutation::nutation(jde).0;
    let aberration = (20.4898_f64 / (3600. * distance)).to_radians();
    [
        (solar.long + nutation - aberration).to_degrees(),
        (lunar.long + nutation).to_degrees(),
    ]
}

fn bisect(mut low: f64, mut high: f64, changed: impl Fn(f64) -> bool) -> f64 {
    while (high - low) * SECONDS_PER_DAY > 1. {
        let mid = (low + high) * 0.5;
        if changed(mid) {
            high = mid;
        } else {
            low = mid;
        }
    }
    high
}

fn solar_altitude(jdut: f64, delta_t: f64, latitude: f64, longitude: f64) -> f64 {
    let sun = sun::geocent_ecl_pos(jdut + delta_t / SECONDS_PER_DAY).0;
    let t = (jdut - 2451545.) / 36525.;
    let obliquity = (23.439291111 - 0.013004167 * t).to_radians();
    let x = sun.lat.cos() * sun.long.cos();
    let y = sun.lat.cos() * sun.long.sin() * obliquity.cos() - sun.lat.sin() * obliquity.sin();
    let z = sun.lat.cos() * sun.long.sin() * obliquity.sin() + sun.lat.sin() * obliquity.cos();
    let declination = z.asin();
    let gmst = (280.46061837 + 360.98564736629 * (jdut - 2451545.) + 0.000387933 * t * t
        - t * t * t / 38710000.
        + longitude)
        .rem_euclid(360.);
    let hour_angle = gmst.to_radians() - y.atan2(x);
    let latitude = latitude.to_radians();
    (latitude.sin() * declination.sin() + latitude.cos() * declination.cos() * hour_angle.cos())
        .clamp(-1., 1.)
        .asin()
        .to_degrees()
}

pub fn calculate(
    jdut: f64,
    civil_start: f64,
    civil_end: f64,
    request: &crate::ChartRequest,
) -> Result<DailyTiming, String> {
    calculate_with_ayanamsha(jdut, civil_start, civil_end, request, 0)
}

pub fn calculate_with_ayanamsha(
    jdut: f64,
    civil_start: f64,
    civil_end: f64,
    request: &crate::ChartRequest,
    mode: usize,
) -> Result<DailyTiming, String> {
    if !(1800..=2100).contains(&request.year)
        || !jdut.is_finite()
        || !(0.9..=1.1).contains(&(civil_end - civil_start))
    {
        return Err("Daily timing needs a supported date, instant and civil day.".into());
    }
    crate::validate(request)?;
    let delta_t = |jd| reference_delta_t(jd, request.year, request.month);
    let ayanamsha_at =
        |jd: f64| crate::ayanamsha::android_apparent(jd + delta_t(jd) / SECONDS_PER_DAY, mode);
    let ayanamsha = ayanamsha_at(jdut).ok_or("Unsupported ayanamsha.")?;
    let sample = |jd| {
        let [sun, moon] = sun_moon(jd, delta_t(jd));
        indices(sun, moon, ayanamsha_at(jd).unwrap_or(ayanamsha))
    };
    let active = sample(jdut);
    let mut changes = [None; 4];
    let mut remaining_day_transitions = std::array::from_fn(|_| Vec::new());
    let mut previous = jdut;
    let mut previous_indices = active;
    for step in 1..=24 {
        let end = jdut + f64::from(step) / 12.;
        let newer = sample(end);
        for kind in 0..4 {
            if newer[kind] != previous_indices[kind] {
                let boundary = bisect(previous, end, |instant| {
                    sample(instant)[kind] != previous_indices[kind]
                });
                changes[kind].get_or_insert((boundary, newer[kind]));
                if boundary < civil_end {
                    remaining_day_transitions[kind].push((boundary, label(kind, newer[kind])));
                }
            }
        }
        if end >= civil_end && changes.iter().all(Option::is_some) {
            break;
        }
        previous = end;
        previous_indices = newer;
    }
    let limbs = std::array::from_fn(|kind| Limb {
        kind: ["Tithi", "Nakshatra", "Yoga", "Karana"][kind],
        active: label(kind, active[kind]),
        next: changes[kind].map_or_else(|| "—".into(), |(_, index)| label(kind, index)),
        next_jdut: changes[kind].map(|(jd, _)| jd),
    });
    let [sunrise, sunset] = horizon_crossings(civil_start, civil_end, |instant| {
        solar_altitude(
            instant,
            delta_t(instant),
            request.latitude,
            request.longitude,
        ) > -0.8333
    });
    let [moonrise, moonset] = horizon_crossings(civil_start, civil_end, |instant| {
        crate::eclipses::moon_above_horizon(
            instant,
            delta_t(instant),
            request.latitude,
            request.longitude,
        )
    });
    Ok(DailyTiming {
        limbs,
        remaining_day_transitions,
        sunrise,
        sunset,
        moonrise,
        moonset,
    })
}

/// The Moon crosses at most one 30-degree sign boundary per civil day.
pub fn moon_sign_change(
    bounds: [f64; 2],
    request: &crate::ChartRequest,
    tropical: bool,
    mode: usize,
) -> Result<Option<(f64, usize)>, String> {
    let [start, end] = bounds;
    let delta_t = |jd| reference_delta_t(jd, request.year, request.month);
    let offset = |jd: f64| {
        if tropical {
            Some(0.)
        } else {
            crate::ayanamsha::android_apparent(jd + delta_t(jd) / SECONDS_PER_DAY, mode)
        }
    };
    let initial_offset = offset(start).ok_or("Unsupported ayanamsha.")?;
    let sign = |jd| {
        let jde = jd + delta_t(jd) / SECONDS_PER_DAY;
        let moon = lunar::geocent_ecl_pos(jde).0.long + nutation::nutation(jde).0;
        let sidereal_offset = offset(jd).unwrap_or(initial_offset);
        ((moon.to_degrees() - sidereal_offset).rem_euclid(360.) / 30.) as usize
    };
    let from = sign(start);
    let to = sign(end);
    Ok((from != to)
        .then(|| (bisect(start, end, |instant| sign(instant) != from), to))
        .filter(|&(instant, _)| instant < end))
}

fn horizon_crossings(start: f64, end: f64, above: impl Fn(f64) -> bool) -> [Option<f64>; 2] {
    let mut low = start;
    let mut was_above = above(low);
    let mut crossings = [None; 2];
    for step in 1..=48 {
        let high = start + (end - start) * f64::from(step) / 48.;
        let is_above = above(high);
        if was_above != is_above {
            let instant = bisect(low, high, |instant| above(instant) != was_above);
            crossings[usize::from(!is_above)].get_or_insert(instant);
        }
        low = high;
        was_above = is_above;
    }
    crossings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divisions_keep_exact_elongation_boundaries_and_fixed_karanas() {
        assert_eq!(indices(0., 0., 0.), [0; 4]);
        assert_eq!(indices(0., 12., 0.)[0], 1);
        assert_eq!(indices(10., 4., 0.)[3], 59);
        assert_eq!(indices(350., 20., 0.)[2], 0);
        assert_eq!(karana(1), "Bava");
        assert_eq!(karana(8), "Bava");
        assert_eq!(karana(56), "Vishti");
        assert_eq!(karana(57), "Shakuni");
        assert_eq!(karana(58), "Chatushpada");
        assert_eq!(karana(59), "Naga");
    }

    #[test]
    fn equinox_sunrise_is_close_to_six_hours_at_equator() {
        let request = crate::ChartRequest {
            year: 2024,
            month: 3,
            day: 20,
            latitude: 0.,
            longitude: 0.,
            ..Default::default()
        };
        let midnight = crate::julian_midnight(2024, 3, 20);
        let timing = calculate(midnight + 0.5, midnight, midnight + 1., &request).unwrap();
        let sunrise = (timing.sunrise.unwrap() - midnight) * 24.;
        let sunset = (timing.sunset.unwrap() - midnight) * 24.;
        assert!((5.9..6.2).contains(&sunrise));
        assert!((17.9..18.3).contains(&sunset));
        for limb in timing.limbs {
            assert!(limb.next_jdut.is_some_and(|next| next > midnight + 0.5));
        }
    }

    #[test]
    fn ayanamsha_changes_sidereal_transitions_without_changing_elongation_or_horizon() {
        let request = crate::ChartRequest {
            year: 2024,
            month: 3,
            day: 20,
            ..Default::default()
        };
        let start = crate::julian_midnight(2024, 3, 20);
        let lahiri = calculate(start + 0.5, start, start + 1., &request).unwrap();
        let raman = calculate_with_ayanamsha(start + 0.5, start, start + 1., &request, 3).unwrap();
        for index in [0, 3] {
            assert_eq!(lahiri.limbs[index].next_jdut, raman.limbs[index].next_jdut);
        }
        for index in [1, 2] {
            assert_ne!(lahiri.limbs[index].next_jdut, raman.limbs[index].next_jdut);
        }
        assert_eq!(lahiri.sunrise, raman.sunrise);
        assert_eq!(lahiri.sunset, raman.sunset);
    }

    #[test]
    fn london_moonset_matches_published_eclipse_prediction() {
        let request = crate::ChartRequest {
            year: 2025,
            month: 3,
            day: 14,
            latitude: 51.5074,
            longitude: -0.1278,
            ..Default::default()
        };
        let start = crate::julian_midnight(2025, 3, 14);
        let timing = calculate(start + 0.5, start, start + 1., &request).unwrap();
        let moonset = (timing.moonset.unwrap() - start) * SECONDS_PER_DAY;
        assert!((moonset - 22_920.).abs() < 90., "{moonset:.1}");
        assert!(
            timing
                .moonrise
                .is_some_and(|rise| rise > start + 0.5 && rise < start + 1.)
        );
    }

    #[test]
    fn finds_only_moon_ingresses_inside_the_local_day() {
        let mut by_zodiac = [Vec::new(), Vec::new()];
        for day in 1..=29 {
            let start = crate::julian_midnight(2024, 2, day);
            let request = crate::ChartRequest {
                year: 2024,
                month: 2,
                day,
                ..Default::default()
            };
            for (events, tropical) in by_zodiac.iter_mut().zip([true, false]) {
                if let Some((instant, sign)) =
                    moon_sign_change([start, start + 1.], &request, tropical, 0).unwrap()
                {
                    let chart_sign = |jdut| {
                        let chart =
                            crate::reference_ephemeris::calculate(jdut, 2024, 2, 0., 0.).unwrap();
                        (chart.longitude("Moon", tropical).unwrap() / 30.) as usize
                    };
                    assert_eq!(chart_sign(instant + 2. / SECONDS_PER_DAY), sign);
                    assert_eq!(chart_sign(instant - 2. / SECONDS_PER_DAY), (sign + 11) % 12);
                    assert!((start..start + 1.).contains(&instant));
                    if let Some(&(_, previous_sign)) = events.last() {
                        assert_eq!(sign, (previous_sign + 1) % 12);
                    }
                    events.push((instant, sign));
                }
            }
        }
        assert!(
            by_zodiac
                .iter()
                .all(|events| (12..=14).contains(&events.len()))
        );
        assert_ne!(by_zodiac[0], by_zodiac[1]);
    }

    #[test]
    fn calendar_records_both_tithi_transitions_on_short_lunar_days() {
        let mut double_transition_day = None;
        for day in 1..=29 {
            let request = crate::ChartRequest {
                year: 2024,
                month: 2,
                day,
                ..Default::default()
            };
            let start = crate::julian_midnight(2024, 2, day);
            let timing = calculate(start, start, start + 1., &request).unwrap();
            let changes = &timing.remaining_day_transitions[0];
            if let [(first, first_label), (second, second_label)] = changes.as_slice() {
                assert!(start <= *first && first < second && *second < start + 1.);
                assert_ne!(first_label, second_label);
                assert_eq!(Some(*first), timing.limbs[0].next_jdut);
                double_transition_day = Some(day);
                break;
            }
        }
        assert!(double_transition_day.is_some());
    }
}

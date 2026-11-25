//! NASA catalog dates with independent, approximate local circumstances.
use astro::{lunar, sun};

const AU_KM: f64 = 149_597_870.7;
const EARTH_KM: f64 = 6_378.137;
const SUN_KM: f64 = 695_700.;
const MOON_KM: f64 = 1_737.4;
const DAY: f64 = 86_400.;

#[derive(Debug, Clone, Copy)]
pub struct Eclipse {
    pub lunar: bool,
    pub year: i32,
    pub greatest_jdut: f64,
    pub delta_t: f64,
    pub global_kind: &'static str,
    pub magnitude: f64,
    penumbral_minutes: Option<f64>,
    partial_minutes: Option<f64>,
    total_minutes: Option<f64>,
}

pub fn catalog() -> impl Iterator<Item = Eclipse> {
    include_str!("../data/nasa-eclipses-2001-2100.csv")
        .lines()
        .skip(1)
        .map(|line| {
            let fields: Vec<_> = line.split(',').collect();
            let parse = |index: usize| {
                fields[index]
                    .parse()
                    .expect("verified bundled NASA catalog")
            };
            let lunar = fields[0] == "true";
            let year = parse(1);
            let midnight = crate::julian_midnight(year, parse(2) as u8, parse(3) as u8);
            let delta_t = f64::from(parse(5));
            let global_kind = match (lunar, fields[6].as_bytes()[0]) {
                (false, b'A') => "Annular solar",
                (false, b'T') => "Total solar",
                (false, b'H') => "Hybrid solar",
                (true, b'T') => "Total lunar",
                (true, b'P') => "Partial lunar",
                (true, _) => "Penumbral lunar",
                (false, _) => "Partial solar",
            };
            Eclipse {
                lunar,
                year,
                greatest_jdut: midnight + (f64::from(parse(4)) - delta_t) / DAY,
                delta_t,
                global_kind,
                magnitude: fields[7].parse().expect("catalog magnitude"),
                penumbral_minutes: fields[8].parse().ok(),
                partial_minutes: fields[9].parse().ok(),
                total_minutes: fields[10].parse().ok(),
            }
        })
}

#[derive(Debug, Clone, Copy)]
pub struct LocalCircumstances {
    pub kind: &'static str,
    pub first_jdut: f64,
    pub maximum_jdut: f64,
    pub last_jdut: f64,
    pub rise_during_eclipse_jdut: Option<f64>,
    pub set_during_eclipse_jdut: Option<f64>,
    pub umbral_contacts_jdut: Option<[f64; 2]>,
    pub central_contacts_jdut: Option<[f64; 2]>,
}

type Vector = [f64; 3];

fn point(longitude: f64, latitude: f64, radius: f64, tilt: f64) -> Vector {
    let x = latitude.cos() * longitude.cos();
    let y = latitude.cos() * longitude.sin();
    let z = latitude.sin();
    [
        radius * x,
        radius * (y * tilt.cos() - z * tilt.sin()),
        radius * (y * tilt.sin() + z * tilt.cos()),
    ]
}

fn dot(left: Vector, right: Vector) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn norm(vector: Vector) -> f64 {
    dot(vector, vector).sqrt()
}

fn subtract(left: Vector, right: Vector) -> Vector {
    std::array::from_fn(|index| left[index] - right[index])
}

fn observer(jdut: f64, latitude: f64, longitude: f64) -> (Vector, Vector, f64) {
    let t = (jdut - 2451545.) / 36525.;
    let longitude = (280.46061837 + 360.98564736629 * (jdut - 2451545.) + 0.000387933 * t * t
        - t * t * t / 38710000.
        + longitude)
        .rem_euclid(360.)
        .to_radians();
    let latitude = latitude.to_radians();
    let flattening = 1. / 298.257223563;
    let geocentric = ((1. - flattening) * latitude.tan()).atan();
    let position = [
        EARTH_KM * geocentric.cos() * longitude.cos(),
        EARTH_KM * geocentric.cos() * longitude.sin(),
        EARTH_KM * (1. - flattening) * geocentric.sin(),
    ];
    let zenith = point(longitude, latitude, 1., 0.);
    let obliquity = (23.439291111 - 0.013004167 * t).to_radians();
    (position, zenith, obliquity)
}

fn local_geometry(
    jdut: f64,
    delta_t: f64,
    latitude: f64,
    longitude: f64,
    lunar: bool,
) -> (f64, f64, f64, bool) {
    let jde = jdut + delta_t / DAY;
    let (position, zenith, tilt) = observer(jdut, latitude, longitude);
    let (moon, distance) = lunar::geocent_ecl_pos(jde);
    let moon = subtract(point(moon.long, moon.lat, distance, tilt), position);
    let moon_radius = (MOON_KM / norm(moon)).asin();
    if lunar {
        let visible =
            dot(moon, zenith) / norm(moon) > -(moon_radius + 0.5667_f64.to_radians()).sin();
        return (0., 0., moon_radius, visible);
    }
    let (sun, distance) = sun::geocent_ecl_pos(jde);
    // Use apparent solar longitude in the mean equinox frame used by the observer.
    let aberration = (20.4898_f64 / (3600. * distance)).to_radians();
    let sun = subtract(
        point(sun.long - aberration, sun.lat, distance * AU_KM, tilt),
        position,
    );
    let sun_radius = (SUN_KM / norm(sun)).asin();
    let separation = (dot(sun, moon) / (norm(sun) * norm(moon)))
        .clamp(-1., 1.)
        .acos();
    let visible = dot(sun, zenith) / norm(sun) > -(sun_radius + 0.5667_f64.to_radians()).sin();
    (separation, sun_radius, moon_radius, visible)
}

pub(crate) fn moon_above_horizon(jdut: f64, delta_t: f64, latitude: f64, longitude: f64) -> bool {
    local_geometry(jdut, delta_t, latitude, longitude, true).3
}

fn first_true(mut left: f64, mut right: f64, condition: impl Fn(f64) -> bool) -> f64 {
    while (right - left) * DAY > 1. {
        let mid = (left + right) * 0.5;
        if condition(mid) {
            right = mid;
        } else {
            left = mid;
        }
    }
    right
}

// Return the first and last above-horizon instants in the contact interval.
fn visible_bounds(first: f64, last: f64, visible: impl Fn(f64) -> bool) -> Option<[f64; 2]> {
    const STEPS: i32 = 240;
    let sample = |index| first + (last - first) * f64::from(index) / f64::from(STEPS);
    let mut visible_samples = (0..=STEPS).filter(|&index| visible(sample(index)));
    let first_visible = visible_samples.next()?;
    let last_visible = visible_samples.next_back().unwrap_or(first_visible);
    let start = if first_visible == 0 {
        first
    } else {
        first_true(sample(first_visible - 1), sample(first_visible), &visible)
    };
    let end = if last_visible == STEPS {
        last
    } else {
        first_true(sample(last_visible), sample(last_visible + 1), |time| {
            !visible(time)
        })
    };
    Some([start, end])
}

pub fn local(eclipse: Eclipse, latitude: f64, longitude: f64) -> Option<LocalCircumstances> {
    if !(-90. ..=90.).contains(&latitude) || !(-180. ..=180.).contains(&longitude) {
        return None;
    }
    let center = eclipse.greatest_jdut;
    let geometry = |time| local_geometry(time, eclipse.delta_t, latitude, longitude, eclipse.lunar);
    if eclipse.lunar {
        // The catalog provides durations. Center each interval on greatest eclipse.
        let bounds = |minutes| {
            let half = minutes / (2. * 1440.);
            [center - half, center + half]
        };
        let [first, last] = bounds(eclipse.penumbral_minutes?);
        let [visible_start, visible_end] = visible_bounds(first, last, |time| geometry(time).3)?;
        let umbral_contacts_jdut = eclipse.partial_minutes.map(bounds);
        let central_contacts_jdut = eclipse.total_minutes.map(bounds);
        let phase_visible = |contacts: Option<[f64; 2]>| {
            contacts.is_some_and(|[start, end]| start.max(visible_start) < end.min(visible_end))
        };
        let kind = if phase_visible(central_contacts_jdut) {
            "Total lunar"
        } else if phase_visible(umbral_contacts_jdut) {
            "Partial lunar"
        } else {
            "Penumbral lunar"
        };
        return Some(LocalCircumstances {
            kind,
            first_jdut: first,
            maximum_jdut: center,
            last_jdut: last,
            rise_during_eclipse_jdut: (visible_start > first).then_some(visible_start),
            set_during_eclipse_jdut: (visible_end < last).then_some(visible_end),
            umbral_contacts_jdut,
            central_contacts_jdut,
        });
    }
    let separation = |time| geometry(time).0;
    let mut left = center - 0.23;
    let mut right = center + 0.23;
    for _ in 0..30 {
        let first = left + (right - left) / 3.;
        let second = right - (right - left) / 3.;
        if separation(first) <= separation(second) {
            right = second;
        } else {
            left = first;
        }
    }
    let maximum = (left + right) * 0.5;
    let (gap, sun_radius, moon_radius, _) = geometry(maximum);
    if gap > sun_radius + moon_radius {
        return None;
    }
    let overlaps = |time| {
        let (gap, sun, moon, _) = geometry(time);
        gap <= sun + moon
    };
    let first = first_true(center - 0.23, maximum, overlaps);
    let last = first_true(maximum, center + 0.23, |time| !overlaps(time));
    let [visible_start, visible_end] = visible_bounds(first, last, |time| geometry(time).3)?;
    let central_contacts_jdut = (gap < (sun_radius - moon_radius).abs()).then(|| {
        let central = |time| {
            let (gap, sun, moon, _) = geometry(time);
            gap <= (sun - moon).abs()
        };
        [
            first_true(first, maximum, central),
            first_true(maximum, last, |time| !central(time)),
        ]
    });
    let central_visible = central_contacts_jdut
        .is_some_and(|[start, end]| start.max(visible_start) < end.min(visible_end));
    let kind = if !central_visible || gap > (sun_radius - moon_radius).abs() {
        "Partial solar"
    } else if moon_radius >= sun_radius {
        "Total solar"
    } else {
        "Annular solar"
    };
    let central_contacts_jdut = central_contacts_jdut.filter(|_| central_visible);
    Some(LocalCircumstances {
        kind,
        first_jdut: first,
        maximum_jdut: maximum,
        last_jdut: last,
        rise_during_eclipse_jdut: (visible_start > first).then_some(visible_start),
        set_during_eclipse_jdut: (visible_end < last).then_some(visible_end),
        umbral_contacts_jdut: None,
        central_contacts_jdut,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_complete_and_modern_fixtures_match_nasa() {
        assert_eq!(catalog().count(), 452);
        assert_eq!(catalog().filter(|event| event.lunar).count(), 228);
        assert_eq!(
            catalog()
                .filter(|event| event.partial_minutes.is_some())
                .count(),
            142
        );
        assert_eq!(
            catalog()
                .filter(|event| event.total_minutes.is_some())
                .count(),
            85
        );
        let april = crate::julian_midnight(2024, 4, 8);
        let event = catalog()
            .find(|event| (event.greatest_jdut - april).abs() < 1.)
            .unwrap();
        assert_eq!(event.global_kind, "Total solar");
        assert!(((event.greatest_jdut - april) * DAY - 65_838.3).abs() < 6.);
        let local = local(event, 25.287, -104.138).unwrap();
        assert_eq!(local.kind, "Total solar");
        assert!((local.maximum_jdut - event.greatest_jdut).abs() * DAY < 300.);
        assert!(local.first_jdut < local.maximum_jdut && local.maximum_jdut < local.last_jdut);
        // USNO Solar Eclipse Computer, Dallas 32.783° N, 96.8° W, height 100 m.
        let dallas = super::local(event, 32.783, -96.8).unwrap();
        assert_eq!(dallas.kind, "Total solar");
        let [second, third] = dallas.central_contacts_jdut.unwrap();
        for (jd, expected) in [
            (dallas.first_jdut, 62_595.1),
            (second, 67_238.5),
            (dallas.maximum_jdut, 67_353.3),
            (third, 67_470.0),
            (dallas.last_jdut, 72_155.8),
        ] {
            let seconds = (jd - april) * DAY;
            assert!(
                (seconds - expected).abs() < 15.,
                "{seconds:.1} vs {expected:.1}"
            );
        }
        assert!(super::local(event, 16.84, 96.17).is_none());
    }

    #[test]
    fn albuquerque_annularity_matches_usno_and_dallas_is_partial() {
        let midnight = crate::julian_midnight(2023, 10, 14);
        let event = catalog()
            .find(|event| !event.lunar && (event.greatest_jdut - midnight).abs() < 1.)
            .unwrap();
        let albuquerque = local(event, 35.0844, -106.6504).unwrap();
        assert_eq!(albuquerque.kind, "Annular solar");
        let [second, third] = albuquerque.central_contacts_jdut.unwrap();
        for (jd, expected) in [
            (albuquerque.first_jdut, 54_793.5),
            (second, 59_671.7),
            (albuquerque.maximum_jdut, 59_814.2),
            (third, 59_958.7),
            (albuquerque.last_jdut, 65_363.4),
        ] {
            let seconds = (jd - midnight) * DAY;
            assert!(
                (seconds - expected).abs() < 15.,
                "{seconds:.1} vs {expected:.1}"
            );
        }
        let honolulu = local(event, 21.3069, -157.8583).unwrap();
        assert_eq!(honolulu.kind, "Partial solar");
        let sunrise = (honolulu.rise_during_eclipse_jdut.unwrap() - midnight) * DAY;
        // USNO rounds sunrise to the minute.
        assert!((sunrise - 59_220.).abs() < 60., "{sunrise:.1}");
        assert!(honolulu.set_during_eclipse_jdut.is_none());
        let last = (honolulu.last_jdut - midnight) * DAY;
        assert!((last - 59_919.8).abs() < 20., "{last:.1}");
        let dallas = local(event, 32.783, -96.8).unwrap();
        assert_eq!(dallas.kind, "Partial solar");
        assert!(dallas.central_contacts_jdut.is_none());
    }

    #[test]
    fn lunar_phases_match_nasa_contact_diagram() {
        let midnight = crate::julian_midnight(2025, 3, 14);
        let event = catalog()
            .find(|event| event.lunar && (event.greatest_jdut - midnight).abs() < 1.)
            .unwrap();
        let eclipse = local(event, 32.783, -96.8).unwrap();
        let auckland = local(event, -36.85, 174.76).unwrap();
        assert_eq!(auckland.kind, "Total lunar");
        let moonrise = (auckland.rise_during_eclipse_jdut.unwrap() - midnight) * DAY;
        assert!((moonrise - 23_880.).abs() < 90., "{moonrise:.1}");
        let tokyo = local(event, 35.68, 139.69).unwrap();
        assert_eq!(tokyo.kind, "Penumbral lunar");
        let london = local(event, 51.5074, -0.1278).unwrap();
        assert_eq!(london.kind, "Partial lunar");
        assert!(london.rise_during_eclipse_jdut.is_none());
        let moonset = (london.set_during_eclipse_jdut.unwrap() - midnight) * DAY;
        assert!((moonset - 22_920.).abs() < 90., "{moonset:.1}");
        assert!(moonset < (london.maximum_jdut - midnight) * DAY);
        let [u1, u4] = eclipse.umbral_contacts_jdut.unwrap();
        let [u2, u3] = eclipse.central_contacts_jdut.unwrap();
        // NASA GSFC diagram LE2025Mar14T, contacts in UT.
        for (jd, expected) in [
            (eclipse.first_jdut, 14_244.),
            (u1, 18_573.),
            (u2, 23_159.),
            (eclipse.maximum_jdut, 25_121.7),
            (u3, 27_083.),
            (u4, 31_668.),
            (eclipse.last_jdut, 36_001.),
        ] {
            let seconds = (jd - midnight) * DAY;
            assert!(
                (seconds - expected).abs() < 5.,
                "{seconds:.1} vs {expected:.1}"
            );
        }
    }

    #[test]
    fn partial_solar_contacts_match_usno_at_high_latitude_and_shallow_overlap() {
        let midnight = crate::julian_midnight(2021, 6, 10);
        let event = catalog()
            .find(|event| !event.lunar && (event.greatest_jdut - midnight).abs() < 1.)
            .unwrap();
        for (city, lat, lon, expected) in [
            ("Tromsø", 69.6492, 18.9553, [35_034.8, 39_518.8, 44_012.0]),
            ("Cádiz", 36.5271, -6.2886, [32_284.8, 34_226.8, 36_315.8]),
        ] {
            let result = local(event, lat, lon).unwrap();
            assert_eq!(result.kind, "Partial solar");
            assert!(result.central_contacts_jdut.is_none());
            assert!(result.rise_during_eclipse_jdut.is_none());
            assert!(result.set_during_eclipse_jdut.is_none());
            for (jd, expected) in [result.first_jdut, result.maximum_jdut, result.last_jdut]
                .into_iter()
                .zip(expected)
            {
                let actual = (jd - midnight) * DAY;
                assert!(
                    (actual - expected).abs() < 30.,
                    "{city}: {actual:.1} vs {expected:.1}"
                );
            }
        }
    }

    #[test]
    fn thompson_eclipse_is_already_in_progress_at_sunrise() {
        let midnight = crate::julian_midnight(2021, 6, 10);
        let event = catalog()
            .find(|event| !event.lunar && (event.greatest_jdut - midnight).abs() < 1.)
            .unwrap();
        let result = local(event, 55.7435, -97.8558).unwrap();
        let sunrise = result.rise_during_eclipse_jdut.unwrap();
        assert_eq!(result.kind, "Partial solar");
        assert!(result.first_jdut < sunrise && sunrise < result.maximum_jdut);
        assert!(result.set_during_eclipse_jdut.is_none());
        for (jd, expected, tolerance) in [
            (sunrise, 35_280., 60.),
            (result.maximum_jdut, 36_440.1, 30.),
            (result.last_jdut, 39_882.9, 30.),
        ] {
            let actual = (jd - midnight) * DAY;
            assert!(
                (actual - expected).abs() < tolerance,
                "{actual:.1} vs {expected:.1}"
            );
        }
    }

    #[test]
    fn high_latitude_visible_bounds_catch_brief_graze_and_below_horizon_central_phase() {
        // Synthetic brief horizon-grazing window (< 2 minutes inside a 4-hour interval).
        let first = 2460000.0;
        let last = first + 4.0 / 24.0;
        let graze_start = first + 0.08;
        let graze_end = graze_start + 90.0 / DAY;
        let bounds = visible_bounds(first, last, |t| (graze_start..=graze_end).contains(&t))
            .expect("90-second graze should be detected");
        assert!((bounds[0] - graze_start).abs() * DAY <= 2.0);
        assert!((bounds[1] - graze_end).abs() * DAY <= 2.0);

        // 2021-06-10 annular solar eclipse: find a polar point where geometric annularity
        // occurs below the horizon while part of the partial phase rises above the horizon.
        let midnight = crate::julian_midnight(2021, 6, 10);
        let event = catalog()
            .find(|event| !event.lunar && (event.greatest_jdut - midnight).abs() < 1.)
            .unwrap();
        let qaanaaq = local(event, 77.4667, -69.2333).unwrap();
        assert_eq!(qaanaaq.kind, "Annular solar");
        assert!(qaanaaq.central_contacts_jdut.is_some());

        let mut downgraded = None;
        for lat_step in 55..=85 {
            let lat = f64::from(lat_step);
            for lon_step in (-180..=180).step_by(10) {
                let lon = f64::from(lon_step);
                if let Some(circumstances) = local(event, lat, lon)
                    && circumstances.kind == "Partial solar"
                    && (circumstances.rise_during_eclipse_jdut.is_some()
                        || circumstances.set_during_eclipse_jdut.is_some())
                {
                    let (gap, sun_r, moon_r, _) =
                        local_geometry(circumstances.maximum_jdut, event.delta_t, lat, lon, false);
                    if gap < (sun_r - moon_r).abs() {
                        downgraded = Some(circumstances);
                        break;
                    }
                }
            }
            if downgraded.is_some() {
                break;
            }
        }
        let downgraded =
            downgraded.expect("expected a high-latitude site with central phase below horizon");
        assert_eq!(downgraded.kind, "Partial solar");
        assert!(downgraded.central_contacts_jdut.is_none());
    }
}
